//! This crate provides the core functionality for Basalt, a TUI application for Obsidian.
//! It lets you read and manipulate Obsidian's configuration, vaults, and notes.
//!
//! # Example
//!
//! ```
//! use basalt_core::obsidian::{ObsidianConfig, Error};
//!
//! let config = ObsidianConfig::load();
//! ```

#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/basalt-core")]

/// Provides Obisidian interoperability operations
pub mod obsidian;
