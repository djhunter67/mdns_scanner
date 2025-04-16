pub mod models;

use std::fmt::Display;
use zeroconf::ServiceDiscovery;

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
