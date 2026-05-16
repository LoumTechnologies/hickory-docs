use anyhow::Result;
use blue_iron_cube::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
