use crate::model::{ContextConfig, ContextProfile};
use secrecy::ExposeSecret;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

const SERVICE_NAME: &str = "ch-query-analyzer";

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("read config error: {0}")]
    ReadConfig(#[from] std::io::Error),
    #[error("parse toml error in {path}: {source}")]
    ParseToml {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("failed to serialize config TOML: {0}")]
    SerializeToml(String),
    #[error("invalid config path")]
    InvalidPath,
    #[error("failed to persist temp file: {0}")]
    PersistTempFile(#[from] tempfile::PersistError),
    #[error("context profile '{0}' not found")]
    ProfileNotFound(String),
    #[error("keyring error: {0}")]
    KeyringError(#[from] keyring::Error),
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
        let path = config_path.map_or_else(
            || -> Result<PathBuf, ContextError> {
                let path = dirs_next::config_dir()
                    .ok_or(ContextError::InvalidPath)?
                    .join("ch-query-analyzer")
                    .join("config.toml");
                Ok(path)
            },
            |p| Ok(p.clone()),
        )?;
        // Создаём директорию при необходимости
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let config = if path.exists() {
            let content = fs::read_to_string(&path)?;
            toml::from_str(&content).map_err(|e| ContextError::ParseToml {
                path: path.clone(),
                source: e,
            })?
        } else {
            ContextConfig::default()
        };

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
            .or(self.config.current.as_deref())
    }

    pub fn profile(&self) -> Result<Option<ContextProfile>, ContextError> {
        self.active_profile_name()
            .map(|name| self.get_profile(name))
            .transpose()
    }

    pub fn set_profile(&mut self, profile: ContextProfile, name: &str) -> Result<(), ContextError> {
        self.store_password(name, &profile.password)?;

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
        let mut profile = self
            .config
            .profiles
            .get(name)
            .ok_or_else(|| ContextError::ProfileNotFound(name.to_string()))?
            .clone();

        profile.password = self.get_password(name)?;
        Ok(profile)
    }

    pub fn get_config_path(&self) -> &PathBuf {
        &self.path
    }

    // --- Приватные вспомогательные методы ---

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

    fn store_password(
        &self,
        profile_name: &str,
        password: &secrecy::SecretString,
    ) -> Result<(), ContextError> {
        let entry = keyring::Entry::new(SERVICE_NAME, profile_name)?;
        entry.set_password(password.expose_secret())?;
        Ok(())
    }

    fn get_password(&self, profile_name: &str) -> Result<secrecy::SecretString, ContextError> {
        let entry = keyring::Entry::new(SERVICE_NAME, profile_name)?;
        let password = entry.get_password()?;
        Ok(secrecy::SecretString::new(password.into()))
    }
}
