use std::str::FromStr;

use anyhow::Result;
use blake3::Hash;
use iroh::PublicKey;
use std::time::Duration;

use smcan::{Chain, Smcan};
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::prelude::*;

mod caps;
mod idstore;
mod settings;

use idstore::IdentityApi;

use settings::Setup;
use tracing::info;

use crate::caps::Caps;

// Test endpoint id.
pub const EP: &str = "342dca9a6a93192cd19ecb1a190cf6b68202cd0d2a81236cebd28a094b314af7";
pub const EP2: &str = "c3ed43570bef3014a3583dfc4088a9eee9698579bd6527424b017c9ba735237e";

pub type Canner = Smcan<Caps>;
pub type Chainer = Chain<Caps>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut filter = Targets::new();
    filter = filter
        .with_target(env!("CARGO_PKG_NAME"), LevelFilter::DEBUG)
        .with_target("", LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let c2 = Setup::new("zignig".to_string())?;
    // println!("{:#?}", c2);

    let _ = IdentityApi::new(Some(c2.database_path())).await;

    // let cl = id_server.client();

    // let e =  cl.get(PublicKey::from_str(EP)?).await?;
    // println!("GETTER {:#?}",e);

    // let e =  cl.get(PublicKey::from_str(EP2)?).await?;
    // println!("GETTER Fail {:#?}",e);

    // let _ = cl.list().await?;

    let r = c2.get_author_secret()?;
    let issuer = smcan::SigningKey::from_bytes(&r.to_bytes());

    let audience = issuer.verifying_key();

    let kind = blake3::hash(b"test kind");
    let capability = Caps::All;

    let rcb = Canner::issuing_builder(&issuer, audience, kind, capability.clone());
    let v = rcb.sign(smcan::Expires::valid_for(Duration::from_secs(60)));
    println!("CAPS {:#?}", v);

    let crunch = blake3::hash(&v.encode());
    println!("crunch {:#?}", crunch);
    let ep = PublicKey::from_str(EP)?;
    let target = smcan::VerifyingKey::from_bytes(ep.as_bytes())?;
    let cap2 = Caps::Issue;
    let rcb2 = Canner::delegating_terminal(
        &issuer,
        target,
        issuer.verifying_key(),
        kind,
        crunch,
        cap2,
    );

    let v2 =  rcb2.sign(smcan::Expires::valid_for(Duration::from_secs(24 * 60 * 60)));
    println!("CAPS2 {:#?}",v2);
    println!("CAPS2 {} bytes long",v2.encode().len());
    if let Some(h) = v2.issuer_hash() { 
        assert!(&h ==  &crunch);
        info!("HASH IS AWESOME");
    }
    let rv = vec![v,v2];
    let ch = Chainer::new(rv);
    // println!("{:#?}",ch);
    ch.check(audience,kind)?;
    let data = ch.dump();
    let _out = Chainer::load(data);
    // println!("{:#?}",out);
    info!("Finish");

    Ok(())
}
