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

    pm.add_package(Package {
        name: "yama".to_string(),
        version: "1.0".to_string(),
        dependencies: vec!["libfoo".to_string()],
        url: "https://example.com/yama-1.0.zip".to_string(),
    });

    pm.add_package(Package {
        name: "libfoo".to_string(),
        version: "1.2".to_string(),
        dependencies: vec!["libbar".to_string()],
        url: "https://example.com/libfoo-1.2.zip".to_string(),
    });

    pm.add_package(Package {
        name: "libbar".to_string(),
        version: "2.0".to_string(),
        dependencies: vec![],
        url: "https://example.com/libbar-2.0.zip".to_string(),
    });

    match &cli.command {
        Some(Commands::Install { name }) => {
            println!("Installing package: {}", name);
            pm.install_package(name)?;
        }
        Some(Commands::Remove { name }) => {
            println!("Removing package: {}", name);
            pm.remove_package(name)?;
        }
        Some(Commands::List) => {
            println!("Installed packages:");
            for package in pm.list_installed_packages() {
                println!("- {}", package);
            }
        }
        None => {
            println!("No command specified. Use --help for usage information.");
        }
    }

    Ok(())
}
