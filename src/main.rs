use std::str::FromStr;

use anyhow::Result;
use iroh::PublicKey;
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::prelude::*;

mod database;
mod settings;

use database::IdentityApi;

use settings::Setup;
use tracing::info;

// Test endpoint id.
pub const EP: &str = "342dca9a6a93192cd19ecb1a190cf6b68202cd0d2a81236cebd28a094b314af7";


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
    println!("{:#?}", c2);
    
    let id_server = IdentityApi::new(Some(c2.database_path())).await;

    let cl = id_server.client();

    let e =  cl.get(PublicKey::from_str(EP)?).await?;
    println!("GETTER {:#?}",e);

    // let _ = tokio::signal::ctrl_c().await;
    info!("Finish");
    Ok(())
}
