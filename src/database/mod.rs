// Database setup for auth and user bits
// Tables
//    endpoint
//    publisher

use std::{
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use iroh::{EndpointId, PublicKey};
use tracing::info;
use turso::{Builder, Connection, params};

// Database structs
// Could be an orm , but I prefer raw sql

#[derive(Debug,Clone)]
#[repr(u32)]
pub enum EndpointStatus {
    Active,
}

#[derive(Debug,Clone)]
pub struct StoredEndpointID {
    endpoint: EndpointId,
    parent: Option<PublicKey>,
    status: EndpointStatus,
    created: u64,
}

impl StoredEndpointID {
    pub fn new(endpoint: EndpointId) -> Self {
        Self {
            endpoint,
            parent: None,
            status: EndpointStatus::Active,
            created: 0,
        }
    }

    pub async fn save(&self, conn: &Connection) -> Result<()> {
        info!("Save to db {:#?}",&self);
        let ep = self.endpoint.to_string();
        let status = self.status.clone() as u32;

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let created = since_the_epoch.as_secs();
        
        let _ = conn
            .query(
                "insert into endpoints (endpoint,status,created) values (?,?,?);",
                params![ep,status,created],
            )
            .await.unwrap();
        // while let Some(row) = results.next().await? {
        //     println!("{:?}", row);
        // }
        Ok(())
    }
}

const STORE_ENDPOINT: &str = "
CREATE TABLE IF NOT EXISTS endpoints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    endpoint TEXT NOT NULL UNIQUE,
    parent TEXT,
    status INTEGER,
    created INTEGER
) STRICT ";

const EP: &str = "342dca9a6a93192cd19ecb1a190cf6b68202cd0d2a81236cebd28a094b314af7";

// Database setup

#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    pub async fn new(path: PathBuf) -> Result<Self> {
        info!("Create database");
        let path_str = path.display().to_string();

        let db = Builder::new_local(&path_str).build().await?;
        let conn = db.connect()?;
        conn.execute(STORE_ENDPOINT, ()).await?;

        let mut results = conn.query("select * from endpoints", ()).await?;

        let ep = StoredEndpointID::new(EndpointId::from_str(EP)?);
        println!("EP {:#?}", &ep);
        ep.save(&conn).await?;

        while let Some(row) = results.next().await? {
            println!("{:?}", row);
        }

        Ok(Self { conn })
    }
}
