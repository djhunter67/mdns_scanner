use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use zeroconf::{
    prelude::{TEventLoop, TMdnsBrowser},
    MdnsBrowser, NetworkInterface, ServiceDiscovery, ServiceType,
};

#[derive(Default)]
struct Service {
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
            txt: serde_json::from_str(&serde_json::to_string(&val.txt().clone().unwrap()).unwrap())
                .ok(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let captured_svc: Arc<Mutex<Vec<Service>>> = Arc::new(Mutex::default());
    let mut browser = MdnsBrowser::new(ServiceType::new("http", "tcp")?);
    // GEt the version of config.toml
    let root = env!("CARGO_MANIFEST_DIR");
    println!("Root: {}", root);
    init_sqlite("mDns.db", root);

    browser.set_network_interface(NetworkInterface::AtIndex(3));
    browser.set_service_discovered_callback(Box::new(
        move |result: zeroconf::Result<ServiceDiscovery>, _context: Option<Arc<dyn Any>>| {
            captured_svc.lock().unwrap().push(result.unwrap().into());

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
	    .unwrap();
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

fn init_sqlite(db_name: &str, db_path: &str) {
    let conn = rusqlite::Connection::open(format!("{}/{}", db_path, db_name)).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS services (
	    id INTEGER PRIMARY KEY,
	    name TEXT NOT NULL,
	    address TEXT NOT NULL,
	    port INTEGER NOT NULL,
	    protocol TEXT NOT NULL,
	    txt TEXT
	)",
        (),
    )
    .unwrap();
}
