//! Prints the available TypeSafe models.
//!
//! Run with:
//! ```text
//! TYPESAFE_API_KEY=... cargo run --example listmodels
//! ```

#[tokio::main]
async fn main() -> typesafe::Result<()> {
    let client = typesafe::Client::from_env()?; // reads TYPESAFE_API_KEY

    let models = client.list_models().await?;
    for m in &models {
        println!("{}\t{}\t{}", m.name, m.release_date, m.description);
    }

    Ok(())
}
