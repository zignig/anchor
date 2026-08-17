// Database setup for auth and user bits

mod caps;
mod db;
mod irpc;
mod users;

use std::path::PathBuf;

pub use db::Database;
pub use irpc::IdentityApi;

use anyhow::Result;
use iroh::PublicKey;
use serde::{Deserialize, Serialize};
use smcan::Smcan;

use crate::database::caps::Caps;

use tracing::info;

// Expand
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fren {
    name: String,
    id: PublicKey,
    rcan: Option<Smcan<Caps>>,
}

impl Fren {
    pub fn new(id: PublicKey) -> Self {
        Self {
            name: "".to_string(),
            id: id,
            rcan: None,
        }
    }
}

pub struct Store {
    database: Database,
}

impl Store {
    pub async fn new(path: PathBuf) -> Result<Self> {
        let database = db::Database::new(path).await?;
        {
            database.test().await?;
        }
        Ok(Self { database })
    }

    pub async fn new_mem() -> Result<Self> {
        let database = db::Database::new_mem().await?;
        Ok(Self { database })
    }

    pub async fn get(&self, key: &iroh::PublicKey) -> Result<Option<Fren>> {
        Ok(None)
    }

    pub async fn add(&self, key: iroh::PublicKey) -> Result<Option<Fren>> {
        info!("add {:#?}", key);
        Ok(None)
    }

    pub async fn remove(&self, key: &iroh::PublicKey) -> Result<Option<Fren>> {
        info!("remove {:#?}", key);
        Ok(None)
    }
}
