use std::{
    io::{self, IsTerminal, Write},
    process::ExitCode,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use mons_box_cli::{app_state::monster::Monster, interactive::event::InteractiveMode};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<SubCommands>,
}

#[derive(Subcommand)]
enum SubCommands {
    /// Feed your monster to reduce hunger
    Feed,
    /// Play with your monster to increase happiness
    Play,
    /// Clean your monster to increase cleanliness
    Sleep,
    /// Show details about your monster
    Status,
    /// Start interactive real-time mode
    Interactive,
    /// Reset the game (create a new monster)
    Reset,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    let mut monster = Monster::load_or_create().context("Failed to load monster state")?;

    match args.command {
        Some(SubCommands::Feed) => {
            let result = monster.feed();
            println!("{}", result);
            monster.save().context("Failed to save monster state")?;
        }
        Some(SubCommands::Play) => {
            let result = monster.play();
            print!("{}", result);
            monster.save().context("Failed to save monster state")?;
        }
        Some(SubCommands::Sleep) => {
            let result = monster.toggle_sleep();
            print!("{}", result);
            monster.save().context("Failed to save monster state")?;
        }
        Some(SubCommands::Status) => {
            if io::stdout().is_terminal() {
                let mut stdout = io::stdout().lock();
                monster
                    .display(&mut stdout)
                    .context("Failed to display monster status")?;
            } else {
                println!("Monster Status:");
                println!("Name: {}", monster.name);
                println!("Hunger: {}%", monster.hunger);
                println!("Happiness: {}%", monster.happiness);
                println!("Energy: {}%", monster.energy);
                println!("Health: {}%", monster.health);
                println!("Age: {} hours", monster.age);
                println!(
                    "Status: {}",
                    if monster.is_sleeping {
                        "Sleeping"
                    } else {
                        "Awake"
                    }
                );
                println!("Alive: {}", if monster.is_alive { "Yes" } else { "No" });
            }
        }
        Some(SubCommands::Interactive) => {
            println!("{}", WELCOME_MESSAGE);
            println!("\nPress Enter to continue...");

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let mut interactive_mode = InteractiveMode::new(monster);
            interactive_mode
                .run()
                .context("Failed to run interactive mode")?;

            println!("\n👋 Thanks for playing! Your progress has been saved.");
        }
        Some(SubCommands::Reset) => {
            println!(
                "Are you sure you want to reset? This will delete your current monster. (y/N)"
            );
            println!("Reset: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" {
                Monster::reset().context("Failed to reset game")?;
                println!("✨ Game reset complete! Run any command to create a new monster.");
            } else {
                print!("🙏 Reset cancelled.");
            }
        }
        None => {
            println!("No command provided. Use --help to see available commands.");
        }
    }

    Ok(ExitCode::SUCCESS)
}

const WELCOME_MESSAGE: &str = "r#
    🎮 Welcome to CLI Mons Box! 🎮

     ╭─────────────────────────╮
     │    Take care of your    │
     │    virtual monster! 🐾  │
     ╰─────────────────────────╯
";
