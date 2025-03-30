pub mod models;

use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use models::sqlite::{get_all_items, get_count, init_sqlite};
use serde::Serialize;
use zeroconf::{
    prelude::{TEventLoop, TMdnsBrowser},
    MdnsBrowser, NetworkInterface, ServiceDiscovery, ServiceType,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let captured_svc: Arc<Mutex<Vec<Service>>> = Arc::new(Mutex::default());
    let mut browser = MdnsBrowser::new(ServiceType::new("http", "tcp")?);
    let scan_time: u8 = 1;
    // let root = env!("CARGO_MANIFEST_DIR");
    let root = "./";
    init_sqlite("mDns.db", root);

    println!("Initiating {scan_time} second scan");

    browser.set_network_interface(NetworkInterface::AtIndex(3));
    //let context = browser.set_context(Arc::new());
    browser.set_service_discovered_callback(Box::new(
        move |result: zeroconf::Result<ServiceDiscovery>, _context: Option<Arc<dyn Any>>| {
            println!(
                "\tDiscovered: {}",
                result.clone().expect("No results").name()
            );

            captured_svc.lock()?.push(result.unwrap().into());

            let conn = rusqlite::Connection::open("mDns.db").unwrap();

            let service = captured_svc.lock().unwrap().pop().unwrap();
            conn.execute(
		"INSERT INTO services (name, address, port, protocol, txt) VALUES (?1, ?2, ?3, ?4, ?5)",
		[
		    &service.name,
		    &service.address,
		    &service.port.to_string(),
		    &service.protocol,
		    &service.txt.unwrap().to_string(),
		],

	    )
		.unwrap_or_default();
        },
    ));

    let event_loop = match browser.browse_services() {
        Ok(object) => object,
        Err(err) => panic!("unable to browse services: {err}"),
    };

    let start_time = std::time::Instant::now();

    loop {
        match event_loop.poll(std::time::Duration::from_millis(157)) {
            Ok(()) => (),
            Err(err) => panic!("Unable to poll: {err}"),
        };

        if start_time.elapsed().as_secs() > scan_time.into() {
            break;
        }
    }

    println!("\nDiscovered {} mDns devices", get_count("mDns.db", root));

    let all_items = get_all_items("mDns.db", root);
    all_items.iter().for_each(|item| {
        println!("\nName: {}", item.name());
        println!("IP: {}", item.address)
    });

    Ok(())
}

#[derive(Default, Serialize, Debug)]
pub struct Service {
    name: String,
    address: String,
    port: u16,
    protocol: String,
    txt: Option<serde_json::value::Value>,
}

impl From<ServiceDiscovery> for Service {
    fn from(val: ServiceDiscovery) -> Self {
        Service {
            name: val.name().to_string(),
            address: val.address().to_string(),
            port: *val.port(),
            protocol: val.service_type().protocol().to_string(),
            txt: serde_json::from_str(
                &serde_json::to_string(&val.txt().clone().unwrap_or_default()).unwrap(),
            )
            .ok(),
        }
    }
}

#[allow(dead_code)]
impl Service {
    fn as_string(&self) -> String {
        format!(
            "Name: {}, Address: {}, Port: {}, Protocol: {}, Txt: {}",
            self.name(),
            self.address(),
            self.port(),
            self.protocol(),
            self.txt().unwrap()
        )
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn address(&self) -> &str {
        &self.address
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn protocol(&self) -> &str {
        &self.protocol
    }

    fn txt(&self) -> Option<&serde_json::value::Value> {
        self.txt.as_ref()
    }
}
