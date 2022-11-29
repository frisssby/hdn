use clap::Parser;
use simplelog::*;
use std::net::IpAddr;

#[derive(Debug, Parser)]
struct Opts {
    #[clap(short, long)]
    ip: IpAddr,

    #[clap(short, long)]
    port: u16,
}

fn main() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .unwrap();

    let opts = Opts::parse();
    hdn::run(opts.ip, opts.port);
}
