use anyhow::Result;

use crate::client::BasemarkClient;
use crate::config::Config;
use crate::convert;
use crate::output;

pub async fn run(config: &Config, id: &str, json: bool, pretty: bool) -> Result<()> {
    let client = BasemarkClient::new(config.url()?, config.token()?);

    if json {
        let doc = client.read_document(id).await?;
        output::print_json(&doc, pretty)
    } else {
        // Try the markdown export endpoint first; if it fails or returns
        // what looks like Tiptap JSON, convert it to markdown.
        match client.read_document_markdown(id).await {
            Ok(text) => {
                // If the response parses as a Tiptap JSON doc, convert it.
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                    if parsed["type"].as_str() == Some("doc") {
                        print!("{}", convert::tiptap_json_to_markdown(&parsed));
                        return Ok(());
                    }
                }
                print!("{}", text);
                Ok(())
            }
            Err(_) => {
                // Markdown endpoint unavailable; fall back to the JSON
                // endpoint and convert the content field.
                let doc = client.read_document(id).await?;
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&doc.content) {
                    if parsed["type"].as_str() == Some("doc") {
                        print!("{}", convert::tiptap_json_to_markdown(&parsed));
                        return Ok(());
                    }
                }
                // Content is already plain text / markdown.
                print!("{}", doc.content);
                Ok(())
            }
        }
    }
}
