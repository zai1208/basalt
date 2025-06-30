use std::io;

use basalt_core::obsidian::ObsidianConfig;
use basalt_tui::app::App;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let obsidian_config = ObsidianConfig::load().unwrap();
    let vaults = obsidian_config.vaults();

    terminal.show_cursor()?;

    App::start(terminal, vaults)?;

    ratatui::restore();

    Ok(())
}
