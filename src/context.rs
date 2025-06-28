//! ClickHouse context manager for handling connection profiles.
//!
//! This module provides functionality for managing multiple ClickHouse connection
//! profiles, storing credentials securely, and persisting configuration in a
//! TOML file.
//!
//! Each profile stores information like username, password (kept in the system
//! keyring), ClickHouse URLs, and TLS certificate options.
//!
//! # Configuration Path
//!
//! By default, the configuration is saved in a platform-specific config directory,
//! e.g. on Linux: `~/.config/clickcheck/config.toml`.
//!
//! # Profiles
//!
//! Profiles can be created, modified, and selected as the default. Credentials
//! are stored securely using the [`keyring`] crate.
use crate::model::{ContextConfig, ContextProfile};
use secrecy::ExposeSecret;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

const SERVICE_NAME: &str = "clickcheck";

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
/// Manages ClickHouse connection profiles.
///
/// Profiles are persisted in a TOML file, while credentials are stored securely
/// in the system keyring.
pub struct Context {
    path: PathBuf,
    config: ContextConfig,
    /// If the user passed `--context foo` on the CLI, store it here
    override_name: Option<String>,
}

impl Context {
    /// Constructs a new `Context` by reading a config file or creating a default config.
    ///
    /// - `config_path`: Optional path to a custom config file.
    /// - `override_name`: Optional profile name to use instead of the default.
    ///
    /// Returns an error if the path is invalid or the profile does not exist.
    pub fn new(
        config_path: Option<&PathBuf>,
        override_name: Option<&str>,
    ) -> Result<Self, ContextError> {
        let path = config_path.map_or_else(
            || -> Result<PathBuf, ContextError> {
                let path = dirs_next::config_dir()
                    .ok_or(ContextError::InvalidPath)?
                    .join(SERVICE_NAME)
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
    /// Returns a list of all available profile names.
    pub fn list(&self) -> Vec<String> {
        self.config.profiles.keys().cloned().collect()
    }

    /// Returns the name of the currently active profile, either the overridden (see [`Context::new`]) one,
    /// or the default profile from the config.
    pub fn active_profile_name(&self) -> Option<&str> {
        self.override_name
            .as_deref()
            .or(self.config.current.as_deref())
    }

    /// Returns the currently active profile, if available.
    pub fn profile(&self) -> Result<Option<ContextProfile>, ContextError> {
        self.active_profile_name()
            .map(|name| self.get_profile(name))
            .transpose()
    }

    /// Adds or updates a profile with the given name, storing the password securely.
    ///
    /// Writes the config to disk after setting.
    pub fn set_profile(&mut self, profile: ContextProfile, name: &str) -> Result<(), ContextError> {
        self.store_password(name, &profile.password)?;

        self.config.profiles.insert(name.to_string(), profile);
        self.write_to_file()?;

        Ok(())
    }

    /// Delete a profile with the given name.
    ///
    /// Writes the config to disk after setting.
    pub fn delete_profile(&mut self, name: &str) -> Result<(), ContextError> {
        if !self.config.profiles.contains_key(name) {
            return Err(ContextError::ProfileNotFound(name.to_string()));
        }

        self.delete_password(name)?;

        self.config.profiles.remove(name);
        self.write_to_file()?;

        Ok(())
    }

    /// Sets the given profile as the default (used if no `--context` is provided).
    ///
    /// Returns an error if the profile does not exist.
    pub fn set_default(&mut self, name: &str) -> Result<(), ContextError> {
        if !self.config.profiles.contains_key(name) {
            return Err(ContextError::ProfileNotFound(name.to_string()));
        }

        self.config.current = Some(name.to_string());
        self.write_to_file()?;

        Ok(())
    }

    /// Loads a profile by name and fills in its password from the system keyring.
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

    /// Returns the resolved path to the config file used by this context.
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

    fn delete_password(&self, profile_name: &str) -> Result<(), ContextError> {
        let entry = keyring::Entry::new(SERVICE_NAME, profile_name)?;
        entry.delete_credential()?;
        Ok(())
    }

    fn get_password(&self, profile_name: &str) -> Result<secrecy::SecretString, ContextError> {
        let entry = keyring::Entry::new(SERVICE_NAME, profile_name)?;
        let password = entry.get_password()?;
        Ok(secrecy::SecretString::new(password.into()))
    }
}
