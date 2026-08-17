// A cache of the rcan authenticated endpoints.

use std::path::PathBuf;
use std::time::SystemTime;

use iroh::EndpointId;
use irpc::{Client, WithChannels, channel::oneshot, rpc_requests};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use tracing::debug;
use tracing::info;

use crate::database::Store;
use super::{Fren,caps::Caps};
use anyhow::Result;


use smcan::Smcan;

// use rcan::Capability;
// use rcan::Rcan;
// use crate::capset::Caps;
// use crate::id_db::PersistStore;

#[rpc_requests(message = IdentityMessage, no_rpc, no_spans)]
#[derive(Serialize, Deserialize, Debug)]
enum StorageProtocol {
    #[rpc(tx=oneshot::Sender<Option<Fren>>)]
    Get(Get),
    #[rpc(tx=oneshot::Sender<()>)]
    Remove(Remove),
    #[rpc(tx=oneshot::Sender<()>)]
    Set(Set),
    #[rpc(tx=oneshot::Sender<bool>)]
    Check(Check),
    #[rpc(tx=oneshot::Sender<Vec<Fren>>)]
    List(List),
}

// Irpc constructs

#[derive(Debug, Serialize, Deserialize)]
struct Get {
    key: EndpointId,
}

#[derive(Debug, Serialize, Deserialize)]
struct Remove {
    key: EndpointId,
}

#[derive(Debug, Serialize, Deserialize)]
struct Set {
    key: EndpointId,
    value: Fren,
}

#[derive(Debug, Serialize, Deserialize)]
struct Check {
    key: EndpointId,
}

#[derive(Debug, Serialize, Deserialize)]
struct List;

// impl From<(EndpointId, Fren)> for Set {
//     fn from((key, value): (EndpointId, Fren)) -> Self {
//         Self { key, value }
//     }
// }

struct Actor {
    recv: tokio::sync::mpsc::Receiver<IdentityMessage>,
    store: Store,
}

impl Actor {
    async fn run(mut self) {
        while let Some(msg) = self.recv.recv().await {
            self.handle(msg).await;
        }
    }

    async fn handle(&mut self, msg: IdentityMessage) -> Result<()> {
        match msg {
            IdentityMessage::Get(get) => {
                let WithChannels { tx, inner, .. } = get;
                let value = match self.store.get(&inner.key).await? {
                    Some(value) => Some(value.clone()),
                    None => None,
                };
                tx.send(value).await.ok();
            }

            IdentityMessage::Set(set) => {
                let WithChannels { tx, inner, .. } = set;
                let _ = self.store.add(inner.key.clone()).await;
                tx.send(()).await.ok();
            }

            IdentityMessage::Remove(remove) => {
                let WithChannels { tx, inner, .. } = remove;
                self.store.remove(&inner.key).await?;
                tx.send(()).await.ok();
            }

            IdentityMessage::Check(check) => {
                let WithChannels { tx, inner, .. } = check;
                let is_good = match self.store.get(&inner.key).await? {
                    Some(fren) => {
                        let _ = self.store.add(inner.key.clone()).await;
                        // Check to see if the rbac is still valid
                        let mut status = false;
                        if let Some(rcan) = fren.rcan.clone() {
                            let time = SystemTime::now();
                            if rcan.expires().is_valid_at(time) {
                                status = true;
                            } else {
                                status = false;
                            }
                        }
                        status
                    }
                    None => false,
                };
                tx.send(is_good).await.ok();
            }

            IdentityMessage::List(list) => {
                let WithChannels { tx, .. } = list;
                let mut res: Vec<Fren> = Vec::new();
                // for item in self.store.iter() {
                //     let (_, item) = item;
                //     res.push(item.clone());
                // }
                tx.send(res).await.ok();
            }
        }
        Ok(())
    }
}

pub struct IdentityApi {
    tx: Sender<IdentityMessage>,
}

impl IdentityApi {
    pub async fn new(db_path: Option<PathBuf>) -> IdentityApi {
        info!("make new ID");
        let (tx, rx) = tokio::sync::mpsc::channel(5);
        let store = match db_path {
            Some(path) => Store::new(path).await.unwrap(),
            None => Store::new_mem().await.unwrap(),
        };
        let actor = Actor {
            recv: rx,
            store: store,
        };
        n0_future::task::spawn(actor.run());
        info!("Running");
        IdentityApi { tx: tx.clone() }
    }

    pub fn client(&self) -> IdClient {
        let tx = self.tx.clone();
        IdClient {
            inner: Client::local(tx),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IdClient {
    inner: Client<StorageProtocol>,
}

impl IdClient {

    pub async fn get(&self, key: EndpointId) -> irpc::Result<Option<Fren>> {
        self.inner.rpc(Get { key }).await
    }

    pub async fn new_fren(&self, key: EndpointId, rcan: Smcan<Caps>) {
        // info!("new fren {:#?} -- {:#?}",key,rcan);
        match self.inner.rpc(Get { key }).await.unwrap() {
            Some(fren) => {
                info!("existing fren => {:}", fren.id.fmt_short());
                return;
            }
            None => {
                info!("make a new fren {}", key.fmt_short());
                let mut value = Fren::new(key);
                value.rcan = Some(rcan);
                self.inner.rpc(Set { key, value }).await.unwrap();
            }
        }
    }

    pub async fn check(&self, key: EndpointId) -> irpc::Result<bool> {
        self.inner.rpc(Check { key }).await
    }

    pub async fn set(&self, key: EndpointId, value: Fren) -> irpc::Result<()> {
        self.inner.rpc(Set { key, value }).await
    }

    pub async fn remove(&self, key: EndpointId) -> irpc::Result<()> {
        self.inner.rpc(Remove { key }).await
    }

    pub async fn list(&self) -> irpc::Result<Vec<Fren>> {
        info!("List");
        self.inner.rpc(List {}).await
    }
}
