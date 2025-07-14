pub mod app;
pub mod aws;
pub mod event;
mod models;
pub mod ui;

use crate::app::App;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // Not used yet
    let _args = Cli::parse();

    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}
