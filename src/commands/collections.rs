use anyhow::Result;
use clap::Subcommand;

use crate::client::BasemarkClient;
use crate::config::Config;
use crate::output;

#[derive(Subcommand)]
pub enum CollectionAction {
    /// List all collections
    List,
    /// Create a new collection
    Create { name: String },
    /// Delete a collection
    Delete {
        id: String,
        #[arg(long)]
        force: bool,
    },
}

pub async fn run(config: &Config, action: CollectionAction, pretty: bool) -> Result<()> {
    let client = BasemarkClient::new(config.url()?, config.token()?);

    match action {
        CollectionAction::List => {
            let collections = client.list_collections().await?;
            output::print_json(&collections, pretty)
        }
        CollectionAction::Create { name } => {
            let collection = client.create_collection(&name).await?;
            output::print_json(&collection, pretty)
        }
        CollectionAction::Delete { id, force } => {
            if !force {
                eprintln!("Are you sure? Use --force to confirm");
                return Ok(());
            }
            client.delete_collection(&id).await?;
            eprintln!("Deleted collection {id}");
            Ok(())
        }
    }
}
