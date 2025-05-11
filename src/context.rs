use crate::model::{ContextConfig, ContextProfile};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("read config error: {0}")]
    ReadConfig(#[from] std::io::Error),
    #[error("parse toml error: {0}")]
    ParseToml(#[from] toml::de::Error),
    #[error("failed to serialize config TOML: {0}")]
    SerializeToml(String),
    #[error("invalid config path")]
    InvalidPath,
    #[error("failed to persist temp file: {0}")]
    PersistTempFile(#[from] tempfile::PersistError),
    #[error("context profile '{0}' not found")]
    ProfileNotFound(String),
}

#[derive(Debug)]
pub struct Context {
    path: PathBuf,
    config: ContextConfig,
    /// If the user passed `--context foo` on the CLI, store it here
    override_name: Option<String>,
}

impl Context {
    pub fn new(
        config_path: Option<&PathBuf>,
        override_name: Option<&str>,
    ) -> Result<Self, ContextError> {
        let path = match config_path {
            Some(p) => p.clone(),
            None => {
                let default_dir = dirs_next::config_dir()
                    .ok_or(ContextError::InvalidPath)?
                    .join("ch-query-analyzer");

                // Создаём директорию при необходимости
                std::fs::create_dir_all(&default_dir)?;

                default_dir.join("config.toml")
            }
        };
        if !path.exists() {
            return Ok(Self {
                path,
                config: ContextConfig::default(),
                override_name: None,
            });
        }
        let content = fs::read_to_string(&path)?;
        let config: ContextConfig = toml::from_str(&content)?;
        let override_name = override_name.map(|n| n.to_string());

        if let Some(name) = override_name.as_deref() {
            if !config.profiles.contains_key(name) {
                return Err(ContextError::ProfileNotFound(name.to_string()));
            }
        }

        Ok(Self {
            config,
            path,
            override_name,
        })
    }

    /// Список всех профилей
    pub fn list(&self) -> Vec<String> {
        self.config.profiles.keys().cloned().collect()
    }

    pub fn active_profile_name(&self) -> Option<&str> {
        self.override_name
            .as_deref()
            .or_else(|| self.config.current.as_deref())
    }

    pub fn profile(&self) -> Option<&ContextProfile> {
        let name = self
            .override_name
            .as_deref()
            .or_else(|| self.config.current.as_deref())?;
        self.config.profiles.get(name)
    }

    pub fn set_profile(&mut self, profile: ContextProfile, name: &str) -> Result<(), ContextError> {
        self.config.profiles.insert(name.to_string(), profile);
        self.write_to_file()?;

        Ok(())
    }

    pub fn set_default(&mut self, name: &str) -> Result<(), ContextError> {
        if !self.config.profiles.contains_key(name) {
            return Err(ContextError::ProfileNotFound(name.to_string()));
        }

        self.config.current = Some(name.to_string());
        self.write_to_file()?;

        Ok(())
    }

    pub fn get_profile(&self, name: &str) -> Result<ContextProfile, ContextError> {
        if let Some(profile) = self.config.profiles.get(name) {
            Ok(profile.clone())
        } else {
            Err(ContextError::ProfileNotFound(name.to_string()))
        }
    }

    fn write_to_file(&self) -> Result<(), ContextError> {
        let toml = toml::to_string_pretty(&self.config)
            .map_err(|e| ContextError::SerializeToml(e.to_string()))?;

        let dir = self
            .path
            .parent()
            .ok_or(ContextError::InvalidPath)?
            .to_path_buf();

        let mut tmp_file = tempfile::NamedTempFile::new_in(dir)?;
        tmp_file.write_all(toml.as_bytes())?;
        tmp_file.flush()?;

        tmp_file.persist(&self.path)?;
        Ok(())
    }
}
