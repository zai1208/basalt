[[Basalt]] features and functionality can be customized using a user-defined configuration file. The configuration file should be located in one of the following directories:

**macOS and Unix:**

- `$HOME/.basalt.toml`
- `$XDG_CONFIG_HOME/basalt/config.toml`

**Windows:**

- `%USERPROFILE%\.basalt.toml`
- `%APPDATA%\basalt\config.toml`

If configuration files exist in multiple locations, only the first one found will be used, with the home directory configuration taking precedence. 

> [!WARNING]
>
> This behavior may change in future versions to merge all found configurations instead.

## Key Mappings

Basalt key mappings can be modified or extended by defining key mappings in the user configuration file.

Each key mapping is associated with a specific 'pane' and becomes active when that pane has focus. The global section applies to all panes and is evaluated first.

## Default configuration

```toml
# The corresponding pane needs to be _active_ in order for the keybindings to
# be read and the attached command activated.
#
# Global commands:
#
# quit: exits the application
# vault_selector_modal_toggle: toggles vault selector modal (not available in splash screen)
# help_modal_toggle: toggles help modal
#
# Splash commands:
#
# splash_up: moves selector up
# splash_down: moves selector down
# splash_open: opens the selected vault
#
# Explorer commands:
#
# explorer_up: moves selector up
# explorer_down: moves selector down
# explorer_open: opens the selected note in note viewer
# explorer_sort: toggles note and folder sorting between A-z and Z-a 
# explorer_toggle: toggles explorer panel
# explorer_switch_pane: switches pane from explorer to note viewer
# explorer_scroll_up_one: scrolls the selector up by one
# explorer_scroll_down_one: scrolls the selector down by one
# explorer_scroll_up_half_page: scrolls the selector up half a page
# explorer_scroll_down_half_page: scrolls the selector down half a page
#
# Note editor commands:
#
# note_editor_scroll_up_one: scrolls up by one
# note_editor_scroll_down_one: scrolls down by one
# note_editor_scroll_up_half_page: scrolls up by half page
# note_editor_scroll_down_half_page: scrolls down by half page
# note_editor_toggle_explorer: toggles explorer panel
# note_editor_switch_pane: switches pane from note viewer to explorer
#
# Help modal commands:
#
# help_modal_toggle: toggles help modal
# help_modal_close: closes help modal
# help_modal_scroll_up_one: scrolls up by one
# help_modal_scroll_down_one: scrolls down by one
# help_modal_scroll_up_half_page: scrolls up by half page
# help_modal_scroll_down_half_page: scrolls down by half page
#
# Vault selector modal commands:
#
# vault_selector_modal_up: moves selector up
# vault_selector_modal_down: moves selector down
# vault_selector_modal_close: closes vault selector modal
# vault_selector_modal_open: opens the selected vault 
# vault_selector_modal_toggle: toggles vault selector modal

# Editor is disabled by default. To enable editor change this setting to true.
experimental_editor = false

[global]
key_bindings = [
 { key = "q", command = "quit" },
 { key = "ctrl+g", command = "vault_selector_modal_toggle" },
 { key = "?", command = "help_modal_toggle" },
]

[splash]
key_bindings = [
 { key = "k", command = "splash_up" },
 { key = "j", command = "splash_down" },
 { key = "up", command = "splash_up" },
 { key = "down", command = "splash_down" },
 { key = "enter", command = "splash_open" },
]

[explorer]
key_bindings = [
 { key = "k", command = "explorer_up" },
 { key = "j", command = "explorer_down" },
 { key = "up", command = "explorer_up" },
 { key = "down", command = "explorer_down" },
 { key = "t", command = "explorer_toggle" },
 { key = "s", command = "explorer_sort" },
 { key = "tab", command = "explorer_switch_pane" },
 { key = "enter", command = "explorer_open" },
 { key = "ctrl+b", command = "explorer_toggle" },
 { key = "ctrl+u", command = "explorer_scroll_up_half_page" },
 { key = "ctrl+d", command = "explorer_scroll_down_half_page" },
]

[note_editor]
key_bindings = [
 { key = "k", command = "note_editor_cursor_up" },
 { key = "j", command = "note_editor_cursor_down" },
 { key = "up", command = "note_editor_cursor_up" },
 { key = "down", command = "note_editor_cursor_down" },
 { key = "t", command = "note_editor_toggle_explorer" },
 { key = "tab", command = "note_editor_switch_pane" },
 { key = "ctrl+b", command = "note_editor_toggle_explorer" },
 { key = "ctrl+u", command = "note_editor_scroll_up_half_page" },
 { key = "ctrl+d", command = "note_editor_scroll_down_half_page" },

 # Experimental editor 
 { key = "i", command = "note_editor_experimental_set_edit_mode" },
 { key = "shift+r", command = "note_editor_experimental_set_read_mode" },
 { key = "ctrl+x", command = "note_editor_experimental_save" },
 { key = "esc", command = "note_editor_experimental_exit_mode" },
 # 'f' translates to arrow key right
 { key = "alt+f", command = "note_editor_experimental_cursor_word_forward" },
 # 'b' translates to arrow key left
 { key = "alt+b", command = "note_editor_experimental_cursor_word_backward" },
]

[help_modal]
key_bindings = [
 { key = "esc", command = "help_modal_close" },
 { key = "k", command = "help_modal_scroll_up_one" },
 { key = "j", command = "help_modal_scroll_down_one" },
 { key = "up", command = "help_modal_scroll_up_one" },
 { key = "down", command = "help_modal_scroll_down_one" },
 { key = "ctrl+u", command = "help_modal_scroll_up_half_page" },
 { key = "ctrl+d", command = "help_modal_scroll_down_half_page" },
]

[vault_selector_modal]
key_bindings = [
 { key = "k", command = "vault_selector_modal_up" },
 { key = "j", command = "vault_selector_modal_down" },
 { key = "up", command = "vault_selector_modal_up" },
 { key = "down", command = "vault_selector_modal_down" },
 { key = "enter", command = "vault_selector_modal_open" },
 { key = "esc", command = "vault_selector_modal_close" },
]
```
