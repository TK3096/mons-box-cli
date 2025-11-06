use std::{
    fmt::format,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const SAVE_FILE_PATH: &str = ".monster_state.json";
const MAX_STAT: u8 = 100;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MonsterState {
    pub name: String,
    pub hunger: u8,
    pub happiness: u8,
    pub energy: u8,
    pub cleanliness: u8,
    pub health: u8,
    pub age: u32,
    pub last_updated: DateTime<Utc>,
    pub is_sleeping: bool,
    pub is_alive: bool,
}

impl Default for MonsterState {
    fn default() -> Self {
        Self {
            name: "Crappy".to_string(),
            hunger: 20,
            happiness: 80,
            energy: 50,
            cleanliness: 40,
            health: 100,
            age: 0,
            last_updated: Utc::now(),
            is_sleeping: false,
            is_alive: true,
        }
    }
}

impl MonsterState {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn update_from_time_passage(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn load_or_create() -> Result<Self> {
        if Path::new(SAVE_FILE_PATH).exists() {
            let mut file = File::open(SAVE_FILE_PATH)
                .with_context(|| format!("Failed to open save file {}", SAVE_FILE_PATH))?;

            let mut content = String::new();

            file.read_to_string(&mut content)
                .with_context(|| "Failed to read file content")?;

            let mut monster: MonsterState =
                serde_json::from_str(&content).with_context(|| "Failed to parse save file")?;

            monster.update_from_time_passage()?;

            Ok(monster)
        } else {
            println!("🥚 A new monster has hatched! What would you like to name them?");
            print!("Name: ");

            io::stdout().flush()?;

            let mut name = String::new();
            io::stdin().read_line(&mut name)?;
            let name = name.trim().to_string();

            let monster = if name.is_empty() {
                MonsterState::default()
            } else {
                MonsterState::new(name)
            };

            monster.save()?;

            println!("🎉 Meet {}! Take good care of them!", monster.name);

            Ok(monster)
        }
    }

    fn save(&self) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(SAVE_FILE_PATH)
            .with_context(|| format!("Failed to create/open save file {}", SAVE_FILE_PATH))?;

        let json = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize monster state")?;

        file.write_all(json.as_bytes())
            .with_context(|| "Failed to write monster state to file")?;

        Ok(())
    }
}
