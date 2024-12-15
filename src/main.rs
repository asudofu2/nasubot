use clap::Parser;
use log::info;
use std::env;
use std::error::Error;
use std::fs::File;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long, help = "設定ファイルパス")]
    config_file_path: String,
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args = Args::parse();

    let config = parse_config(&args.config_file_path).unwrap_or_else(|e| {
        eprintln!("Failed to parse config: {}", e);
        std::process::exit(1);
    });

    info!("Config: {:?}", config);

    nasubot::run(&config).await.unwrap_or_else(|e| {
        eprintln!("Failed to run: {}", e);
        std::process::exit(1);
    });
}

fn parse_config(config_file_path: &str) -> Result<nasubot::Config, Box<dyn Error>> {
    let f = File::open(config_file_path)?;
    let config = serde_json::from_reader(f)?;
    Ok(config)
}
