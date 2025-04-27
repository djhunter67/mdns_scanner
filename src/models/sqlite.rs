use rusqlite::{Connection, DatabaseName};
use tracing::{debug, error, info, instrument};

use crate::Service;

/// # Panics
///   - Attempt to create a file that already exists
/// # Errors
///   - Unable to create the db file
#[instrument(
    name = "Sqlite connection getter",
    target = "mdns_scanner",
    level = "info"
)]
pub fn init_sqlite(db_path: &str) -> Result<(), rusqlite::Error> {
    debug!("Establishing the db connection to initialize the db");
    let conn = match rusqlite::Connection::open(db_path) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Unable to open the DB: {err}");
            std::fs::File::create_new(db_path)
                .unwrap_or_else(|err| panic!("DB file already exists: {err}"));
            return Err(rusqlite::Error::InvalidPath(db_path.into()));
        }
    };

    info!("Applying sqlite pragma optimizations");
    conn.pragma_update(Some(DatabaseName::Main), "foriegn_key", "true")?;
    conn.pragma_update(Some(DatabaseName::Main), "journal_mode", "WAL")?;
    conn.pragma_update(Some(DatabaseName::Main), "busy_timeout", "5000")?;
    conn.pragma_update(Some(DatabaseName::Main), "synchronous", "NORMAL")?;
    conn.pragma_update(Some(DatabaseName::Main), "cache_size", "2000")?;
    conn.pragma_update(Some(DatabaseName::Main), "temp_store", "memory")?;
    conn.pragma_update(Some(DatabaseName::Main), "mmap_size", "4096")?;
    debug!("Creating sqlite table if not exists");
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS services (
	    id INTEGER PRIMARY KEY,
            time TEXT NOT NULL,
            date TEXT NOT NULL,            
	    name TEXT UNIQUE NOT NULL,
	    address TEXT NOT NULL,
	    port INTEGER NOT NULL,
            hostname TEXT NOT NULL,
        service_type TEXT NOT NULL
           ) STRICT;",
        [],
    ) {
        Ok(_) => (),
        Err(err) => error!("TABLE creation error: {err}"),
    }

    debug!("Closing the init_sql() db connection");
    conn.close().map_err(|err| {
        error!("Unable to close the connection: {err:#?}");
        rusqlite::Error::InvalidQuery
    })?;

    Ok(())
}

/// # Panics
///  - There is no result from the database
/// # Errors
///  - None
#[instrument(
    name = "Number of scan results",
    target = "mdns_scanner",
    level = "info"
)]
pub fn get_count(conn: &mut Connection) -> Result<u8, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM services")?;
    let count = stmt.query_map([], |row| {
        let count: u8 = row.get_unwrap::<usize, u8>(0);
        Ok(count)
    })?;

    Ok(*count
        .collect::<Vec<_>>()
        .first()
        .expect("No values to get")
        .as_ref()
        .expect("Can't reference a non-existent value"))
}

/// # Panics
///   - ``SqliteDB`` returns no information stored
/// # Errors
///   - None
#[instrument(
    name = "Retrieve all mDns scan results",
    target = "mdns_scanner",
    level = "info"
)]
pub fn get_all_items(conn: &mut Connection) -> Result<Vec<Service>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM services")?;
    let rows = stmt.query_map([], |row| {
        // let _id: u32 = row.get_unwrap(0);
        let time: String = row.get_unwrap(1);
        let date: String = row.get_unwrap(2);
        let name: String = row.get_unwrap(3);
        let address: String = row.get_unwrap(4);
        let port: u16 = row.get_unwrap(5);
        let hostname: String = row.get_unwrap(6);
        let service_type: String = row.get_unwrap(7);

        Ok(Service {
            time,
            date,
            name,
            address,
            port,
            hostname,
            service_type,
        })
    });

    let mut services: Vec<Service> = Vec::new();
    rows?.by_ref().for_each(|row| {
        services.push(row.expect("Unable to use the returned row"));
    });

    Ok(services)
}
