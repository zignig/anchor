// Database setup for auth and user bits

use std::path::PathBuf;

use turso::{Builder, Connection};
use anyhow::Result;

#[derive(Debug)]
pub struct Database{ 
    conn : Connection
}

impl Database { 
    pub async fn new(path: PathBuf) -> Result<Self> { 
        let path_str = path.display().to_string();
        let db = Builder::new_local(&path_str).build().await?;
        
        let conn = db.connect()?;

        
        Ok(Self { 
            conn
        })
    }
}
