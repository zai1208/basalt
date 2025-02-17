# Basalt Core: basalt-core

This crate provides the core functionality for Basalt TUI application and
interoperability layer to Obsidian vaults and notes.

## Obsidian

Obsidian module provides functionality operating with Obsidian. It lets you
read and manipulate Obsidian's configuration, vaults, and notes.

Currently supports reading vaults, notes, and writing to note path.

### Example

```
use basalt_core::obsidian::{ObsidianConfig, Error, Vault};

let config = ObsidianConfig::from([
  ("Obsidian", Vault::default()),
  ("My Vault", Vault::default()),
]);

_ = config.get_vault_by_name("Obsidian");
```
