
use anyhow::Result;

mod settings;

use settings::Setup; 

#[tokio::main]
async fn main() -> Result<()> {
    let c = Setup::new("zignig".to_string())?;
    println!("{:#?}",c);
    Ok(())
}
