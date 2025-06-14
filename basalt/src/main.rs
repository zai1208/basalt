use std::io;

use basalt_core::obsidian::ObsidianConfig;
use basalt_tui::app::App;

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let obsidian_config = ObsidianConfig::load().unwrap();
    let vaults = obsidian_config.vaults();

    App::start(terminal, vaults)?;

    ratatui::restore();

    Ok(())
}
