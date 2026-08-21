//! Chain Builder
//! Make a vector of smcan objects
//! Sign build and save.

use std::{collections::HashMap, path::PathBuf, time::Duration};

use anyhow::{anyhow, Result};
use blake3::Hash;
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::{Capability, Chain, Expires, Smcan};

// Limit the length of the chain ( testing for now )
const MAX_LENGTH: usize = 8;

#[derive(Debug)]
pub struct ChainBuilder<C>
where
    C: Serialize,
{
    sourcekey: VerifyingKey,
    signing_keys: HashMap<VerifyingKey, SigningKey>,
    chain: Chain<C>,
    hashes: Vec<Hash>,
}

impl<C> ChainBuilder<C>
where
    C: Serialize + Capability + for<'de> Deserialize<'de> + std::fmt::Debug + Default,
{
    pub fn new(sourcekey: VerifyingKey) -> Self {
        Self {
            sourcekey,
            signing_keys: HashMap::new(),
            chain: Chain::<C>::default(),
            hashes: Vec::default(),
        }
    }

    pub fn add_key(&mut self, key: &SigningKey) -> Result<()> {
        let vkey = key.verifying_key();
        self.signing_keys.insert(vkey, key.clone());
        Ok(())
    }

    // Start a chain
    pub fn start(
        &mut self,
        source: SigningKey,
        target: VerifyingKey,
        cap: C,
        dur: Duration,
    ) -> Result<()> {
        self.add_key(&source)?;
        let rc = Smcan::<C>::issuing_builder(&source, target, cap);
        let smc = rc.sign(Expires::valid_for(dur));
        let h = blake3::hash(&smc.encode());
        self.hashes.push(h);
        self.chain.items.push(smc);

        Ok(())
    }

    pub fn start_terminal(
        &mut self,
        source: SigningKey,
        target: VerifyingKey,
        cap: C,
        dur: Duration,
    ) -> Result<()> {
        self.add_key(&source)?;
        let rc = Smcan::<C>::issuing_terminal(&source, target, cap);
        let smc = rc.sign(Expires::valid_for(dur));
        let h = blake3::hash(&smc.encode());
        self.hashes.push(h);
        self.chain.items.push(smc);

        Ok(())
    }

    // Add a link to the chain
    pub fn append(
        &mut self,
        target: VerifyingKey,
        cap: C,
        dur: Duration,
        terminal: bool,
    ) -> Result<()> {
        // Check the lengths of the chain
        if self.chain.len() < 1 {
            return Err(anyhow!("Chain not long enough"));
        };

        if self.chain.len() > MAX_LENGTH {
            return Err(anyhow!("Chain too long"));
        }

        // Get the last item off the chain , and try to find the SigningKey
        let last_can = self.chain.items.last().expect("no items");
        let last_aud = last_can.audience().to_owned();
        let signkey = match self.signing_keys.get(&last_aud) {
            Some(key) => key,
            None => return Err(anyhow!("No signing key")),
        };

        // Get the has of the last item
        let h = self.hashes.last().expect("no hashes");

        // Build the next smcan
        let rc = if terminal {
            Smcan::<C>::delegating_terminal(&signkey, target, last_aud, h.clone(), cap)
        } else {
            Smcan::<C>::delegating_builder(&signkey, target, last_aud, h.clone(), cap)
        };
        
        let smc = rc.sign(Expires::valid_for(dur));

        // Grab the hash and update the chain.
        let h = blake3::hash(&smc.encode());
        self.hashes.push(h);
        self.chain.items.push(smc);
        Ok(())
    }

    pub fn check(&self) -> Result<()> {
        self.chain.check(self.sourcekey)?;
        Ok(())
    }

    pub fn check_cap(&self, cap: C) -> Result<bool> {
        let val = self.chain.check_cap(self.sourcekey, cap)?;
        Ok(val)
    }

    pub fn dump(&self) -> Vec<u8> {
        self.chain.dump()
    }

    pub fn show(&self) {
        self.chain.show();
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
        for i in &self.chain.items {
            self.hashes.push(blake3::hash(&i.encode()));
        }
        Ok(())
    }
}
