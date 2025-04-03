pub mod models;

use std::{fmt::Display, time::Duration};

use models::sqlite::{get_all_items, get_count, init_sqlite};
use serde::Serialize;
use strum::{EnumIter, IntoEnumIterator};
use zeroconf_tokio::{
    MdnsBrowser, MdnsBrowserAsync, ServiceDiscovery, ServiceType, prelude::TMdnsBrowser,
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

impl Display for ServiceDetect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http => write!(f, "http"),
            Self::Scanner => write!(f, "scanner"),
            Self::AndroidTvRemote2 => write!(f, "androidtvremote2"),
            Self::Uscans => write!(f, "uscans"),
            Self::PdlDataStream => write!(f, "pdldatastream"),
            Self::Printer => write!(f, "printer"),
            Self::NvShieldRemote => write!(f, "nvshieldremote"),
            Self::HttpAlt => write!(f, "http-alt"),
            Self::SftpSsh => write!(f, "sftp-ssh"),
            Self::Ssh => write!(f, "ssh"),
            Self::GoogleZone => write!(f, "googlezone"),
            Self::GoogleCast => write!(f, "googlecast"),
            Self::CompanionLink => write!(f, "companionlink"),
            Self::SpotifyConnect => write!(f, "spotifyconnect"),
            Self::AirPlay => write!(f, "airplay"),
        }
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

#[derive(Debug, Default)]
struct ServiceMembers {
    service: Vec<Service>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut servicer = ServiceMembers {
        service: Vec::new(),
    };
    let browser: [MdnsBrowser; ServiceDetect::length()] = ServiceDetect::iter()
        .map(|val| {
            MdnsBrowser::new(
                ServiceType::new(val.into(), "tcp").expect("Unable to create service type"),
            )
        })
        .collect::<Vec<MdnsBrowser>>()
        .try_into()
        .expect("Unable to convert to array");

    let scan_time: u8 = 0;
    let mut conn = match init_sqlite(DB_NAME, DB_PATH).await {
        Ok(object) => object,
        Err(err) => {
            eprintln!("Unable to create connection: {err}");
            return Ok(());
        }
    };

    for (i, browse) in browser.into_iter().enumerate() {
        println!(
            "Initiating scan for {}",
            ServiceDetect::iter()
                .get(i)
                .map_or_else(String::new, |val| val.to_string())
        );

        let mut browse = match MdnsBrowserAsync::new(browse) {
            Ok(object) => {
                println!("Created browser");
                object
            }
            Err(err) => {
                eprintln!("Unable to create browser: {err}");
                break;
            }
        };

        let start_time = std::time::Instant::now();

        // browse.set_network_interface(NetworkInterface::AtIndex(3));
        match browse
            .start_with_timeout(Duration::from_secs(scan_time.into()))
            .await
        {
            Ok(()) => (),
            Err(err) => {
                eprintln!("Unable to start: {err}");
                break;
            }
        };

        println!("Starting service loop\n\n");

        'service_loop: loop {
            // println!("Waiting for discovery");
            let discovery = match browse.next().await {
                Some(result) => {
                    // println!("Received discovery: {}", result.clone()?.name());
                    result
                }
                None => {
                    eprintln!("NO DISCOVERY");
                    break 'service_loop;
                }
            };

            servicer.service.push(
                match Service::try_from(match discovery {
                    Ok(val) => {
                        // println!("Received discovery");
                        val
                    }
                    Err(err) => {
                        eprintln!("Error: {err}");
                        break 'service_loop;
                    }
                }) {
                    Ok(val) => {
                        // println!("Received discovery");
                        val
                    }
                    Err(err) => {
                        eprintln!("Error: {err:#?}");
                        break 'service_loop;
                    }
                },
            );

            save_svc(servicer.service.last().expect("Empty result"), conn.clone()).await?;

            if start_time.elapsed().as_secs() > scan_time.into() {
                // browse.shutdown().await?;
                break 'service_loop;
            }
        }
        browse.shutdown().await.expect("Shutdown failed");
    }

    println!("\nRecalled {} mDns devices", get_count(&mut conn).await?);

    let all_items = get_all_items(&mut conn);
    for item in &all_items.await? {
        println!("\nName: {}", item.name());
        println!("IP: {}", item.address());
        println!("Hostname: {}", item.hostname());
    }

    Ok(())
}

async fn save_svc(service: &Service, conn: limbo::Connection) -> Result<(), limbo::Error> {
    // for svc in &servicer.service {
    let svc = service;
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
                        println!("Successfully saved service: {}", svc.name());
                } else {
                    println!("No rows affected");
                }},
                Err(err) => {
                    eprintln!("Error saving service: {err}");
                    return Err(err);
                }
            }
    // }

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
