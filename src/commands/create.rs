use anyhow::Result;
use std::io::Read;

use crate::client::BasemarkClient;
use crate::config::Config;
use crate::convert;
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

    // Convert markdown content to Tiptap JSON before sending to the API.
    let tiptap_string = final_content.map(|md| {
        let tiptap_json = convert::markdown_to_tiptap_json(md);
        serde_json::to_string(&tiptap_json).expect("failed to serialize Tiptap JSON")
    });

    let doc = client
        .create_document(title, tiptap_string.as_deref(), collection_id)
        .await?;
    output::print_json(&doc, pretty)
}
