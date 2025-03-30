pub mod models;

use std::{any::Any, sync::Arc};

use models::sqlite::{get_all_items, get_count, init_sqlite};
use serde::Serialize;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = rusqlite::Connection::open(format!("{DB_PATH}/{DB_NAME}"))?;
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
    init_sqlite(DB_NAME, DB_PATH)?;

    for browse in &mut browser {
        // let captured_svc: Arc<Mutex<Vec<Service>>> = Arc::new(Mutex::default());
        println!("Initiating {scan_time} second scan");
        // browse.set_network_interface(NetworkInterface::AtIndex(3));
        browse.set_service_discovered_callback(Box::new(
            move |result: zeroconf::Result<ServiceDiscovery>, _context: Option<Arc<dyn Any>>| {
                println!(
                    "\tDiscovered: {}",
                    result.clone().expect("No results").name()
                );

                let conn = rusqlite::Connection::open(DB_NAME).expect("DB does not exist");

                conn.execute(
		    "INSERT INTO services (time, date, name, address, port, hostname) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
		    [
			chrono::Local::now().time().to_string(),
			chrono::Local::now().date_naive().to_string(),
			result.clone().expect("Does not exist").name().to_string(),
			result.clone().expect("Does not exist").address().to_string(),
		        result.clone().expect("Does not exist").port().to_string(),
		        result.expect("Does not exist").host_name().to_string(),
		    ],
	        )
		    .unwrap_or_default();
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

            if start_time.elapsed().as_secs() > scan_time.into() {
                break;
            }
        }
    }

    println!("\nDiscovered {} mDns devices", get_count(&mut conn)?);

    let all_items = get_all_items(&mut conn);
    for item in &all_items? {
        println!("\nName: {}", item.name());
        println!("IP: {}", item.address());
        println!("Hostname: {}", item.hostname());
    }

    Ok(())
}

#[derive(Default, Serialize, Debug)]
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
