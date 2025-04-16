use mdns_scanner::mdns_scan;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let results = mdns_scan()?;

    for item in &results {
        println!("Name: {}", item.name());
        println!("Address: {}", item.address());
        // println!("Port: {}", item.port());
        println!("Host Name: {}", item.hostname());
    }

    Ok(())
}
