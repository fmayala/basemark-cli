use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub url: Option<String>,
    pub token: Option<String>,
}

impl Config {
    pub fn path() -> Result<PathBuf> {
        let dir = dirs::home_dir()
            .context("Cannot find home directory")?
            .join(".basemark");
        fs::create_dir_all(&dir)?;
        Ok(dir.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn url(&self) -> Result<&str> {
        self.url.as_deref().context("URL not configured. Run: basemark config set url <url>")
    }

    pub fn token(&self) -> Result<&str> {
        self.token.as_deref().context("Token not configured. Run: basemark config set token <token>")
    }
}
