use std::{
    cmp::Ordering,
    fs::{self, read_dir},
    path::{Path, PathBuf},
    result,
    time::SystemTime,
};

use serde::{Deserialize, Deserializer};

use crate::obsidian::Note;

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
    /// Returns an iterator over Markdown (`.md`) files in this vault as [`Note`] structs.
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
    /// assert_eq!(vault.notes().collect::<Vec<_>>(), vec![]);
    /// ```
    pub fn notes(&self) -> impl Iterator<Item = Note> {
        read_dir(&self.path)
            .into_iter()
            .flatten()
            .filter_map(|entry| Option::<Note>::from(DirEntry::from(entry.ok()?)))
    }

    /// Returns a sorted vector [Vec<Note>] of all notes in the vault, sorted according to the
    /// provided comparison function.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use basalt_core::obsidian::{Vault, Note};
    ///
    /// let vault = Vault {
    ///     name: "MyVault".to_string(),
    ///     path: "path/to/my_vault".into(),
    ///     ..Default::default()
    /// };
    ///
    /// let alphabetically = |a: &Note, b: &Note| a.name.to_lowercase().cmp(&b.name.to_lowercase());
    ///
    /// let notes = vault.notes_sorted_by(alphabetically);
    /// for note in notes {
    ///     println!("{}", note.name);
    /// }
    /// ```
    pub fn notes_sorted_by(&self, compare: impl Fn(&Note, &Note) -> Ordering) -> Vec<Note> {
        let mut notes: Vec<Note> = self.notes().collect();
        notes.sort_by(compare);
        notes
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

        impl TryInto<Vault> for Json {
            type Error = String;
            fn try_into(self) -> result::Result<Vault, Self::Error> {
                let path = Path::new(&self.path);
                let name = path
                    .file_name()
                    .ok_or_else(|| String::from("unable to retrieve vault name"))?
                    .to_string_lossy()
                    .to_string();
                Ok(Vault {
                    name,
                    path: self.path,
                    open: self.open.unwrap_or_default(),
                    ts: self.ts,
                })
            }
        }

        Json::from(Deserialize::deserialize(deserializer)?)
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

impl Default for Note {
    fn default() -> Self {
        Self {
            name: String::default(),
            path: PathBuf::default(),
            created: SystemTime::now(),
        }
    }
}

/// Internal wrapper for directory entries to implement custom conversion between [`fs::DirEntry`]
/// and [`Option<Note>`].
#[derive(Debug)]
struct DirEntry(fs::DirEntry);

impl From<fs::DirEntry> for DirEntry {
    fn from(value: fs::DirEntry) -> Self {
        DirEntry(value)
    }
}

impl From<DirEntry> for Option<Note> {
    /// Transforms path with extension `.md` into [`Option<Note>`].
    fn from(value: DirEntry) -> Option<Note> {
        let dir = value.0;
        let created = dir.metadata().ok()?.created().ok()?;
        let path = dir.path();

        if path.extension()? != "md" {
            return None;
        }

        let name = path
            .with_extension("")
            .file_name()
            .map(|file_name| file_name.to_string_lossy().into_owned())?;

        Some(Note {
            name,
            path,
            created,
        })
    }
}
