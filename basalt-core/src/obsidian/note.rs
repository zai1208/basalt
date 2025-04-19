use std::{fs, path::PathBuf, time::SystemTime};

use crate::obsidian::{Error, Result};

/// Represents a single note (Markdown file) within a vault.
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    /// The base filename without `.md` extension.
    pub name: String,

    /// Filesystem path to the `.md` file.
    pub path: PathBuf,

    /// File creation time.
    ///
    /// TODO: Use chrono or time crates for better time format handling
    pub created: SystemTime,
}

impl Default for Note {
    fn default() -> Self {
        Self {
            name: String::default(),
            path: PathBuf::default(),
            created: SystemTime::UNIX_EPOCH,
        }
    }
}

impl Note {
    /// Reads the note's contents from disk to a `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use basalt_core::obsidian::Note;
    ///
    /// let note = Note {
    ///     name: "Example".to_string(),
    ///     path: "path/to/Example.md".into(),
    ///     ..Default::default()
    /// };
    ///
    /// _ = Note::read_to_string(&note);
    /// ```
    pub fn read_to_string(note: &Note) -> Result<String> {
        fs::read_to_string(&note.path).map_err(Error::Io)
    }

    /// Writes given content to notes path.
    ///
    /// # Examples
    ///
    /// ```
    /// use basalt_core::obsidian::Note;
    ///
    /// let note = Note {
    ///     name: "Example".to_string(),
    ///     path: "path/to/Example.md".into(),
    ///     ..Default::default()
    /// };
    ///
    /// _ = Note::write(&note, String::from("# Heading"));
    /// ```
    pub fn write(note: &Note, contents: String) -> Result<()> {
        fs::write(&note.path, contents).map_err(Error::Io)
    }
}
