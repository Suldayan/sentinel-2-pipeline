use postgres::{Client, NoTls};
use std::error::Error;
use dotenvy::dotenv;

pub fn connect() -> Result<Client, Box<dyn Error>> {
    dotenv().ok();

    let conn_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            panic!(
                "DATABASE_URL is not set. Example:\n\
                 DATABASE_URL=host=localhost user=postgres password=secret dbname=gisdb"
            )
        });

    let client = Client::connect(&conn_str, NoTls)?;
    Ok(client)
}
