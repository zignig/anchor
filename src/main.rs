use anyhow::Result;

mod database;
mod settings;

use settings::Setup;
use database::Database;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let c2 = Setup::open()?;
    println!("{:#?}", c2);
    // println!("database path {:#?}", c2.database_path());

    // let d = database::Database::new(c2.database_path()).await?.
    let d = Database::new(c2.database_path()).await?;
    println!("{:#?}",d);
    Ok(())
}
