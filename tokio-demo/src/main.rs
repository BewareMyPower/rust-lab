pub mod server;

use clap::{Parser, Subcommand};
use mini_redis::Result;

#[derive(Parser)]
#[command(version, about = "Mini Redis Server", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Server {
        #[arg(long, default_value_t = String::from("127.0.0.1:6379"))]
        url: String,
    },
    Client {
        #[arg(long, default_value_t = String::from("127.0.0.1:6379"))]
        url: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Server { url }) => server::run_server(&url).await,
        Some(Commands::Client { .. }) => Err("client mode is not implemented yet".into()),
        None => Err("No command provided. Use --help for usage information.".into()),
    }
}
