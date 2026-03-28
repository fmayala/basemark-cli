use anyhow::Result;

use crate::client::BasemarkClient;
use crate::config::Config;
use crate::output;

pub async fn run(config: &Config, id: &str, json: bool, pretty: bool) -> Result<()> {
    let client = BasemarkClient::new(config.url()?, config.token()?);

    if json {
        let doc = client.read_document(id).await?;
        output::print_json(&doc, pretty)
    } else {
        let markdown = client.read_document_markdown(id).await?;
        print!("{}", markdown);
        Ok(())
    }
}
