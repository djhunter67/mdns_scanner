use crate::Service;

pub fn init_sqlite(db_name: &str, db_path: &str) {
    let conn = rusqlite::Connection::open(format!("{}/{}", db_path, db_name)).unwrap();
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
    )
    .unwrap();
}

pub fn get_count(db_name: &str, db_path: &str) -> u8 {
    let conn = rusqlite::Connection::open(format!("{}/{}", db_path, db_name)).unwrap();
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM services").unwrap();
    let count = stmt
        .query_map([], |row| {
            let count: u8 = row.get_unwrap::<usize, u8>(0);
            Ok(count)
        })
        .unwrap();

    *count.collect::<Vec<_>>().first().unwrap().as_ref().unwrap()
}

pub fn get_all_items(db_name: &str, db_path: &str) -> Vec<Service> {
    let conn = rusqlite::Connection::open(format!("{}/{}", db_path, db_name)).unwrap();
    let mut stmt = conn.prepare("SELECT * FROM services").unwrap();
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
    rows.unwrap().by_ref().for_each(|row| {
        services.push(row.unwrap());
    });

    services
}
