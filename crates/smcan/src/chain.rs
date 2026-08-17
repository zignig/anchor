// Chain of smcans

// This is part of the extension from Rcan.

use std::{
    fmt::Display,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use anyhow::{anyhow, Result};
use blake3::Hash;
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{self, Deserialize, Serialize};

use crate::{Expires, Smcan};

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
            ChainError::Expired(num) => write!(f, "Item {} Expired",num),
            ChainError::IssuerMismatch => write!(f, "Issuer Mismatch"),
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

    pub fn len(&self) -> usize {
        self.items.len()
    }

    // Chech the rcans
    pub fn check(&self, root: VerifyingKey, kind: Hash) -> Result<()> {
        // let bail = false;
        let mut hashes = Vec::<Hash>::new();
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
            hashes.push(blake3::hash(&j.encode()));
        }

        // println!("HASHES {:#?}", hashes);
        println!("Check the kinds");
        for item in &self.items {
            // println!("{} {}", &kind, &item.get_kind());
            if item.get_kind() != &kind {
                return Err(anyhow!("bad kind"));
            }
        }

        println!("Check Expiry");
        for (num,item ) in self.items.iter().enumerate() {
            let expiry = &item.payload.valid_until;
            if !expiry.is_valid_at(now) {
                return Err(anyhow!(ChainError::Expired(num)));
            }
        }

        println!("Check this issuing chain");
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
        println!("Check the hash chain");
        let mut current_hash: Hash = hashes.first().expect("Hash missing").clone();
        for item in &self.items { 
            if let Some(hash)  = item.issuer_hash(){
                println!("HASH {:#?}",hash);
            } else { 
                println!("deal with issue at the front");
                
                // todo!("deal with issue at the front");
            }
        }

        Ok(())
    }
}

/// Chain Builder
#[derive(Debug)]
pub struct ChainBuilder<C>
where
    C: Serialize,
{
    signkey: SigningKey,
    kind: Hash,
    chain: Chain<C>,
    hashes: Vec<Hash>,
}

impl<C> ChainBuilder<C>
where
    C: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Default,
{
    pub fn new(signkey: SigningKey, kind: Hash) -> Self {
        Self {
            signkey,
            kind,
            chain: Chain::<C>::default(),
            hashes: Vec::default(),
        }
    }

    pub fn start(&mut self, target: VerifyingKey, cap: C, dur: Duration) -> Result<()> {
        let rc = Smcan::<C>::issuing_builder(&self.signkey, target, self.kind, cap);
        let smc = rc.sign(Expires::valid_for(dur));
        let h = blake3::hash(&smc.encode());
        self.hashes.push(h);
        self.chain.items.push(smc);

        Ok(())
    }

    pub fn append(&mut self, target: VerifyingKey, cap: C, dur: Duration) -> Result<()> {
        if self.chain.len() < 1 {
            return Err(anyhow!("chain not long enough"));
        };
        let h = self.hashes.last().expect("no hashes");
        let vkey = self.signkey.verifying_key();
        let rc =
            Smcan::<C>::delegating_builder(&self.signkey, target, vkey, self.kind, h.clone(), cap);
        let smc = rc.sign(Expires::valid_for(dur));
        let h = blake3::hash(&smc.encode());
        self.hashes.push(h);
        self.chain.items.push(smc);
        Ok(())
    }

    pub fn check(&self) -> Result<()> {
        let vkey = self.signkey.verifying_key();
        self.chain.check(vkey, self.kind)?;
        Ok(())
    }

    pub fn dump(&self) -> Vec<u8> {
        self.chain.dump()
    }

    pub fn show(&self) {
        self.chain.scan();
    }

    pub fn pop(&mut self) { 
        let _ = self.chain.items.pop();
    }

    pub fn load(&mut self, path: PathBuf) -> Result<()> {
        match std::fs::read(path) {
            Ok(content) => {
                let chain = Chain::<C>::load(content)?;
                self.chain = chain
            }
            Err(e) => {
                return Err(anyhow!("Bad Data Parse {:#?}", e));
            }
        };
        // Extract the hashes on load
        for i in &self.chain.items{ 
            self.hashes.push(blake3::hash(&i.encode()));
        }
        Ok(())
    }
}
