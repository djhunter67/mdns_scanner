use std::{any::Any, sync::Arc};

use zeroconf::{
    prelude::{TEventLoop, TMdnsBrowser},
    MdnsBrowser, NetworkInterface, ServiceDiscovery, ServiceType,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut browser = MdnsBrowser::new(ServiceType::new("http", "tcp")?);

    browser.set_network_interface(NetworkInterface::AtIndex(3));
    browser.set_service_discovered_callback(Box::new(
        |result: zeroconf::Result<ServiceDiscovery>, _context: Option<Arc<dyn Any>>| {
            println!("Service: {:#?}", result.expect("Service discovery failed"));
        },
    ));

    let event_loop = match browser.browse_services() {
        Ok(object) => object,
        Err(err) => panic!("unable to browse services: {err}"),
    };

    println!("Browsing for services...");
    loop {
        event_loop.poll(std::time::Duration::from_secs(1))?;

        // Implement a delay to avoid excessive CPU usage
    }
}
