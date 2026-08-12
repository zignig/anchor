use anyhow::Result;
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::prelude::*;

mod database;
mod settings;

use settings::Setup;
use database::Database;
use tracing::info; 

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
    // println!("database path {:#?}", c2.database_path());

    // let d = database::Database::new(c2.database_path()).await?.
    let d = Database::new(c2.database_path()).await?;
    println!("{:#?}",d);
    Ok(())
}