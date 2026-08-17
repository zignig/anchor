// Database setup for auth and user bits

mod db;
mod irpc;
mod users;

use std::{path::PathBuf, str::FromStr};

pub use db::Database;
use geekorm::GeekConnector;
pub use irpc::IdentityApi;

use anyhow::Result;
use iroh::PublicKey;
use serde::{Deserialize, Serialize};
use smcan::Smcan;

use crate::idstore::db::StoredEndpointID;
use crate::caps::Caps;


use tracing::info;

// Expand
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fren {
    id: PublicKey,
    rcan: Option<Smcan<Caps>>,
}

impl Fren {
    pub fn new(id: PublicKey) -> Self {
        Self { id: id, rcan: None }
    }
}

// Convert from stored endpoint to fren
impl From<&StoredEndpointID> for Fren {
    fn from(value: &StoredEndpointID) -> Self {
        let pk = PublicKey::from_str(&value.endpoint).unwrap();
        Self { id: pk, rcan: None }
    }
}

impl From<StoredEndpointID> for Fren {
    fn from(value: StoredEndpointID) -> Self {
        let pk = PublicKey::from_str(&value.endpoint).unwrap();
        Self { id: pk, rcan: None }
    }
}

impl Into<StoredEndpointID> for Fren {
    fn into(self) -> StoredEndpointID {
        todo!()
    }
}

pub struct Store {
    database: Database,
}

impl Store {
    pub async fn new(path: PathBuf) -> Result<Self> {
        let database = db::Database::new(path).await?;
        {
            // Add some test data
            //TODO remove.
            let _ = database.test().await;
        }
        Ok(Self { database })
    }

    pub async fn new_mem() -> Result<Self> {
        let database = db::Database::new_mem().await?;
        Ok(Self { database })
    }

    pub async fn get(&self, key: &iroh::PublicKey) -> Result<Option<Fren>> {
        let pk = key.to_string();
        info!("PK {:#?}", pk);
        let connection = self.database.conn().await?;
        let sp = match StoredEndpointID::fetch_by_endpoint(&connection, pk).await {
            Ok(ep) => {
                let f: Fren = ep.into();
                Some(f)
            }
            Err(_) => None,
        };
        // info!("{:#?}", sp);
        Ok(sp)
    }

    pub async fn add(&self, key: iroh::PublicKey) -> Result<Option<Fren>> {
        info!("add {:#?}", key);
        Ok(None)
    }

    pub async fn remove(&self, key: &iroh::PublicKey) -> Result<Option<Fren>> {
        info!("remove {:#?}", key);
        Ok(None)
    }

    pub async fn iter_end(&self) -> Result<Vec<Fren>> {
        let conn = self.database.conn().await?;
        let q = StoredEndpointID::all(&conn).await?;
        let fr: Vec<Fren> = q.iter().map(|sep| sep.into()).collect();
        // for i in &fr {
        //     info!("{:#?}", i);
        // }
        Ok(fr)
    }
}
