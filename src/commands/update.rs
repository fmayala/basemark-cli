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
    id: &str,
    title: Option<&str>,
    pretty: bool,
) -> Result<()> {
    let client = BasemarkClient::new(config.url()?, config.token()?);

    let stdin_content = read_stdin();

    // Convert markdown content from stdin to Tiptap JSON before sending.
    let tiptap_string = stdin_content.as_deref().map(|md| {
        let tiptap_json = convert::markdown_to_tiptap_json(md);
        serde_json::to_string(&tiptap_json).expect("failed to serialize Tiptap JSON")
    });

    let doc = client
        .update_document(id, title, tiptap_string.as_deref())
        .await?;
    output::print_json(&doc, pretty)
}
