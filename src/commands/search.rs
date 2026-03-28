use anyhow::Result;

use crate::client::BasemarkClient;
use crate::config::Config;
use crate::output;

pub async fn run(config: &Config, query: &str, pretty: bool) -> Result<()> {
    let client = BasemarkClient::new(config.url()?, config.token()?);
    let results = client.search(query).await?;
    output::print_json(&results, pretty)
}
