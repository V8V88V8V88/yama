use clap::{Parser, Subcommand};
use env_logger;
use log::info;
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
    /// Installs a specified package and its dependencies
    Install { name: String },
    /// Removes a specified package
    Remove { name: String },
    /// Lists all installed packages
    List,
    /// Displays details about a specific package
    Info { name: String },
}

const YAMA_URL: &str = "https://example.com/yama-1.0.zip";
const LIBFOO_URL: &str = "https://example.com/libfoo-1.2.zip";
const LIBBAR_URL: &str = "https://example.com/libbar-2.0.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();
    info!("Yama Package Manager started");

    let cli = Cli::parse();
    let install_dir = PathBuf::from("./packages");
    std::fs::create_dir_all(&install_dir)?;

    let mut pm = PackageManager::new(install_dir);

    // Load installed packages
    pm.load_installed_packages()?;

    // Define available packages
    pm.add_package(Package {
        name: "yama".to_string(),
        version: "1.0".to_string(),
        dependencies: vec!["libfoo".to_string()],
        url: YAMA_URL.to_string(),
    });

    pm.add_package(Package {
        name: "libfoo".to_string(),
        version: "1.2".to_string(),
        dependencies: vec!["libbar".to_string()],
        url: LIBFOO_URL.to_string(),
    });

    pm.add_package(Package {
        name: "libbar".to_string(),
        version: "2.0".to_string(),
        dependencies: vec![],
        url: LIBBAR_URL.to_string(),
    });

    match &cli.command {
        Some(Commands::Install { name }) => {
            info!("Installing package: {}", name);
            match pm.install_package(name) {
                Ok(_) => println!("Successfully installed: {}", name),
                Err(e) => eprintln!("Failed to install package {}: {}", name, e),
            }
        }
        Some(Commands::Remove { name }) => {
            info!("Removing package: {}", name);
            match pm.remove_package(name) {
                Ok(_) => println!("Successfully removed: {}", name),
                Err(e) => eprintln!("Failed to remove package {}: {}", name, e),
            }
        }
        Some(Commands::List) => {
            println!("Installed packages:");
            for package in pm.list_installed_packages() {
                println!("- {}", package);
            }
        }
        Some(Commands::Info { name }) => {
            info!("Fetching package info for: {}", name);
            if let Some(package) = pm.get_package_info(name) {
                println!(
                    "Package: {}\nVersion: {}\nDependencies: {:?}\nURL: {}",
                    package.name, package.version, package.dependencies, package.url
                );
            } else {
                println!("Package not found: {}", name);
            }
        }
        None => {
            println!("No command specified. Use --help for usage information.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_install_command() {
        let install_dir = PathBuf::from("./test_packages");
        fs::create_dir_all(&install_dir).unwrap();
        let mut pm = PackageManager::new(install_dir);

        pm.add_package(Package {
            name: "test_pkg".to_string(),
            version: "1.0".to_string(),
            dependencies: vec![],
            url: "https://example.com/test_pkg.zip".to_string(),
        });

        assert!(pm.install_package("test_pkg").is_ok());
    }

    #[test]
    fn test_list_command() {
        let install_dir = PathBuf::from("./test_packages");
        let mut pm = PackageManager::new(install_dir);

        pm.add_package(Package {
            name: "test_pkg".to_string(),
            version: "1.0".to_string(),
            dependencies: vec![],
            url: "https://example.com/test_pkg.zip".to_string(),
        });

        pm.install_package("test_pkg").unwrap();
        let installed_packages = pm.list_installed_packages();
        assert!(installed_packages.contains(&"test_pkg".to_string()));
    }

    #[test]
    fn test_info_command() {
        let install_dir = PathBuf::from("./test_packages");
        let mut pm = PackageManager::new(install_dir);

        pm.add_package(Package {
            name: "test_pkg".to_string(),
            version: "1.0".to_string(),
            dependencies: vec![],
            url: "https://example.com/test_pkg.zip".to_string(),
        });

        let package = pm.get_package_info("test_pkg");
        assert!(package.is_some());
        assert_eq!(package.unwrap().name, "test_pkg".to_string());
    }
}
