use std::error::Error;

use mdns_scanner::{Service, mdns_scan};

fn main() -> Result<(), Box<dyn Error>> {
    let results: Vec<Service> = mdns_scan()?;

    for item in &results {
        println!("Name: {}", item.name());
        println!("Address: {}", item.address());
        // println!("Port: {}", item.port());
        println!("Host Name: {}", item.hostname());
    }

    Ok(())
}
