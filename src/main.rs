use clap::{Parser, Subcommand};

mod config;
mod commands;

#[derive(Parser)]
#[command(name = "basemark", about = "CLI for Basemark wiki", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output as pretty-printed text
    #[arg(long, global = true)]
    pretty: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure CLI settings
    Config {
        #[command(subcommand)]
        action: commands::config_cmd::ConfigAction,
    },
    /// Create a new document
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        collection: Option<String>,
        #[arg(long)]
        content: Option<String>,
    },
    /// Read a document (outputs markdown)
    Read {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Update a document
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
    },
    /// Delete a document
    Delete {
        id: String,
        #[arg(long)]
        force: bool,
    },
    /// List documents
    List {
        #[arg(long)]
        collection: Option<String>,
    },
    /// Full-text search
    Search {
        query: String,
    },
    /// Manage collections
    Collections {
        #[command(subcommand)]
        action: commands::config_cmd::ConfigAction, // placeholder — will be replaced in Task 4
    },
    /// Share a document
    Share {
        id: String,
        #[arg(long)]
        public: bool,
        #[arg(long)]
        private: bool,
        #[arg(long)]
        invite: Option<String>,
        #[arg(long)]
        url: bool,
    },
    /// Start MCP server (stdio transport)
    Mcp,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { action } => commands::config_cmd::run(action),
        _ => {
            eprintln!("Command not yet implemented");
            Ok(())
        }
    }
}
