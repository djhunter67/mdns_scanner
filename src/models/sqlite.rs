use crate::Service;

/// # Panics
///   - Attempt to create a file that already exists
/// # Errors
///   - Unable to create the db file
pub fn init_sqlite(db_name: &str, db_path: &str) -> Result<(), rusqlite::Error> {
    let conn = match rusqlite::Connection::open(format!("{db_path}/{db_name}")) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Unable to open the DB: {err}");
            std::fs::File::create_new(format!("{db_path}/{db_name}"))
                .unwrap_or_else(|err| panic!("DB file already exists: {err}"));
            return Err(rusqlite::Error::InvalidPath(db_path.into()));
        }
    };
    conn.execute(
        "CREATE TABLE IF NOT EXISTS services (
	    id INTEGER PRIMARY KEY,
	    name TEXT UNIQUE NOT NULL,
	    address TEXT NOT NULL,
	    port INTEGER NOT NULL,
	    protocol TEXT NOT NULL,
	    txt TEXT
	)",
        (),
    )?;

    Ok(())
}

/// # Panics
///  - There is no result from the database
/// # Errors
///  - None
pub fn get_count(db_name: &str, db_path: &str) -> Result<u8, rusqlite::Error> {
    let conn = rusqlite::Connection::open(format!("{db_path}/{db_name}"))?;
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
pub fn get_all_items(db_name: &str, db_path: &str) -> Result<Vec<Service>, rusqlite::Error> {
    let conn = rusqlite::Connection::open(format!("{db_path}/{db_name}"))?;
    let mut stmt = conn.prepare("SELECT * FROM services")?;
    let rows = stmt.query_map([], |row| {
        let _id: u32 = row.get_unwrap(0);
        let name: String = row.get_unwrap(1);
        let address: String = row.get_unwrap(2);
        let port: u16 = row.get_unwrap(3);
        let protocol: String = row.get_unwrap(4);
        let txt: String = row.get_unwrap(5);

        Ok(Service {
            name,
            address,
            port,
            protocol,
            txt: serde_json::from_str(&txt).ok(),
        })
    });

    let mut services: Vec<Service> = Vec::new();
    rows?.by_ref().for_each(|row| {
        services.push(row.expect("Unable to use the returned row"));
    });

    Ok(services)
}
