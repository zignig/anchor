// Chain of smcans

// This is part of the extension from Rcan.

use std::{fmt::Display, time::SystemTime};

use anyhow::{anyhow, Result};
use blake3::Hash;
use ed25519_dalek::VerifyingKey;
use serde::{self, Deserialize, Serialize};

use crate::Smcan;

// List of the errors
#[derive(Debug)]
pub enum ChainError {
    Empty,
    VerifyError(String),
    IssuerMismatch
}

impl Display for ChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainError::Empty => write!(f, "Empty Chain"),
            ChainError::VerifyError(_) => todo!(),
            ChainError::IssuerMismatch => write!(f,"Issuer Mismatch"),
        }
    }
}

//  This serves as the top of the chain
pub struct Anchor {}

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
    C: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Default,
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

    pub fn check(&self, root: VerifyingKey, kind: Hash) -> Result<()> {
        // let bail = false;
        let now = SystemTime::now();
        let chain_length = self.items.len();
        if chain_length == 0 {
            return Err(anyhow!(ChainError::Empty));
        }
        println!("Verify the signatures");
        for (i, j) in self.items.iter().enumerate() {
            println!("{}", i);
            match j.verify_signature() {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }

        println!("Check the kinds");
        for item in &self.items  { 
            if item.get_kind() != &kind { 
                return Err(anyhow!("bad kind"))
            }
        }

        let mut current_issuer = root.clone();
        println!("{:#?}",current_issuer);
        for item in &self.items { 
            let issuer = item.issuer().clone();
            let audience = item.audience().clone();
            println!("{:#?} -- {:#?}",hex::encode(&issuer),hex::encode(&current_issuer));
            if issuer != current_issuer {
                return Err(anyhow!(ChainError::IssuerMismatch));
            }
            
        }

        println!("Chain Length {}", chain_length);
        for item in &self.items {
            println!("{:#?}", item.issuer())
        }
        Ok(())
    }
}
