use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use chrono::Utc;
use geekorm::{Connection, ConnectionManager, prelude::*};
use iroh::EndpointId;
use tracing::info;

use super::users::Users;

// Database structs

#[derive(Data, Debug, Default, Clone)]
#[repr(u32)]
pub enum EndpointStatus {
    #[default]
    Seen,
    Known,
    Apparent,
    Fren,
    Enemy,
    DestroyOnSight,
}

#[derive(Table, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredEndpointID {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKeyInteger,
    #[geekorm(unique)]
    pub endpoint: String,
    pub parent: String,
    pub status: EndpointStatus,
    pub rcan: Option<String>,
    pub created: u64,
}

// Test endpoint id.
pub const EP: &str = "342dca9a6a93192cd19ecb1a190cf6b68202cd0d2a81236cebd28a094b314af7";

// Database setup

pub struct Database {
    manager: ConnectionManager,
}

impl<'a> Database {
    pub async fn new(path: PathBuf) -> Result<Self> {
        info!("Create database");
        let manager = ConnectionManager::connect(path.to_string_lossy()).await?;
        {
            let conn = manager.acquire().await;
            Users::create_table(&conn).await?;
            StoredEndpointID::create_table(&conn).await?;
        };
        Ok(Self { manager })
    }

    pub async fn new_mem() -> Result<Self> {
        info!("Create database in memory");
        let manager = ConnectionManager::connect("::memory::").await?;
        {
            let conn = manager.acquire().await;
            Users::create_table(&conn).await?;
            StoredEndpointID::create_table(&conn).await?;
        };
        Ok(Self { manager })
    }

    pub async fn conn(&self) -> Result<Connection<'_>> {
        let conn = self.manager.acquire().await;
        Ok(conn)
    }

    pub async fn test(&self) -> Result<()> {
        info!("start");
        let conn = self.conn().await?;
        let now = Utc::now().timestamp() as u64;
        let ep = EndpointId::from_str(EP)?;
        info!("make");

        let mut stp =
            StoredEndpointID::new(ep.to_string(), EP.to_string(), EndpointStatus::Seen, now);

        stp.save(&conn).await?;
        info!("write");

        // let mut u = Users::new("bob", "bob@bob.com", UserType::User);
        // print!("{:#?}", &u);
        // let r = u.save(&conn).await;
        // println!("{:#?}", &r);
        // info!("finish");

        Ok(())
    }
}
