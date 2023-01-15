use simplelog::*;

use clap::Parser;
use hdn::node::{NetworkConfig, Node};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, value_name = "PATH")]
    config: PathBuf,
    #[arg(long, value_name = "NODE_ID")]
    id: usize,
}

fn main() {
    let cli = Cli::parse();
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .unwrap();
    let config = NetworkConfig::build(&cli.config);
    let node = Node::init(config, cli.id);
    node.launch();
}
