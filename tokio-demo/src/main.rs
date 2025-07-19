pub mod server;

use clap::{Parser, Subcommand};
use mini_redis::{Result, client};

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

        #[command(subcommand)]
        client_commands: Option<ClientCommands>,
    },
}

#[derive(Subcommand)]
enum ClientCommands {
    Get { key: String },
    Set { key: String, value: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Server { url }) => server::run_server(&url).await,
        Some(Commands::Client {
            url,
            client_commands,
        }) => match client_commands {
            Some(ClientCommands::Get { key }) => {
                let mut client = client::connect(&url).await?;
                match client.get(&key).await? {
                    Some(value) => {
                        let string_value = String::from_utf8(value.to_vec()).unwrap();
                        println!("Got value {:?} for key {:?}", string_value, key);
                    }
                    None => println!("No value found for key {:?}", key),
                }
                Ok(())
            }
            Some(ClientCommands::Set { key, value }) => {
                let mut client = client::connect(&url).await?;
                let value_clone = value.clone();
                client.set(&key, value.into()).await?;
                println!("Set {:?} => {:?} in Redis", key, value_clone);
                Ok(())
            }
            None => Err("No client command provided. Use --help for usage information.".into()),
        },
        None => Err("No command provided. Use --help for usage information.".into()),
    }
}
