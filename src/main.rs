use std::{collections::HashMap, env, error::Error, fs, path::PathBuf};

use arboard::Clipboard;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(
    name = "rclip",
    version = "0.1.1",
    about = "A simple key-value store with clipboard support"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Get {
        key: String,
        #[arg(short = 'k', long)]
        by_key: bool,
    },
    Set {
        key: String,
        value: String,
    },
    Del {
        key: String,
        #[arg(short = 'k', long)]
        by_key: bool,
    },
    List,
    Copy {
        key: String,
        #[arg(short = 'k', long)]
        by_key: bool,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct Record {
    id: u64,
    key: String,
    value: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut db = load_db();

    match cli.command {
        Commands::Get { key, by_key } => {
            let record = if by_key {
                db.values().find(|r| r.key == key)
            } else {
                key.parse::<u64>().ok().and_then(|id| db.get(&id))
            };

            if let Some(rec) = record {
                println!("{}", rec.value);
            } else {
                println!("(nil)")
            }
        }
        Commands::Set { key, value } => {
            let id = db.keys().max().map(|x| x + 1).unwrap_or(1);
            db.insert(id, Record { id, key, value });
            save_db(&db)?;
            println!("OK (id = {})", id);
        }
        Commands::Del { key, by_key } => {
            let removed = if by_key {
                let remove_id = db.iter().find(|(_, r)| r.key == key).map(|(id, _)| *id);
                remove_id.and_then(|id| db.remove(&id))
            } else {
                key.parse::<u64>().ok().and_then(|id| db.remove(&id))
            };

            if removed.is_some() {
                save_db(&db)?;
                println!("(integer) 1");
            } else {
                println!("(integer) 0")
            }
        }
        Commands::List => {
            let mut items: Vec<_> = db.values().collect();
            items.sort_by_key(|r| r.id);
            for rec in items {
                println!("{} => ({} => {})", rec.id, rec.key, rec.value);
            }
        }
        Commands::Copy { key, by_key } => {
            let record = if by_key {
                db.values().find(|r| r.key == key)
            } else {
                key.parse::<u64>().ok().and_then(|id| db.get(&id))
            };

            if let Some(rec) = record {
                let mut clipboard = Clipboard::new()?;
                let _ = clipboard.set_text(rec.value.clone());
                println!("copied '{}' to clipboard", rec.value)
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

fn load_db() -> HashMap<u64, Record> {
    let db_path = get_db_path();
    if PathBuf::from(&db_path).exists() {
        let content = fs::read_to_string(db_path).unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new())
    } else {
        HashMap::new()
    }
}

fn save_db(db: &HashMap<u64, Record>) -> Result<(), std::io::Error> {
    let json = serde_json::to_string_pretty(db).unwrap();
    fs::write(get_db_path(), json)
}
