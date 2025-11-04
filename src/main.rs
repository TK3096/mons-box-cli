use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::{io, process::ExitCode};

mod interactive;

use interactive::InteractiveMode;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Feed your monster to reduce hunger
    Feed,
    /// Play with your monster to increase happiness
    Play,
    /// Put your monster to sleep to restore energy
    Sleep,
    /// Give your monster a bath to improve cleanliness
    Bath,
    /// Display the current status of your monster
    Status,
    /// Start interactive mode
    Interactive,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    match args.command {
        Some(Commands::Feed) => {
            println!("Feeding the monster...");
        }
        Some(Commands::Play) => {
            println!("Playing with the monster...");
        }
        Some(Commands::Sleep) => {
            println!("Putting the monster to sleep...");
        }
        Some(Commands::Bath) => {
            println!("Giving the monster a bath...");
        }
        Some(Commands::Status) => {
            println!("Displaying the monster's status...");
        }
        Some(Commands::Interactive) => {
            println!("\nPress Enter to continue...");

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let mut interactive_mode = InteractiveMode::new();
            interactive_mode
                .run()
                .context("Failt to run interactive mode")?;

            println!("\n👋 Thanks for playing! Your pet has been saved.");
        }
        None => {
            println!("No command provided. Use --help for more information.");
        }
    }

    Ok(ExitCode::SUCCESS)
}
