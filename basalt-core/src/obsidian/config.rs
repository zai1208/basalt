use dirs::config_local_dir;

use serde::{Deserialize, Deserializer};
use std::result;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::obsidian::{Error, Result, Vault};

/// Represents the Obsidian configuration, typically loaded from an `obsidian.json` file.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ObsidianConfig {
    /// A mapping of vault (folder) names to [`Vault`] definitions.
    vaults: HashMap<String, Vault>,
}

impl ObsidianConfig {
    /// Attempts to locate and load the system's `obsidian.json` file as an [`ObsidianConfig`].
    ///
    /// Returns an [`Error`] if the filepath doesn't exist or JSON parsing failed.
    pub fn load() -> Result<Self> {
        obsidian_config_dir()
            .map(ObsidianConfig::load_from)
            .ok_or_else(|| Error::PathNotFound("Obsidian config directory".to_string()))?
    }

    /// Attempts to load `obsidian.json` file as an [`ObsidianConfig`] from the given directory
    /// [`PathBuf`].
    ///
    /// Returns an [`Error`] if the filepath doesn't exist or JSON parsing failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use basalt_core::obsidian::ObsidianConfig;
    /// use std::path::PathBuf;
    ///
    /// _ = ObsidianConfig::load_from(PathBuf::from("./dir-with-config-file"));
    /// ```
    pub fn load_from(dir: PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(dir.join("obsidian.json"))?;
        Ok(serde_json::from_str(&contents)?)
    }

    /// Returns an iterator over the vaults in the configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use basalt_core::obsidian::{ObsidianConfig, Vault};
    ///
    /// let config = ObsidianConfig::from([
    ///     ("Obsidian", Vault::default()),
    ///     ("Work", Vault::default()),
    /// ]);
    ///
    /// _ = config.vaults();
    /// ```
    pub fn vaults(&self) -> impl Iterator<Item = (String, Vault)> {
        self.vaults.clone().into_iter()
    }

    /// Finds a vault by name, returning a reference if it exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use basalt_core::obsidian::{ObsidianConfig, Vault};
    ///
    /// let config = ObsidianConfig::from([
    ///     ("Obsidian", Vault::default()),
    ///     ("Work", Vault::default()),
    /// ]);
    ///
    /// _ = config.vault_by_name("Obsidian");
    /// ```
    pub fn vault_by_name(&self, name: &str) -> Option<&Vault> {
        self.vaults.get(name)
    }
}

impl<const N: usize> From<[(&str, Vault); N]> for ObsidianConfig {
    /// # Examples
    ///
    /// ```
    /// use basalt_core::obsidian::{ObsidianConfig, Vault};
    ///
    /// let config_1 = ObsidianConfig::from([
    ///   ("Obsidian", Vault::default()),
    ///   ("My Vault", Vault::default()),
    /// ]);
    ///
    /// let config_2: ObsidianConfig = [
    ///   ("Obsidian", Vault::default()),
    ///   ("My Vault", Vault::default()),
    /// ].into();
    ///
    /// assert_eq!(config_1, config_2);
    /// ```
    fn from(arr: [(&str, Vault); N]) -> Self {
        Self {
            vaults: HashMap::from(arr.map(|(name, vault)| (name.to_owned(), vault))),
        }
    }
}

impl<const N: usize> From<[(String, Vault); N]> for ObsidianConfig {
    /// # Examples
    ///
    /// ```
    /// use basalt_core::obsidian::{ObsidianConfig, Vault};
    ///
    /// let config_1 = ObsidianConfig::from([
    ///   (String::from("Obsidian"), Vault::default()),
    ///   (String::from("My Vault"), Vault::default()),
    /// ]);
    ///
    /// let config_2: ObsidianConfig = [
    ///   (String::from("Obsidian"), Vault::default()),
    ///   (String::from("My Vault"), Vault::default()),
    /// ].into();
    ///
    /// assert_eq!(config_1, config_2);
    /// ```
    fn from(arr: [(String, Vault); N]) -> Self {
        Self {
            vaults: HashMap::from(arr),
        }
    }
}

impl<'de> Deserialize<'de> for ObsidianConfig {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Json {
            vaults: HashMap<String, Vault>,
        }

        impl Into<ObsidianConfig> for Json {
            fn into(self) -> ObsidianConfig {
                ObsidianConfig {
                    vaults: self
                        .vaults
                        .into_iter()
                        .map(|(_, vault)| (vault.name.clone(), vault))
                        .collect(),
                }
            }
        }

        Json::from(Deserialize::deserialize(deserializer)?)
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

/// Returns the system path to Obsidian's config folder, if any.
///
/// For reference:
/// - macOS:  `/Users/username/Library/Application Support/obsidian`
/// - Windows: `%APPDATA%\Obsidian\`
/// - Linux:   `$XDG_CONFIG_HOME/Obsidian/` or `~/.config/Obsidian/`
///
/// More info: [https://help.obsidian.md/Files+and+folders/How+Obsidian+stores+data]
fn obsidian_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    const OBSIDIAN_CONFIG_DIR_NAME: &str = "obsidian";

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    const OBSIDIAN_CONFIG_DIR_NAME: &str = "Obsidian";

    config_local_dir().map(|config_path| config_path.join(OBSIDIAN_CONFIG_DIR_NAME))
}
