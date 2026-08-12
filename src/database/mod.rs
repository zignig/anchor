// Database setup for auth and user bits
// Tables
//    endpoint
//    publisher

use std::path::PathBuf;

use iroh::EndpointId;
use tracing::info;
use turso::{Builder, Connection};
use anyhow::Result;


// Database structs
// Could be an orm , but I prefer raw sql

pub struct StoredEndpointID { 
    endpoint: EndpointId,
}

const STORE_ENDPOINT: &str = "
CREATE TABLE IF NOT EXISTS endpoints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    endpoint TEXT NOT NULL
) STRICT ";

#[derive(Debug)]
pub struct Database{ 
    conn : Connection
}

impl Database { 
    pub async fn new(path: PathBuf) -> Result<Self> { 
        let path_str = path.display().to_string();

        let db = Builder::new_local(&path_str).build().await?;
        let conn = db.connect()?;
        conn.execute(STORE_ENDPOINT, ()).await?;

        let mut results = conn.query("select * from endpoints",()).await?;

 
        while let Some(row) = results.next().await? { 
            println!("{:?}",row);
        }

        Ok(Self { 
            conn
        })
    }
}


