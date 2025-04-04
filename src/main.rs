pub mod models;

use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use models::sqlite::{get_all_items, get_count, init_sqlite};
use strum::{EnumIter, IntoEnumIterator};
use zeroconf::{
    avahi::browser::AvahiMdnsBrowser,
    prelude::{TEventLoop, TMdnsBrowser},
    MdnsBrowser, ServiceDiscovery, ServiceType,
};

const DB_PATH: &str = "./";
const DB_NAME: &str = "mDns.db";

#[derive(EnumIter)]
enum ServiceDetect {
    Http,
    Scanner,
    AndroidTvRemote2,
    Uscans,
    PdlDataStream,
    Printer,
    NvShieldRemote,
    HttpAlt,
    SftpSsh,
    Ssh,
    GoogleZone,
    GoogleCast,
    CompanionLink,
    SpotifyConnect,
    AirPlay,
}

impl ServiceDetect {
    const fn length() -> usize {
        [
            Self::Http,
            Self::Scanner,
            Self::AndroidTvRemote2,
            Self::Uscans,
            Self::PdlDataStream,
            Self::Printer,
            Self::NvShieldRemote,
            Self::HttpAlt,
            Self::SftpSsh,
            Self::Ssh,
            Self::GoogleZone,
            Self::GoogleCast,
            Self::CompanionLink,
            Self::SpotifyConnect,
            Self::AirPlay,
        ]
        .len()
    }
}

impl From<ServiceDetect> for &str {
    fn from(val: ServiceDetect) -> Self {
        match val {
            ServiceDetect::Http => "http",
            ServiceDetect::Scanner => "scanner",
            ServiceDetect::AndroidTvRemote2 => "androidtvremote2",
            ServiceDetect::Uscans => "uscans",
            ServiceDetect::PdlDataStream => "pdldatastream",
            ServiceDetect::Printer => "printer",
            ServiceDetect::NvShieldRemote => "nvshieldremote",
            ServiceDetect::HttpAlt => "http-alt",
            ServiceDetect::SftpSsh => "sftp-ssh",
            ServiceDetect::Ssh => "ssh",
            ServiceDetect::GoogleZone => "googlezone",
            ServiceDetect::GoogleCast => "googlecast",
            ServiceDetect::CompanionLink => "companionlink",
            ServiceDetect::SpotifyConnect => "spotifyconnect",
            ServiceDetect::AirPlay => "airplay",
        }
    }
}

#[derive(Debug)]
struct ServiceMembers {
    service: &'static [Service; 12],
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auto_servicer = Arc::new(Mutex::new(ServiceMembers {
        service: core::array::from_fn(|_| &[Arc::new(Service::default()); 12]),
    }));

    let servicer = Arc::clone(&auto_servicer);
    let mut browser: [AvahiMdnsBrowser; ServiceDetect::length()] = ServiceDetect::iter()
        .map(|val| {
            MdnsBrowser::new(
                ServiceType::new(val.into(), "tcp").expect("Unable to create service type"),
            )
        })
        .collect::<Vec<AvahiMdnsBrowser>>()
        .try_into()
        .expect("Unable to convert to array");

    let scan_time: u8 = 1;
    let mut conn = init_sqlite(DB_NAME, DB_PATH).await?;

    for browse in &mut browser {
        // let captured_svc: Arc<Mutex<Vec<Service>>> = Arc::new(Mutex::default());
        println!("Initiating {scan_time} second scan");
        // browse.set_network_interface(NetworkInterface::AtIndex(3));
        browse.set_service_discovered_callback(Box::new(
            move |result: zeroconf::Result<ServiceDiscovery>, context: Option<Arc<dyn Any>>| {
                println!(
                    "\tDiscovered: {}",
                    result.as_ref().expect("No results").name()
                );

                match context.as_ref() {
                    Some(object) => {
                        if let Some(object) = object.downcast_ref::<Arc<Mutex<ServiceMembers>>>() {
                            object
                                .as_ref()
                                .lock()
                                .expect("Lock is poisoned")
                                .service
                                .to_vec()
                                .push(
                                    Service::try_from(result.expect("Does not exist"))
                                        .expect("Unable to convert"),
                                );
                        }
                        println!(
                            "Service count is: {}",
                            object
                                .downcast_ref::<Arc<Mutex<ServiceMembers>>>()
                                .expect("Unable to downcast")
                                .lock()
                                .expect("Lock is poisoned")
                                .service
                                .len()
                        );
                    }
                    None => {
                        println!("continuing...");
                        // turn the array into a vector
                        let mut servicer = Arc::clone(&servicer);
                        // Arc::into_inner(servicer).expect("no data").service.push(
                        //     Service::try_from(result.expect("Does not exist"))
                        //         .expect("Unable to convert"),
                        // );
                        servicer
                            .lock()
                            .expect("Lock is poisoned")
                            .service
                            .to_vec()
                            .push(
                                Service::try_from(result.expect("Does not exist"))
                                    .expect("Unable to convert"),
                            );
                        println!(
                            "Service count is: {}",
                            servicer.lock().expect("Lock is poisoned").service.len()
                        );
                    }
                };

                // let mut context = context.lock().expect("Lock is poisoned");

                // context.service.push(w
                //     Service::try_from(result.expect("Does not exist")).expect("Unable to convert"),
                // );
                // println!("Service count is: {}", context.service.len());
            },
        ));

        let event_loop = match browse.browse_services() {
            Ok(object) => object,
            Err(err) => panic!("unable to browse services: {err}"),
        };

        let start_time = std::time::Instant::now();

        loop {
            match event_loop.poll(std::time::Duration::from_millis(2)) {
                Ok(()) => (),
                Err(err) => panic!("Unable to poll: {err}"),
            };

            if start_time.elapsed().as_secs() >= scan_time.into() {
                break;
            }
        }
    }

    tokio::task::block_in_place(|| {
        save_svc(&servicer.lock().expect("lock is poisoned"), conn.clone())
    })
    .await?;

    // println!("Service count is: {}", servicer.service.len());

    println!("\nRecalled {} mDns devices", get_count(&mut conn).await?);

    let all_items = get_all_items(&mut conn);
    for item in &all_items.await? {
        println!("\nName: {}", item.name());
        println!("IP: {}", item.address());
        println!("Hostname: {}", item.hostname());
    }

    Ok(())
}

async fn save_svc(servicer: &ServiceMembers, conn: limbo::Connection) -> Result<(), limbo::Error> {
    for svc in servicer.service {
        // let svc = service;
        match conn.execute(
                "INSERT INTO services (time, date, name, address, port, hostname) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                &[
                    chrono::Local::now().time().to_string(),
                    chrono::Local::now().date_naive().to_string(),
                    svc.name().to_string(),
                    svc.address().to_string(),
                    svc.port().to_string(),
                    svc.hostname().to_string(),
                ],
            )
            .await {
                Ok(result) => {
                    if  result == 0 {
                        println!("\tSuccessfully saved service: {}\n", svc.name());
                } else {
                    println!("No rows affected");
                }},
                Err(err) => {
                    eprintln!("Error saving service: {err}");
                    return Err(err);
                }
            }
    }

    Ok(())
}

#[derive(Default, Debug, Clone)]
pub struct Service {
    time: String,
    date: String,
    name: String,
    address: String,
    port: u16,
    hostname: String,
}

impl TryFrom<ServiceDiscovery> for Service {
    type Error = ();

    fn try_from(val: ServiceDiscovery) -> Result<Self, Self::Error> {
        Ok(Self {
            time: String::new(),
            date: String::new(),
            name: val.name().to_string(),
            address: val.address().to_string(),
            port: *val.port(),
            hostname: val.host_name().to_string(),
        })
    }
}

#[allow(dead_code)]
impl Service {
    fn as_string(&self) -> String {
        format!(
            "Name: {}, Address: {}, Port: {}, Protocol: {}",
            self.name(),
            self.address(),
            self.port(),
            self.hostname(),
        )
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn address(&self) -> &str {
        &self.address
    }

    const fn port(&self) -> u16 {
        self.port
    }

    fn hostname(&self) -> &str {
        &self.hostname
    }
}
