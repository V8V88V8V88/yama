use clap::{Parser, Subcommand};
use std::path::PathBuf;
use yama::{Package, PackageManager};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Install { name: String },
    Remove { name: String },
    List,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let install_dir = PathBuf::from("./packages");
    std::fs::create_dir_all(&install_dir)?;

    let mut pm = PackageManager::new(install_dir);

    pm.load_installed_packages()?;

    // Consider loading these packages from a config file or arguments
    add_initial_packages(&mut pm)?;

    if let Some(command) = cli.command {
        execute_command(command, &mut pm)?;
    } else {
        println!("No command specified. Use --help for usage information.");
    }

    Ok(())
}

fn add_initial_packages(pm: &mut PackageManager) -> Result<(), Box<dyn std::error::Error>> {
    let initial_packages = vec![
        Package {
            name: "yama".to_string(),
            version: "1.0".to_string(),
            dependencies: vec!["libfoo".to_string()],
            url: "https://example.com/yama-1.0.zip".to_string(),
        },
        Package {
            name: "libfoo".to_string(),
            version: "1.2".to_string(),
            dependencies: vec!["libbar".to_string()],
            url: "https://example.com/libfoo-1.2.zip".to_string(),
        },
        Package {
            name: "libbar".to_string(),
            version: "2.0".to_string(),
            dependencies: vec![],
            url: "https://example.com/libbar-2.0.zip".to_string(),
        },
    ];

    for package in initial_packages {
        pm.add_package(package);
    }

    Ok(())
}

fn execute_command(command: Commands, pm: &mut PackageManager) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Install { name } => {
            println!("Installing package: {}", name);
            pm.install_package(&name).map_err(|e| format!("Failed to install package: {}", e))?;
        }
        Commands::Remove { name } => {
            println!("Removing package: {}", name);
            pm.remove_package(&name).map_err(|e| format!("Failed to remove package: {}", e))?;
        }
        Commands::List => {
            println!("Installed packages:");
            for package in pm.list_installed_packages() {
                println!("- {}", package);
            }
        }
    }
    Ok(())
}
