/*
* Password Manager
* Stores passwords encrypted via a private - public key pair
*/
use std::string::String;
use clap::{Parser, Subcommand};

use cli::{App, auth_device, register};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// A simple password manager
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create your credentials
    Register {
        #[clap(short, long)]
        device_pass: String,
    },
    /// Authenticate device using your seed phrase
    Login {
        #[clap(short, long)]
        seed_phrase: String,
        #[clap(short, long)]
        device_pass: String,
    },
    /// Add a new password
    Add {
        #[clap(short, long)]
        site: String,
        #[clap(short, long)]
        username: String,
        #[clap(short, long)]
        password: String,
        #[clap(short, long)]
        device_pass: String,
    },
    /// Get a password
    Get {
        #[clap(short, long)]
        site: String,
        #[clap(short, long)]
        username: Option<String>,
        #[clap(short, long)]
        device_pass: String,
    },
}


#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Register {device_pass} => {
            register(&device_pass).await;
        },
        Commands::Login { seed_phrase, device_pass } => {
            auth_device(&seed_phrase, &device_pass);
        },
        Commands::Add { site, username, password, device_pass } => {
            App::new(&device_pass).add(site, username, password).await
                .expect("failed to add password");
        },
        Commands::Get { site, username, device_pass } => {
            App::new(&device_pass).get(site, username);
        }
    }

}




