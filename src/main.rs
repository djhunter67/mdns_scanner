use std::{any::Any, sync::Arc};

use mdns_scanner::{
    models::sqlite::{get_all_items, get_count, init_sqlite},
    ServiceDetect, DB_NAME, DB_PATH,
};
use strum::IntoEnumIterator;
use zeroconf::{
    avahi::browser::AvahiMdnsBrowser,
    prelude::{TEventLoop, TMdnsBrowser},
    MdnsBrowser, ServiceDiscovery, ServiceType,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        println!(
            "Scanning for \'_{}\' devices",
            ServiceDetect::to_iter().get(i).expect("No service")
        );
        // browse.set_network_interface(NetworkInterface::AtIndex(3));  // Pick the connected network port
        browse.set_service_discovered_callback(Box::new(
            move |result: zeroconf::Result<ServiceDiscovery>, _context: Option<Arc<dyn Any>>| {
                println!(
                    "\tDiscovered: {}",
                    result.clone().expect("No results").name()
                );
                println!("\t\tIp Address: {}\n",
                    result.clone().expect("No results").address()
                );

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
            Err(err) => panic!("unable to browse services: {err}"),
        };

        let start_time = std::time::Instant::now();

        loop {
            match event_loop.poll(std::time::Duration::from_millis(2)) {
                Ok(()) => (),
                Err(err) => panic!("Unable to poll: {err}"),
            }
            // allow for longer scan time
            if start_time.elapsed().as_secs() > scan_time.into() {
                break;
            }
        }
    }

    println!("\nDiscovered {} mDns devices", get_count(&mut conn)?);

    let all_items = get_all_items(&mut conn);
    for item in &all_items? {
        println!("\nName: {}", item.name());
        println!("IP: {}", item.address());
        println!("Hostname: {}", item.hostname());
    }

    // Destroy the database file
    std::fs::remove_file(format!("{DB_PATH}/{DB_NAME}")).unwrap_or_default();

    Ok(())
}
