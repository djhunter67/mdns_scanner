use std::{any::Any, sync::Arc};

use models::sqlite::{get_all_items, get_count, init_sqlite};
use strum::IntoEnumIterator;
use zeroconf::{
    MdnsBrowser, ServiceDiscovery, ServiceType,
    avahi::browser::AvahiMdnsBrowser,
    prelude::{TEventLoop, TMdnsBrowser},
};

pub mod models;

use std::fmt::Display;

use serde::Serialize;
use strum::EnumIter;

pub const DB_PATH: &str = "./";
pub const DB_NAME: &str = "mDns.db";

#[derive(EnumIter)]
pub enum ServiceDetect {
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
    #[must_use]
    pub const fn length() -> usize {
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

    #[must_use]
    pub const fn to_iter() -> [Self; Self::length()] {
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
    #[must_use]
    pub fn as_string(&self) -> String {
        format!(
            "Name: {}, Address: {}, Port: {}, Protocol: {}",
            self.name(),
            self.address(),
            self.port(),
            self.hostname(),
        )
    }

    #[must_use]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[must_use]
    pub fn address(&self) -> String {
        self.address.clone()
    }

    const fn port(&self) -> u16 {
        self.port
    }

    #[must_use]
    pub fn hostname(&self) -> String {
        self.hostname.clone()
    }
}

/// # Result
///   - Vector of discovered service in the ``Service`` struct
/// # Errors
///   - Box<dyn std::error::Error> if any error occurs
/// # Panics
///   - If the database cannot be created or opened
pub fn mdns_scan() -> Result<Vec<Service>, Box<dyn std::error::Error>> {
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

    let scan_time: u8 = 0;
    init_sqlite(DB_NAME, DB_PATH)?;

    for (i, browse) in browser.iter_mut().enumerate() {
        if ServiceDetect::to_iter()
            .get(i)
            .expect("No data to scan")
            .to_string()
            .ne("http")
        {
            continue;
        }

        println!(
            "Scanning for \'_{}\' devices",
            ServiceDetect::to_iter().get(i).expect("No service")
        );
        // browse.set_network_interface(zeroconf::NetworkInterface::AtIndex(3)); // Pick the connected network port
        browse.set_service_discovered_callback(Box::new(
            move |result: zeroconf::Result<ServiceDiscovery>, _context: Option<Arc<dyn Any>>| {
                // Log instead of printing
                // println!(
                    // "\tDiscovered: {}",
                    // result.clone().expect("No results").name()
                // );
                // println!("\t\tIp Address: {}\n",
                    // result.clone().expect("No results").address()
                // );

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
            Err(err) => panic!("Unable to create event loop: {err}"),
        };

        let start_time = std::time::Instant::now();

        loop {
            match event_loop.poll(std::time::Duration::from_millis(2)) {
                Ok(()) => (),
                Err(err) => panic!("Unable to poll: {err}"),
            }
            // allow for adjustable scan time
            if start_time.elapsed().as_secs() > scan_time.into() {
                break;
            }
        }
    }
    // Log this
    // println!("\nDiscovered {} mDns devices", get_count(&mut conn)?);
    let mut return_vec: Vec<Service> = Vec::with_capacity(get_count(&mut conn)?.into());
    return_vec.extend(get_all_items(&mut conn)?);
    std::fs::remove_file(format!("{DB_PATH}/{DB_NAME}")).unwrap_or_default();

    Ok(return_vec)
}
