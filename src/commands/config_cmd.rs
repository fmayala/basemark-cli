use crate::config::Config;
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a config value
    Set { key: String, value: String },
    /// Get a config value
    Get { key: String },
    /// Show full config
    Show,
}

pub fn run(action: ConfigAction) -> Result<()> {
    let mut config = Config::load()?;
    match action {
        ConfigAction::Set { key, value } => {
            match key.as_str() {
                "url" => config.url = Some(value),
                "token" => config.token = Some(value),
                _ => anyhow::bail!("Unknown key: {key}. Valid: url, token"),
            }
            config.save()?;
            eprintln!("Set {key}");
        }
        ConfigAction::Get { key } => {
            match key.as_str() {
                "url" => println!("{}", config.url.unwrap_or_default()),
                "token" => {
                    if let Some(t) = config.token {
                        let prefix = if t.len() > 11 { &t[..11] } else { &t };
                        println!("{}...", prefix);
                    }
                }
                _ => anyhow::bail!("Unknown key: {key}"),
            }
        }
        ConfigAction::Show => {
            println!("{}", toml::to_string_pretty(&config)?);
        }
    }
    Ok(())
}
