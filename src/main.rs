use std::{path::PathBuf, str::FromStr, time::Duration};

use anyhow::Result;
use blake3::Hash;
use iroh::PublicKey;
use smcan::{Chain, ChainBuilder, Resolver, Smcan};
use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    prelude::*,
};

mod caps;
mod idstore;
mod settings;

use idstore::IdentityApi;
use settings::Setup;
use tracing::{error, info};

use crate::caps::Caps;

// Test endpoint id.
pub const EP: &str = "342dca9a6a93192cd19ecb1a190cf6b68202cd0d2a81236cebd28a094b314af7";
pub const EP2: &str = "c3ed43570bef3014a3583dfc4088a9eee9698579bd6527424b017c9ba735237e";

// pub type Chainer = Chainbuilder<Caps>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut filter = Targets::new();
    filter = filter
        .with_target(env!("CARGO_PKG_NAME"), LevelFilter::DEBUG)
        .with_target("smcan", LevelFilter::DEBUG);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let c2 = Setup::new("zignig".to_string())?;
    // println!("{:#?}", c2);

    let _ = IdentityApi::new(Some(c2.database_path())).await;

    let r = c2.get_author_secret()?;

    let issuer = smcan::SigningKey::from_bytes(&r.to_bytes());
    let audience = issuer.verifying_key();

    let ep = PublicKey::from_str(EP)?;
    let target = smcan::VerifyingKey::from_bytes(ep.as_bytes())?;

    info!("CHAIN ISSUER");

    let mut cb = ChainBuilder::<Caps>::new(audience);

    // Load and save a binary file for local testing
    let path = PathBuf::from_str("data.bin").unwrap();
    match cb.load(path) {
        Ok(_) => {
            info!("Load from file");
        }
        Err(_) => {
            //
            cb.start(
                issuer,
                audience,
                Caps::All,
                Duration::from_secs(24 * 60 * 60 * 5),
            )?;
            cb.append(
                audience,
                Caps::PathTest {
                    path: "/".to_string(),
                },
                Duration::from_secs(3000),
            )?;
            cb.append(audience, Caps::Info, Duration::from_secs(4000))?;
            cb.append(target, Caps::Status, Duration::from_secs(1000))?;
            std::fs::write("data.bin", cb.dump()).expect("Can't write data file");
        }
    }
    cb.show();
    println!("data_size {}", cb.dump().len());
    let cb_res = cb.check();
    info!("{:#?}", cb_res);

    info!("RESOLVER TEST");
    let res = Resolver::<Caps>::new();
    println!("{:#?}", res);
    // info!("{:#?}",cb);
    info!("Finish");

    Ok(())
}
