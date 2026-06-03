use clap::{Parser, Subcommand};
use std::path::PathBuf;
use yama::PackageManager;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        name: String,
        #[arg(short, long)]
        version: Option<String>,
    },
    Remove {
        name: String,
    },
    List,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let install_dir = PathBuf::from("./packages");
    std::fs::create_dir_all(&install_dir)?;

    let mut pm = PackageManager::new(install_dir, Box::new(yama::MockRegistry));

    if let Err(e) = pm.load_state() {
        eprintln!("state initialization failed: {}", e);
    }

    if let Some(command) = cli.command {
        match command {
            Commands::Install { name, version } => {
                pm.install(&name, version.as_deref())?;
                println!("successfully installed {}", name);
            }
            Commands::Remove { name } => {
                pm.remove(&name)?;
                println!("successfully removed {}", name);
            }
            Commands::List => {
                let installed = pm.list_installed();
                if installed.is_empty() {
                    println!("no packages found");
                } else {
                    println!("installed packages:");
                    for pkg in installed {
                        println!("  {}", pkg);
                    }
                }
            }
        }
    }

    Ok(())
}
