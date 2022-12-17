use simplelog::*;

use hdn::node::{Node, NodeConfig};

fn main() {
    std::env::set_var("HDN_CONFIG", "/home/frisssby/hdn/config/config.json");
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .unwrap();
    let config = NodeConfig::build();
    let node = Node::init(config);
    node.launch();
}
