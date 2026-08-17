use std::{
    path::PathBuf,
    str::FromStr
};

use chrono::Utc;
use geekorm::ConnectionManager;
use geekorm::{Connection, prelude::*};

use anyhow::Result;
use iroh::EndpointId;
use tracing::info;

// Database structs

#[derive(Data, Debug, Default, Clone)]
#[repr(u32)]
pub enum EndpointStatus {
    #[default]
    New,
    Active,
}

#[derive(Table, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredEndpointID {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKeyInteger,
    endpoint: String,
    parent: String,
    status: EndpointStatus,
    rcan: Option<String>,
    created: u64,
}


#[derive(Data, Debug, Clone, Default)]
enum UserType {
    Admin,
    #[default]
    User,
    Guest,
}

#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Users {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKeyInteger,

    #[geekorm(unique)]
    username: String,

    #[geekorm(unique)]
    email: String,

    user_type: UserType,

    #[geekorm(new = false)]
    active: bool,
    postcode: Option<String>,
}

pub const EP: &str = "342dca9a6a93192cd19ecb1a190cf6b68202cd0d2a81236cebd28a094b314af7";

// Database setup

pub struct Database {
    manager: ConnectionManager,
}

impl Database {
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

    pub async fn conn(&self) -> Result<Connection<'_>> {
        let conn = self.manager.acquire().await;
        Ok(conn)
    }

    pub async fn test(&self) -> Result<()> {
        let conn = self.conn().await?;
        let now = Utc::now().timestamp() as u64;
        let ep = EndpointId::from_str(EP)?;

        let mut stp =
        StoredEndpointID::new(ep.to_string(), EP.to_string(), EndpointStatus::Active, now);

        stp.save(&conn).await?;

        let mut u = Users::new("bob", "bob@bob.com", UserType::Admin);
        print!("{:#?}", &u);
        let r = u.save(&conn).await;
        println!("{:#?}", &r);
        Ok(())
    }


}
