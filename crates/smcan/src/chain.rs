//! Chain of smcans
//! Import, show and check and export a vector of CAN objects

use std::{fmt::Display, time::SystemTime};

use anyhow::{anyhow, Result};
use blake3::Hash;
use ed25519_dalek::VerifyingKey;
use serde::{self, Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::{Capability, Smcan};

// List of the errors
#[derive(Debug)]
pub enum ChainError {
    Empty,
    BadKind(Hash, Hash),
    BadIssuer(VerifyingKey),
    Expired(usize),
    PermissionDeny,
    IssuerMismatch,
    TerminalInChain,
}

impl Display for ChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainError::Empty => write!(f, "Empty Chain"),
            ChainError::BadKind(k1, k2) => write!(f, "Kind should be {}, got {}", k1, k2),
            ChainError::Expired(num) => write!(f, "Item {} Expired", num),
            ChainError::IssuerMismatch => write!(f, "Issuer Mismatch"),
            ChainError::PermissionDeny => write!(f, "permission denied"),
            ChainError::BadIssuer(verifying_key) => write!(f, "Bad issuer , {:?}", verifying_key),
            ChainError::TerminalInChain => write!(f,"Terminal before chain end."),
        }
    }
}

//  This serves as the top of the chain

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
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

    pub fn show(&self) {
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

    // Check the smcans , general hygiene , does not
    // check the capability chain
    pub fn check(&self, root: VerifyingKey) -> Result<()> {
        let kind_hash = blake3::hash(C::KIND.as_bytes());
        let mut hashes = Vec::<Hash>::new();
        let now = SystemTime::now();
        let chain_length = self.items.len();
        if chain_length == 0 {
            return Err(anyhow!(ChainError::Empty));
        }

        debug!("Verify the signatures");
        for (i, j) in self.items.iter().enumerate() {
            debug!(
                "{} {} -> {}",
                i,
                hex::encode(j.issuer()),
                hex::encode(j.audience())
            );

            match j.verify_signature() {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
            hashes.push(blake3::hash(&j.encode()));
        }

        debug!("Check the kinds");
        for item in &self.items {
            // println!("{} {}", &kind_hash, &item.get_kind());
            let kind = item.get_kind();
            if kind != &kind_hash {
                return Err(anyhow!(ChainError::BadKind(kind_hash, *kind)));
            }
        }

        debug!("Check Origin");
        for item in &self.items {
            let issuer = item.capability_issuer();
            if issuer != &root {
                return Err(anyhow!(ChainError::BadIssuer(issuer.clone())));
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
            let issuer = item.issuer().clone();
            let audience = item.audience().clone();
            if issuer != current_issuer {
                return Err(anyhow!(ChainError::IssuerMismatch));
            }
            current_issuer = audience;
        }

        warn!("Check terminations");
        for (num, item) in self.items.iter().enumerate() {
            // there is a terminal entry in the chain, bug out
            if item.is_terminal() && (num < chain_length ) {
                return Err(anyhow!(ChainError::TerminalInChain));
            }
        }

        warn!("Check the hash chain");
        let _current_hash: Hash = hashes.first().expect("Hash missing").clone();
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

    pub fn check_cap(&self, root: VerifyingKey, cap: C) -> Result<bool> {
        // Perform general checks on the chain
        debug!("Check cap {:#?}", &cap);
        self.check(root)?;
        for item in &self.items {
            let current_cap = item.capability();
            info!("{:?} -> {:?}", cap, current_cap);
            if !current_cap.permits(&cap) {
                warn!("Permission Fail");
                return Err(anyhow!(ChainError::PermissionDeny));
            }
        }
        //If it does not bail out before this, it's good
        Ok(true)
    }
}
