use anyhow::Result;

use crate::client::BasemarkClient;
use crate::config::Config;

pub async fn run(config: &Config, id: &str, force: bool) -> Result<()> {
    if !force {
        eprintln!("Are you sure? Use --force to confirm");
        return Ok(());
    }

    let client = BasemarkClient::new(config.url()?, config.token()?);
    client.delete_document(id).await?;
    eprintln!("Deleted document {id}");
    Ok(())
}
