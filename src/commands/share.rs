use anyhow::Result;

use crate::client::BasemarkClient;
use crate::config::Config;
use crate::output;

pub async fn run(
    config: &Config,
    id: &str,
    public: bool,
    private: bool,
    invite: Option<&str>,
    show_url: bool,
    pretty: bool,
) -> Result<()> {
    let client = BasemarkClient::new(config.url()?, config.token()?);

    if public {
        let doc = client.update_document_public(id, true).await?;
        eprintln!("Document is now public");
        output::print_json(&doc, pretty)?;
    }
    if private {
        let doc = client.update_document_public(id, false).await?;
        eprintln!("Document is now private");
        output::print_json(&doc, pretty)?;
    }
    if let Some(email) = invite {
        client.invite_to_document(id, email).await?;
        eprintln!("Invited {email}");
    }
    if show_url {
        let url = format!("{}/share/{}", config.url()?, id);
        println!("{url}");
    }

    Ok(())
}
