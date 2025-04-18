use std::error::Error;

use mdns_scanner::{init_subscriber, mdns_scan, Service, ServiceDetect};
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    let tracing_subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    init_subscriber(tracing_subscriber);
    let items_to_scan: ServiceDetect = ServiceDetect::Http;
    let results: Vec<Service> = mdns_scan(items_to_scan)?;

    for item in &results {
        info!("Name: {}", item.name());
        info!("Address: {}", item.address());
        // println!("Port: {}", item.port());
        info!("Host Name: {}", item.hostname());
    }

    Ok(())
}
