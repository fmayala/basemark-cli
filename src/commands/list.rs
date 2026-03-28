use anyhow::Result;

use crate::client::BasemarkClient;
use crate::config::Config;
use crate::output;

pub async fn run(config: &Config, collection_id: Option<&str>, pretty: bool) -> Result<()> {
    let client = BasemarkClient::new(config.url()?, config.token()?);
    let docs = client.list_documents(collection_id).await?;
    output::print_json(&docs, pretty)
}
