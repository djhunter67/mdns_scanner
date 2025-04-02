use limbo::{Connection, params::IntoValue};

use crate::Service;

/// # Panics
///   - Attempt to create a file that already exists
/// # Errors
///   - Unable to create the db file
pub async fn init_sqlite(db_name: &str, db_path: &str) -> Result<Connection, limbo::Error> {
    if db_name.eq(":memory:") {
        let db = match limbo::Builder::new_local(":memory:").build().await {
            Ok(conn) => conn,
            Err(err) => {
                eprintln!("Unable to open the DB: {err}");
                return Err(limbo::Error::ToSqlConversionFailure(db_path.into()));
            }
        };

        let conn = db.connect()?;
        match conn
            .execute(
                "CREATE TABLE IF NOT EXISTS services (
        id INTEGER PRIMARY KEY,
            time TEXT NOT NULL,
            date TEXT NOT NULL,            
        name TEXT UNIQUE NOT NULL,
        address TEXT NOT NULL,
        port INTEGER NOT NULL,
        hostname TEXT NOT NULL
    )",
                (),
            )
            .await
        {
            Ok(_) => (),
            Err(err) => eprintln!("TABLE creation error: {err}"),
        };

        return Ok(conn);
    }

    let db = match limbo::Builder::new_local(&format!("{db_path}/{db_name}"))
        .build()
        .await
    {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Unable to open the DB: {err}");
            std::fs::File::create_new(format!("{db_path}/{db_name}"))
                .unwrap_or_else(|err| panic!("DB file already exists: {err}"));
            return Err(limbo::Error::ToSqlConversionFailure(db_path.into()));
        }
    };

    let conn = db.connect()?;
    match conn
        .execute(
            "CREATE TABLE IF NOT EXISTS services (
	    id INTEGER PRIMARY KEY,
            time TEXT NOT NULL,
            date TEXT NOT NULL,            
	    name TEXT UNIQUE NOT NULL,
	    address TEXT NOT NULL,
	    port INTEGER NOT NULL,
	    hostname TEXT NOT NULL
)",
            (),
        )
        .await
    {
        Ok(_) => (),
        Err(err) => eprintln!("TABLE creation error: {err}"),
    };

    Ok(conn)
}

/// # Panics
///  - There is no result from the database
/// # Errors
///  - None
pub async fn get_count(conn: &mut Connection) -> Result<u8, limbo::Error> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM services").await?;
    let mut count = stmt.query(()).await?;

    let rows = count.next().await.expect("No rows returned");

    // |row| {let count: u8 = row.get_unwrap::<usize, u8>(0);
    // Ok(count)
    // }

    Ok(rows.map_or(0, |row| {
        let count: u8 = u8::try_from(
            *row.get_value(0)
                .expect("No value returned")
                .as_integer()
                .expect("No integer returned"),
        )
        .expect("Truncated the u64 to a u8");
        count
    }))

    // Ok(match rows {
    //     Some(row) => *row
    //         .get_value(0)
    //         .expect("No value returned")
    //         .as_integer()
    //         .expect("No integer returned") as u8,
    //     None => 0,
    // })
}

/// # Panics
///   - ``SqliteDB`` returns no information stored
/// # Errors
///   - None
pub async fn get_all_items(conn: &mut Connection) -> Result<Vec<Service>, limbo::Error> {
    let mut stmt = conn.prepare("SELECT * FROM services").await?;
    let rows: Service = match stmt.query(()).await?.next().await.expect("query error") {
        Some(row) => row,
        None => {
            return Ok(vec![]);
        }
    }
    .get_value(0)
    .expect("No value returned")
    .into_value()
    .map(|row| {
        let _id: u32 = u32::try_from(*row.as_integer().expect("No text returned"))
            .expect("Truncated the u64 to a u32");
        let time: String = row.as_text().expect("No text returned").to_string();
        let date: String = row.as_text().expect("No text returned").to_string();
        let name: String = row.as_text().expect("No text returned").to_string();
        let address: String = row.as_text().expect("No text returned").to_string();
        let port: u16 = u16::try_from(*row.as_integer().expect("No integer returned"))
            .expect("Truncated the u64 to a u16");
        let hostname: String = row.as_text().expect("No text returned").to_string();

        Service {
            time,
            date,
            name,
            address,
            port,
            hostname,
        }
    })
    .expect("No value returned");

    Ok(vec![rows])
}
