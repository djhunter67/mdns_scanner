use std::{any::Any, sync::Arc};

use models::sqlite::{get_all_items, init_sqlite};
use strum::IntoEnumIterator;
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use zeroconf::{
    avahi::browser::AvahiMdnsBrowser,
    prelude::{TEventLoop, TMdnsBrowser},
    MdnsBrowser, ServiceDiscovery, ServiceType,
};

pub mod models;

use serde::Serialize;
use strum::EnumIter;

pub const DB_PATH: &str = "./mDns.db";

#[derive(EnumIter, Debug, PartialEq, PartialOrd, Eq)]
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

impl std::fmt::Display for ServiceDetect {
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

impl TryFrom<String> for ServiceDetect {
    type Error = ();

    fn try_from(val: String) -> Result<Self, Self::Error> {
        match val.as_str() {
            "http" => Ok(Self::Http),
            "scanner" => Ok(Self::Scanner),
            "androidtvremote2" => Ok(Self::AndroidTvRemote2),
            "uscans" => Ok(Self::Uscans),
            "pdldatastream" => Ok(Self::PdlDataStream),
            "printer" => Ok(Self::Printer),
            "nvshieldremote" => Ok(Self::NvShieldRemote),
            "http-alt" => Ok(Self::HttpAlt),
            "sftp-ssh" => Ok(Self::SftpSsh),
            "ssh" => Ok(Self::Ssh),
            "googlezone" => Ok(Self::GoogleZone),
            "googlecast" => Ok(Self::GoogleCast),
            "companionlink" => Ok(Self::CompanionLink),
            "spotifyconnect" => Ok(Self::SpotifyConnect),
            "airplay" => Ok(Self::AirPlay),
            _ => panic!("Unknown service type"),
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

/// # Getters
///  - `time()`: The time the service was discovered
///  - `date()`: The date the service was discovered
///  - `name()`: The name of the service
///  - `address()`: The IP address of the service
///  - `port()`: The port of the service
///  - `hostname()`: The hostname of the service
/// # Example
/// ```
/// let service = Service {
///     time: String::new(),
///     date: String::new(),
///     name: String::from("Test Service"),
///     address: String::from(192.168.33.268),
///     port: 8080,
///     hostname: String::from("Test Hostname"),
/// };
/// assert_eq!(service.name(), "Test Service");
/// assert_eq!(service.address(), "192.168.33.268");
/// assert_eq!(service.port(), 8080);
/// assert_eq!(service.hostname(), "Test Hostname");
/// ```
///
/// # Service
///  - Struct to hold the discovered service information
/// # Fields
///  - `time`: The time the service was discovered
///  - `date`: The date the service was discovered
///  - `name`: The name of the service
///  - `address`: The IP address of the service
///  - `port`: The port of the service
///  - `hostname`: The hostname of the service
/// # Errors
///  - `()` if any error occurs
/// # Panics
///  - If the service cannot be converted from `ServiceDiscovery`
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

    #[instrument(name = "mdns_scanner", target = "mdns_scanner", level = "info")]
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

impl Service {
    #[must_use]
    #[instrument(
        name = "print the struct as a string",
        target = "mdns_scanner",
        level = "info"
    )]
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
    #[instrument(name = "time discovered", target = "mdns_scanner", level = "info")]
    pub fn time(&self) -> String {
        self.time.clone()
    }

    #[must_use]
    #[instrument(name = "date discovered", target = "mdns_scanner", level = "info")]
    pub fn date(&self) -> String {
        self.date.clone()
    }

    #[must_use]
    #[instrument(name = "name of item", target = "mdns_scanner", level = "info")]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[must_use]
    #[instrument(name = "IP address of item", target = "mdns_scanner", level = "info")]
    pub fn address(&self) -> String {
        self.address.clone()
    }

    #[must_use]
    #[instrument(name = "Port of the item", target = "mdns_scanner", level = "info")]
    pub fn port(&self) -> u16 {
        self.port
    }

    #[must_use]
    #[instrument(
        name = "HOSTNAME of detected devices",
        target = "mdns_scanner",
        level = "info"
    )]
    pub fn hostname(&self) -> String {
        self.hostname.clone()
    }

    #[must_use]
    #[instrument(
        name = "print the Error if the struct is empty",
        target = "mdns_scanner",
        level = "info"
    )]
    pub fn error(err: Box<dyn std::error::Error>) -> Self {
        Self {
            time: String::new(),
            date: String::new(),
            name: String::from("{err:#?}"),
            address: String::new(),
            port: 0,
            hostname: String::new(),
        }
    }
}

/// # Result
///   - Vector of discovered service in the ``Service`` struct
/// # Errors
///   - Box<dyn std::error::Error> if any error occurs
/// # Panics
///   - If the database cannot be created or opened
#[instrument(name = "mdns_scanner", target = "mdns_scan", level = "info")]
pub fn mdns_scan() -> Result<Vec<Service>, Box<dyn std::error::Error>> {
    // let mut conn = rusqlite::Connection::open(format!("{DB_PATH}/{DB_NAME}"))?;
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

    for (i, browse) in browser.iter_mut().enumerate() {
        if let Some(ref scan_items) = scan_items {
            if ServiceDetect::to_iter()
                .get(i)
                .expect("no match")
                .ne(scan_items)
            {
                debug!(
                    "Skipping scan for \'_{}\' devices",
                    ServiceDetect::to_iter().get(i).expect("No service")
                );
                continue;
            }
        }

        info!(
            "Scanning for \'_{}\' devices",
            ServiceDetect::to_iter().get(i).expect("No service")
        );
        // browse.set_network_interface(zeroconf::NetworkInterface::AtIndex(3)); // Pick the connected network port
        browse.set_service_discovered_callback(Box::new(
            move |result: zeroconf::Result<ServiceDiscovery>, _context: Option<Arc<dyn Any>>| {
                // Log instead of printing
                debug!(
                    "Discovered: {}",
                    result.clone().expect("No results").name()
                );
                debug!("Ip Address: {}",
                    result.clone().expect("No results").address()
                );

                let conn = init_sqlite(DB_PATH).expect("Unable to open database");


                conn.execute(
            "INSERT INTO services (time, date, name, address, port, hostname) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                chrono::Local::now().time().format("%H:%M").to_string(),
                // chrono::Local::now().date_naive().to_string(),
                chrono::Local::now().date_naive().format("%Y-%b-%d").to_string(),
                result.clone().expect("Does not exist").name().to_string(),
                result.clone().expect("Does not exist").address().to_string(),
                result.clone().expect("Does not exist").port().to_string(),
                result.expect("Does not exist").host_name().to_string(),
            ],
            )
            .unwrap_or_default();
            },
        ));

        if let Ok(event_loop) = browse.browse_services() {
            debug!("Event loop created");
            let start_time = std::time::Instant::now();

            loop {
                match event_loop.poll(std::time::Duration::from_millis(2)) {
                    Ok(()) => (),
                    Err(err) => {
                        error!("Unable to poll: {err}");
                        return Err("Unable to poll".into());
                    }
                }
                // allow for adjustable scan time
                if start_time.elapsed().as_millis() > scan_time.into() {
                    break;
                }
            }
        } else {
            error!("Unable to create event loop");
            return Err("Unable to create event loop".into());
        }
    }
    let mut conn = init_sqlite(DB_PATH)?;
    let mut results = get_all_items(&mut conn)?;
    let scan_count = results.len();
    info!("Discovered {} mDns devices", scan_count);
    let mut return_vec: Vec<Service> = Vec::with_capacity(scan_count);
    // Filter out anything that is not a poco
    return_vec.extend(
        results
            .iter_mut()
            .filter(|result| result.name().to_lowercase().contains("poco"))
            .map(|result| Service {
                time: result.time(),
                date: result.date(),
                name: result.name(),
                address: result.address(),
                port: result.port(),
                hostname: result.hostname(),
            }),
    );
    // return_vec.extend(results);
    std::fs::remove_file(DB_PATH).unwrap_or_default();

    Ok(return_vec)
}

#[must_use]
/// # Result
///  - `None` if the `RUST_LOG` environment variable is not set
/// # Errors
///  - If the `RUST_LOG` environment variable is set but the value is invalid
/// # Panics
///  - If the `RUST_LOG` environment variable is set but the value is invalid
pub fn get_subcriber(debug: bool) -> impl tracing::Subscriber + Send + Sync {
    let env_filter = if debug {
        String::from("info")
    } else {
        String::from("debug")
    };

    let env_filter = EnvFilter::new(env_filter);

    let stdout_log = tracing_subscriber::fmt::layer()
        .compact()
        // .without_time()
        .pretty();
    let subscriber = Registry::default().with(env_filter).with(stdout_log);

    let json_log = if debug {
        None
    } else {
        let json_log = tracing_subscriber::fmt::layer().json();
        Some(json_log)
    };

    subscriber.with(json_log)
}

/// # Result
///  - `None` if the `RUST_LOG` environment variable is not set
/// # Errors
///  - If the `RUST_LOG` environment variable is set but the value is invalid
/// # Panics
///  - If the `RUST_LOG` environment variable is set but the value is invalid
pub fn init_subscriber(subscriber: impl tracing::Subscriber + Send + Sync) {
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
