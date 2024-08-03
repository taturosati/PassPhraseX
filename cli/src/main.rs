/*
* Password Manager
* Stores passwords encrypted via a private - public key pair
*/
use clap::{Parser, Subcommand};
use std::error::Error;
use std::string::String;

use passphrasex::{auth_device, register, App};
use passphrasex_common::generator::generate_password;

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
        device_pass: Option<String>,
    },
    /// Authenticate device using your seed phrase
    Login {
        #[clap(short, long)]
        seed_phrase: Option<String>,
        #[clap(short, long)]
        device_pass: Option<String>,
    },
    /// Add a new password
    Add {
        #[clap(short, long)]
        site: String,
        #[clap(short, long)]
        username: String,
        #[clap(short, long)]
        password: Option<String>,
        #[clap(short, long)]
        device_pass: Option<String>,
    },
    /// Get a password
    Get {
        #[clap(short, long)]
        site: String,
        #[clap(short, long)]
        username: Option<String>,
        #[clap(short, long)]
        device_pass: Option<String>,
    },
    /// Modify a password
    Edit {
        #[clap(short, long)]
        site: String,
        #[clap(short, long)]
        username: String,
        #[clap(short, long)]
        password: Option<String>,
        #[clap(short, long)]
        device_pass: Option<String>,
    },
    /// Delete a password
    Delete {
        #[clap(short, long)]
        site: String,
        #[clap(short, long)]
        username: String,
        #[clap(short, long)]
        device_pass: Option<String>,
    },
    /// Generate a random password
    Generate {
        #[clap(short, long)]
        length: Option<usize>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Register { device_pass } => {
            match register(&get_or_prompt_device_password(device_pass)).await {
                Ok(seed_phrase) => println!(
                    "Successfully registered!\nYour seed phrase is: \n{}",
                    seed_phrase.get_phrase()
                ),
                Err(e) => println!("Failed to create user: {}", e),
            }
        }
        Commands::Login {
            seed_phrase,
            device_pass,
        } => match auth_device(
            &get_or_prompt_seed_phrase(seed_phrase),
            &get_or_prompt_device_password(device_pass),
        )
        .await
        {
            Ok(_) => println!("Successfully authenticated!"),
            Err(e) => println!("Failed to authenticate: {}", e),
        },
        Commands::Add {
            site,
            username,
            password,
            device_pass,
        } => {
            match App::new(&get_or_prompt_device_password(device_pass))
                .await?
                .add(site, username, get_or_prompt_password(password))
                .await
            {
                Ok(_) => println!("Password added successfully"),
                Err(e) => println!("Failed to add password: {}", e),
            }
        }
        Commands::Get {
            site,
            username,
            device_pass,
        } => match App::new(&get_or_prompt_device_password(device_pass))
            .await?
            .get(site, username)
            .await
        {
            Ok(passwords) => {
                for credential in passwords {
                    println!(
                        "username: {}\npassword: {}\n",
                        credential.username, credential.password
                    );
                }
            }
            Err(e) => println!("Failed to get password: {}", e),
        },
        Commands::Edit {
            site,
            username,
            password,
            device_pass,
        } => {
            match App::new(&get_or_prompt_device_password(device_pass))
                .await?
                .edit(site, username, get_or_prompt_password(password))
                .await
            {
                Ok(_) => println!("Password edited successfully"),
                Err(e) => println!("Failed to edit password: {}", e),
            }
        }
        Commands::Delete {
            site,
            username,
            device_pass,
        } => match App::new(&get_or_prompt_device_password(device_pass))
            .await?
            .delete(site, username)
            .await
        {
            Ok(_) => println!("Password deleted successfully"),
            Err(e) => println!("Failed to delete password: {}", e),
        },
        Commands::Generate { length } => {
            println!("{}", generate_password(length.unwrap_or(16)));
        }
    };

    Ok(())
}

fn get_or_prompt(password: Option<String>, name: &str) -> String {
    if let Some(password) = password {
        password
    } else {
        rpassword::prompt_password(format!("Enter {name}:")).expect("Failed to read password")
    }
}

fn get_or_prompt_device_password(password: Option<String>) -> String {
    get_or_prompt(password, "device password")
}

fn get_or_prompt_seed_phrase(password: Option<String>) -> String {
    get_or_prompt(password, "seed phrase")
}

fn get_or_prompt_password(password: Option<String>) -> String {
    get_or_prompt(password, "site password")
}
