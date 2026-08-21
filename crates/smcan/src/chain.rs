//! Chain of smcans
//! Import, show and check and export a vector of CAN objects

use std::{fmt::Display, time::SystemTime};

use anyhow::{anyhow, Result};
use blake3::Hash;
use ed25519_dalek::VerifyingKey;
use serde::{self, Deserialize, Serialize};
use tracing::debug;

use crate::{Capability, Smcan};

// List of the errors
#[derive(Debug)]
pub enum ChainError {
    Empty,
    VerifyError(String),
    Expired(usize),
    IssuerMismatch,
}

impl Display for ChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainError::Empty => write!(f, "Empty Chain"),
            ChainError::VerifyError(_) => todo!(),
            ChainError::Expired(num) => write!(f, "Item {} Expired", num),
            ChainError::IssuerMismatch => write!(f, "Issuer Mismatch"),
        }
    }
}

//  This serves as the top of the chain

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Chain<C>
where
    C: Serialize,
{
    #[serde[default]]
    pub items: Vec<Smcan<C>>,
}

impl<C> Chain<C>
where
    C: Serialize + Capability + for<'de> Deserialize<'de> + std::fmt::Debug + Default,
{
    pub fn new(items: Vec<Smcan<C>>) -> Chain<C> {
        Self { items }
    }

    pub fn scan(&self) {
        for (i, j) in self.items.iter().enumerate() {
            println!("{:#?} -> {:#?}", i, j);
        }
    }

    pub fn dump(&self) -> Vec<u8> {
        let val = postcard::to_allocvec(self).unwrap();
        val
    }

    pub fn load(data: Vec<u8>) -> Result<Self> {
        let val: Chain<C> = postcard::from_bytes(&data)?;
        Ok(val)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    // Chech the rcans
    pub fn check(&self, root: VerifyingKey) -> Result<()> {
        // let bail = false;
        let kind_hash = blake3::hash(C::KIND.as_bytes());
        let mut hashes = Vec::<Hash>::new();
        let now = SystemTime::now();
        let chain_length = self.items.len();
        if chain_length == 0 {
            return Err(anyhow!(ChainError::Empty));
        }

        debug!("Verify the signatures");
        for (i, j) in self.items.iter().enumerate() {
            println!("{}", i);
            match j.verify_signature() {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
            hashes.push(blake3::hash(&j.encode()));
        }

        // println!("HASHES {:#?}", hashes);
        debug!("Check the kinds");
        for item in &self.items {
            // println!("{} {}", &kind, &item.get_kind());
            if item.get_kind() != &kind_hash {
                return Err(anyhow!("bad kind"));
            }
        }

        debug!("Check Expiry");
        for (num, item) in self.items.iter().enumerate() {
            let expiry = &item.payload.valid_until;
            if !expiry.is_valid_at(now) {
                return Err(anyhow!(ChainError::Expired(num)));
            }
        }

        debug!("Check this issuing chain");
        let mut current_issuer = root.clone();
        for item in &self.items {
            // println!("{:#?}", item);
            let issuer = item.issuer().clone();
            let audience = item.audience().clone();
            // println!(
            //     "{:#?} -- {:#?}",
            //     hex::encode(&issuer),
            //     hex::encode(&audience)
            // );
            if issuer != current_issuer {
                return Err(anyhow!(ChainError::IssuerMismatch));
            }
            current_issuer = audience;
        }

        debug!("Check the hash chain");
        let mut current_hash: Hash = hashes.first().expect("Hash missing").clone();
        for item in &self.items {
            if let Some(hash) = item.issuer_hash() {
                println!("HASH {:#?}", hash);
            } else {
                println!("deal with issue at the front");

                // todo!("deal with issue at the front");
            }
        }

        Ok(())
    }
}
