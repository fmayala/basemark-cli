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
    id: &str,
    title: Option<&str>,
    pretty: bool,
) -> Result<()> {
    let client = BasemarkClient::new(config.url()?, config.token()?);

    let stdin_content = read_stdin();
    let content = stdin_content.as_deref();

    let doc = client.update_document(id, title, content).await?;
    output::print_json(&doc, pretty)
}
