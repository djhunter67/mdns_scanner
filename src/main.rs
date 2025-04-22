use std::error::Error;

use mdns_scanner::{get_subcriber, init_subscriber, mdns_scan, Service};
use tracing::{info, instrument, warn};

#[instrument(name = "mdns_scan main fn", target = "mdns_scanner", level = "info")]
fn main() -> Result<(), Box<dyn Error>> {
    let tracing_subscriber = get_subcriber(true);

    init_subscriber(tracing_subscriber);
    // let results: Vec<Service> = mdns_scan(Some(ServiceDetect::Http), Some("poco"))?;
    // let results: Vec<Service> = mdns_scan(Some(ServiceDetect::Http), None)?;
    let results: Vec<Service> = mdns_scan(None, None)?; // Open scan

    for item in &results {
        warn!("Name: {}", item.name());
        warn!("Address: {}", item.address());
        warn!("Port: {}", item.port());
        warn!("Host Name: {}", item.hostname());
    }

    info!("Scan results: {}", results.len());

    Ok(())
}
