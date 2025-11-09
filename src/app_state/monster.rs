use anyhow::Result;
use crossterm::{
    QueueableCommand,
    style::{Color, ResetColor, SetForegroundColor},
};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, StdoutLock, Write},
    path::Path,
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

const MONSTER_STATE_FILE: &str = ".monster-state.json";
const STAT_DECAY_RATE: u8 = 2;
const SLEEP_RECOVERY_RATE: u8 = 10;
const MAX_STAT: u8 = 100;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Monster {
    pub name: String,
    pub hunger: u8,
    pub happiness: u8,
    pub energy: u8,
    pub health: u8,
    pub age: u32,
    pub is_sleeping: bool,
    pub is_alive: bool,
    pub updated_at: DateTime<Utc>,
}

impl Default for Monster {
    fn default() -> Self {
        Self {
            name: "Fluffy".to_string(),
            hunger: 50,
            happiness: 70,
            energy: 80,
            health: 100,
            age: 0,
            is_sleeping: false,
            is_alive: true,
            updated_at: Utc::now(),
        }
    }
}

impl Monster {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn load_or_create() -> Result<Self> {
        if Path::new(MONSTER_STATE_FILE).exists() {
            let mut file = File::open(MONSTER_STATE_FILE)
                .with_context(|| format!("Failed to open state file {}", MONSTER_STATE_FILE))?;

            let mut content = String::new();
            file.read_to_string(&mut content)
                .with_context(|| "Failed to read state file")?;

            let mut monster: Monster =
                serde_json::from_str(&content).with_context(|| "Failed to parse state file")?;

            monster.update_from_time_passage()?;
            monster.save()?;

            Ok(monster)
        } else {
            println!("🥚 A new monster has hatched! What would you like to name them?");
            println!("Name: ");
            io::stdout().flush()?;

            let mut name = String::new();
            io::stdin().read_line(&mut name)?;
            let name = name.trim().to_string();

            let monster = if name.is_empty() {
                Monster::default()
            } else {
                Monster::new(name)
            };

            monster.save()?;
            println!("🎉 Meet {}! Take good care of them!", monster.name);

            Ok(monster)
        }
    }

    pub fn save(&self) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(MONSTER_STATE_FILE)
            .with_context(|| format!("Failed to create/open state file {}", MONSTER_STATE_FILE))?;

        let json = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize monster state")?;

        file.write_all(json.as_bytes())
            .with_context(|| "Failed to write monster state to file")?;

        Ok(())
    }

    pub fn update_from_time_passage(&mut self) -> Result<()> {
        let now = Utc::now();
        let time_passed = now.signed_duration_since(self.updated_at);
        let hours_passed = time_passed.num_hours();

        if hours_passed > 0 {
            let hours_clamped = (hours_passed as u32).min(1000);
            self.age = self.age.saturating_add(hours_clamped);

            let decay_amount =
                ((hours_clamped as u32 * STAT_DECAY_RATE as u32) / 1).min(MAX_STAT as u32) as u8;
            let recovery_amount = ((hours_clamped as u32 * SLEEP_RECOVERY_RATE as u32) / 2)
                .min(MAX_STAT as u32) as u8;

            if self.is_sleeping {
                self.energy = (self.energy.saturating_add(recovery_amount)).min(MAX_STAT);
                self.hunger = (self.hunger.saturating_add(decay_amount / 2)).min(MAX_STAT);
            } else {
                self.hunger = (self.hunger.saturating_add(decay_amount)).min(MAX_STAT);
                self.happiness = (self.happiness.saturating_sub(decay_amount / 2)).max(1);
                self.energy = (self.energy.saturating_sub(decay_amount)).max(0);
            }

            if self.hunger > 80 || self.happiness < 20 || self.energy < 10 {
                self.health = self.health.saturating_sub((decay_amount * 2).max(1));
            }

            if self.health == 0 {
                self.is_alive = false;
            }
        }

        self.updated_at = now;

        Ok(())
    }

    pub fn reset() -> Result<()> {
        if Path::new(MONSTER_STATE_FILE).exists() {
            std::fs::remove_file(MONSTER_STATE_FILE)
                .with_context(|| "Failed to remove state file")?;
        }

        println!("💫 Starting fresh with a new monster!");
        Ok(())
    }

    pub fn feed(&mut self) -> String {
        if !self.is_alive {
            return format!("💀 {} has passed away...", self.name);
        }

        if self.is_sleeping {
            return format!("😴 {} is sleeping peacefully. Try again later!", self.name);
        }

        if self.hunger <= 20 {
            self.happiness = self.happiness.saturating_sub(5);
            return format!("🤢 {} is too full to eat more!", self.name);
        }

        self.hunger = self.hunger.saturating_sub(25);
        self.happiness = (self.happiness + 10).min(MAX_STAT);
        self.health = (self.health + 5).min(MAX_STAT);

        let foods = ["🍎", "🥕", "🍖", "🐟", "🥛"];
        let food = foods[rand::rng().random_range(0..foods.len())];

        format!("{} ate {} and feels much better!", self.name, food)
    }

    pub fn play(&mut self) -> String {
        if !self.is_alive {
            return format!("💀 {} has passed away...", self.name);
        }

        if self.is_sleeping {
            return format!("😴 {} is sleeping peacefully. Try again later!", self.name);
        }

        if self.energy < 20 {
            return format!("😫 {} is too tired to play right now!", self.name);
        }

        if self.hunger > 80 {
            return format!("😵 {} is too hungry to play! Feed them first!", self.name);
        }

        self.happiness = (self.happiness + 20).min(MAX_STAT);
        self.energy = self.energy.saturating_sub(15);
        self.hunger = (self.hunger + 5).min(MAX_STAT);

        let activities = ["⚽", "🎾", "🛹", "🎮", "🏀"];
        let activity = activities[rand::rng().random_range(0..activities.len())];

        format!("{} played {} and is super happy!", self.name, activity)
    }

    pub fn toggle_sleep(&mut self) -> String {
        if !self.is_alive {
            return format!("💀 {} has passed away...", self.name);
        }

        self.is_sleeping = !self.is_sleeping;

        if self.is_sleeping {
            format!("😴 {} has gone to sleep. Sweet dreams!", self.name)
        } else {
            format!("🌞 {} has woken up feeling refreshed!", self.name)
        }
    }

    pub fn get_mood(&self) -> (&str, &str) {
        if !self.is_alive {
            return ("💀", "Dead");
        }

        if self.is_sleeping {
            return ("😴", "Sleeping");
        }

        let avg_stat = (self.happiness as u16
            + (MAX_STAT.saturating_sub(self.hunger)) as u16
            + self.health as u16
            + self.energy as u16)
            / 4;

        let avg_stat = avg_stat as u8;

        match avg_stat {
            90..=100 => ("😁", "Ecstatic"),
            75..=89 => ("😊", "Happy"),
            60..=74 => ("🙂", "Content"),
            45..=59 => ("😐", "Okay"),
            30..=44 => ("☹️", "Sad"),
            15..=29 => ("😢", "Very Sad"),
            _ => ("😵", "Critical"),
        }
    }

    pub fn display(&self, stdout: &mut StdoutLock) -> Result<()> {
        let (emoji, mood) = self.get_mood();

        write!(stdout, "╭─────────────────────────────────╮\r\n")?;
        write!(stdout, "│      🐲  Monster Status  🐲     │\r\n")?;
        write!(stdout, "╰─────────────────────────────────╯\r\n")?;
        writeln!(stdout)?;

        if !self.is_alive {
            write!(stdout, "        💀     💀\r\n")?;
            write!(stdout, "          ╲   ╱\r\n")?;
            write!(stdout, "           ╲ ╱\r\n")?;
            write!(stdout, "         ───┴───\r\n")?;
            write!(stdout, "        💀 R.I.P 💀\r\n")?;
        } else if self.is_sleeping {
            write!(stdout, "          zzZ\r\n")?;
            write!(stdout, "        ╭─────╮\r\n")?;
            write!(stdout, "       ╱  - -  ╲\r\n")?;
            write!(stdout, "      ╱    ω    ╲\r\n")?;
            write!(stdout, "     ╱___________╲\r\n")?;
            write!(stdout, "        😴💤💤\r\n")?;
        } else {
            let face = match self.get_mood().0 {
                "😁" => ("◕", "‿", "◕"),
                "😊" => ("^", "‿", "^"),
                "🙂" => ("•", "‿", "•"),
                "😐" => ("•", "_", "•"),
                "☹️" => ("•", "︵", "•"),
                "😢" => ("╥", "﹏", "╥"),
                "😵" => ("x", "_", "x"),
                _ => ("•", "‿", "•"),
            };

            write!(stdout, "        ╭─────╮\r\n")?;
            write!(stdout, "       ╱  {} {}  ╲\r\n", face.0, face.2)?;
            write!(stdout, "      ╱    {}    ╲\r\n", face.1)?;
            write!(stdout, "     ╱___________╲\r\n")?;
            write!(stdout, "        {}  {}\r\n", emoji, self.name)?;
        }

        writeln!(stdout)?;
        write!(stdout, "📊 Stats:\r\n")?;

        self.draw_status_bar(
            stdout,
            "🍽️  Hunger",
            MAX_STAT - self.hunger,
            Color::Green,
            Color::Red,
        )?;
        self.draw_status_bar(
            stdout,
            "😊 Happiness",
            self.happiness,
            Color::Yellow,
            Color::Grey,
        )?;
        self.draw_status_bar(stdout, "💖 Health", self.health, Color::Red, Color::DarkRed)?;
        self.draw_status_bar(
            stdout,
            "⚡ Energy",
            self.energy,
            Color::Cyan,
            Color::DarkCyan,
        )?;

        writeln!(stdout)?;
        write!(stdout, "📈 Info:")?;
        write!(stdout, "   Age: {} hours old\r\n", self.age)?;
        write!(stdout, "   Mood: {}\r\n", mood)?;
        write!(
            stdout,
            "   Status: {}\r\n",
            if self.is_sleeping {
                "😴 Sleeping"
            } else {
                "👁️ Awake"
            }
        )?;

        if !self.is_alive {
            stdout.queue(SetForegroundColor(Color::Red))?;
            writeln!(stdout)?;
            write!(
                stdout,
                "💀 Your pet has died. You can start over with a new pet.\r\n"
            )?;
            stdout.queue(ResetColor)?;
        } else {
            writeln!(stdout)?;
            write!(
                stdout,
                "🎮 Commands: feed, play, sleep, status, interactive\r\n"
            )?;

            if self.hunger > 70 {
                stdout.queue(SetForegroundColor(Color::Red))?;
                write!(stdout, "⚠️  {} is very hungry!\r\n", self.name)?;
                stdout.queue(ResetColor)?;
            }
            if self.happiness < 30 {
                stdout.queue(SetForegroundColor(Color::Yellow))?;
                write!(
                    stdout,
                    "⚠️  {} looks sad. Try playing with them!\r\n",
                    self.name
                )?;
                stdout.queue(ResetColor)?;
            }
            if self.energy < 20 {
                stdout.queue(SetForegroundColor(Color::Cyan))?;
                write!(
                    stdout,
                    "⚠️  {} is exhausted. Let them sleep!\r\n",
                    self.name
                )?;
                stdout.queue(ResetColor)?;
            }
            if self.health < 50 {
                stdout.queue(SetForegroundColor(Color::Red))?;
                write!(
                    stdout,
                    "⚠️  {} doesn't look well. Take better care!\r\n",
                    self.name
                )?;
                stdout.queue(ResetColor)?;
            }
        }

        stdout.flush()?;
        Ok(())
    }

    fn draw_status_bar(
        &self,
        stdout: &mut StdoutLock,
        label: &str,
        value: u8,
        good_color: Color,
        bad_color: Color,
    ) -> Result<()> {
        let bar_width = 20;
        let filled = (value as usize * bar_width) / MAX_STAT as usize;
        let empty = bar_width - filled;

        write!(stdout, "   {}: [", label)?;

        let color = if value > 60 { good_color } else { bad_color };
        stdout.queue(SetForegroundColor(color))?;

        for _ in 0..filled {
            write!(stdout, "█")?;
        }

        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        for _ in 0..empty {
            write!(stdout, "░")?;
        }

        stdout.queue(ResetColor)?;
        write!(stdout, "] {}%\r\n", value)?;

        Ok(())
    }
}
