use std::{path::PathBuf, result};

use serde::{Deserialize, Deserializer};

use super::vault_entry::VaultEntry;

/// Represents a single Obsidian vault.
///
/// A vault is a folder containing notes and other metadata.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Vault {
    /// The name of the vault, inferred from its directory name.
    pub name: String,

    /// Filesystem path to the vault's directory.
    pub path: PathBuf,

    /// Whether the vault is marked 'open' by Obsidian.
    pub open: bool,

    /// Timestamp of last update or creation.
    pub ts: u64,
}

impl Vault {
    /// Returns a [`Vec`] of Markdown vault entries in this vault as [`VaultEntry`] structs.
    /// Entries can be either directories or files (notes). If the directory is marked hidden with
    /// a dot (`.`) prefix it will be filtered out from the resulting [`Vec`].
    ///
    /// The returned entries are not sorted.
    ///
    /// # Examples
    ///
    /// ```
    /// use basalt_core::obsidian::{Vault, Note};
    ///
    /// let vault = Vault {
    ///     name: "MyVault".into(),
    ///     path: "path/to/my_vault".into(),
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(vault.entries(), vec![]);
    /// ```
    pub fn entries(&self) -> Vec<VaultEntry> {
        match self.path.as_path().try_into() {
            Ok(VaultEntry::Directory { entries, .. }) => entries
                .into_iter()
                .filter(|entry| !entry.name().starts_with('.'))
                .collect(),
            _ => vec![],
        }
    }
}

impl<'de> Deserialize<'de> for Vault {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Json {
            path: PathBuf,
            open: Option<bool>,
            ts: u64,
        }

        impl TryFrom<Json> for Vault {
            type Error = String;
            fn try_from(Json { path, open, ts }: Json) -> result::Result<Self, Self::Error> {
                let name = path
                    .file_name()
                    .map(|file_name| file_name.to_string_lossy().to_string())
                    .ok_or("unable to retrieve vault name")?;

                Ok(Vault {
                    name,
                    path,
                    open: open.unwrap_or(false),
                    ts,
                })
            }
        }

        let deserialized: Json = Deserialize::deserialize(deserializer)?;
        deserialized.try_into().map_err(serde::de::Error::custom)
    }
}
