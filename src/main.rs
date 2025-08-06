use std::{collections::HashMap, env, error::Error, fs, path::PathBuf};

use arboard::Clipboard;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rclip",
    version = "1.0",
    about = "A simple key-value store with clipboard support"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Get { key: String },
    Set { key: String, value: String },
    Del { key: String },
    List,
    Copy { key: String },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut db = load_db();

    match cli.command {
        Commands::Get { key } => {
            if let Some(value) = db.get(&key) {
                println!("{}", value);
            } else {
                println!("(nil)")
            }
        }
        Commands::Set { key, value } => {
            db.insert(key.clone(), value.clone());
            save_db(&db)?;
            println!("OK");
        }
        Commands::Del { key } => {
            if db.remove(&key).is_some() {
                save_db(&db)?;
                println!("(integer) 1");
            } else {
                println!("(integer) 0")
            }
        }
        Commands::List => {
            for (k, v) in &db {
                println!("{}=>{}", k, v)
            }
        }
        Commands::Copy { key } => {
            if let Some(value) = db.get(&key) {
                let mut clipboard = Clipboard::new()?;
                let _ = clipboard.set_text(value.clone());
                println!("copied '{}' to clipboard", value)
            } else {
                println!("(nil)")
            }
        }
    }

    Ok(())
}

fn get_db_path() -> PathBuf {
    let exe_path = env::current_exe().expect("Failed to get executable path");
    let exe_dir = exe_path
        .parent()
        .expect("Failed to get executable directory");
    exe_dir.join("rclip_db.json")
}

fn load_db() -> HashMap<String, String> {
    let db_path = get_db_path();
    if PathBuf::from(&db_path).exists() {
        let content = fs::read_to_string(db_path).unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new())
    } else {
        HashMap::new()
    }
}

fn save_db(db: &HashMap<String, String>) -> Result<(), std::io::Error> {
    let json = serde_json::to_string_pretty(db).unwrap();
    fs::write(get_db_path(), json)
}
