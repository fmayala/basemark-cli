use anyhow::Result;
use std::io::Read;

use crate::client::BasemarkClient;
use crate::config::Config;
use crate::output;

fn read_stdin() -> Option<String> {
    if atty::is(atty::Stream::Stdin) {
        return None;
    }
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).ok()?;
    if buf.is_empty() {
        None
    } else {
        Some(buf)
    }
}

pub async fn run(
    config: &Config,
    title: &str,
    collection_id: Option<&str>,
    content: Option<&str>,
    pretty: bool,
) -> Result<()> {
    let client = BasemarkClient::new(config.url()?, config.token()?);

    // Use explicit --content if provided, otherwise try stdin.
    let stdin_content = if content.is_some() {
        None
    } else {
        read_stdin()
    };
    let final_content = content.or(stdin_content.as_deref());

    let doc = client
        .create_document(title, final_content, collection_id)
        .await?;
    output::print_json(&doc, pretty)
}
