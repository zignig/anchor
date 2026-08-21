//! This takes a chain and provides an interface to
//! check and query the integrity and values of a chain

use serde::{Deserialize, Serialize};

use crate::{Capability, Chain};

pub struct Anchor {

}

#[derive(Debug)]
pub struct Resolver<C>
where
    C: Serialize,
{
    chain: Chain<C>,
}

impl<C> Resolver<C>
where
    C: Serialize + Capability + for<'de> Deserialize<'de> + std::fmt::Debug + Default,
{
    pub fn new() -> Self {
        Self {
            chain: Chain::<C>::default(),
        }
    }

    
}
