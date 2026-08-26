//! anb — a thin shell over `anb_core`: hosts own adapters and rendering,
//! all logic lives in the Core.

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "anb",
    version,
    about = "A project's working memory as typed records in the repository"
)]
struct Cli {}

fn main() {
    Cli::parse();
    println!("anb: no commands yet");
}
