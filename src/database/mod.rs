// Database setup for auth and user bits
// Tables
//    endpoint
//    publisher

use std::{
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use geekorm::ConnectionManager;
use geekorm::{Connection, prelude::*};

use anyhow::Result;
use iroh::{EndpointId, PublicKey};
use tracing::info;
// use turso::{Builder, Connection, params};

// Database structs
// Could be an orm , but I prefer raw sql

#[derive(Debug,Default, Clone,serde::Serialize, serde::Deserialize)]
#[repr(u32)]
pub enum EndpointStatus {
    Active,
}

#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StoredEndpointID {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKeyInteger,
    endpoint: EndpointId,
    parent: Option<PublicKey>,
    status: EndpointStatus,
    created: u64,
}

// impl StoredEndpointID { 

// }

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

const EP: &str = "342dca9a6a93192cd19ecb1a190cf6b68202cd0d2a81236cebd28a094b314af7";

// Database setup

#[derive(Debug)]
pub struct Database {
    // conn: Connection,
}

impl Database {
    pub async fn new(path: PathBuf) -> Result<Self> {
        info!("Create database");
        let manager = ConnectionManager::connect(path.to_string_lossy()).await?;
        let m2 = ConnectionManager::path(path).await;
        let conn = manager.acquire().await;
        Users::create_table(&conn).await?;
        // StoredEndpointID::create_table(&conn).await;

        let mut u = Users::new("bob", "bob@bob.com", UserType::Admin);
        print!("{:#?}", &u);
        u.save(&conn).await?;
        Ok(Self {})
    }
}
