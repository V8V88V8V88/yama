use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::copy;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub url: String,
}

pub struct PackageManager {
    packages: HashMap<String, Package>,
    install_dir: PathBuf,
}

impl PackageManager {
    pub fn new(install_dir: PathBuf) -> Self {
        PackageManager {
            packages: HashMap::new(),
            install_dir,
        }
    }

    pub fn add_package(&mut self, package: Package) {
        self.packages.insert(package.name.clone(), package);
    }

    pub fn resolve_dependencies(&self, root: &str) -> Vec<String> {
        let mut resolved = Vec::new();
        let mut stack = vec![root.to_string()];

        while let Some(pkg) = stack.pop() {
            if !resolved.contains(&pkg) {
                if let Some(package) = self.packages.get(&pkg) {
                    for dep in &package.dependencies {
                        stack.push(dep.clone());
                    }
                    resolved.push(pkg);
                }
            }
        }

        resolved.reverse();
        resolved
    }

    pub fn download_package(&self, name: &str) -> Result<(), Box<dyn Error>> {
        if let Some(package) = self.packages.get(name) {
            let url = &package.url;
            let resp = ureq::get(url).call()?;
            let dest_path = self
                .install_dir
                .join(format!("{}-{}.zip", name, package.version));
            let mut dest = fs::File::create(&dest_path)?;
            copy(&mut resp.into_reader(), &mut dest)?;
            println!("Downloaded package {} to {:?}", name, dest_path);
            Ok(())
        } else {
            Err(format!("Package {} not found", name).into())
        }
    }

    pub fn extract_package(&self, name: &str) -> Result<(), Box<dyn Error>> {
        let package = self.packages.get(name).ok_or("Package not found")?;
        let package_file = self
            .install_dir
            .join(format!("{}-{}.zip", name, package.version));
        let extraction_dir = self.install_dir.join(name);

        let file = fs::File::open(&package_file)?;
        let mut archive = zip::ZipArchive::new(file)?;

        fs::create_dir_all(&extraction_dir)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = extraction_dir.join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        println!("Extracted package {} to {:?}", name, extraction_dir);
        Ok(())
    }

    pub fn install_package(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        let deps = self.resolve_dependencies(name);
        for dep in deps {
            self.download_package(&dep)?;
            self.extract_package(&dep)?;
        }
        self.save_installed_packages()?;
        Ok(())
    }

    pub fn remove_package(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        if !self.packages.contains_key(name) {
            return Err(format!("Package {} is not installed", name).into());
        }

        // Remove the package directory
        let package_dir = self.install_dir.join(name);
        fs::remove_dir_all(package_dir)?;

        // Remove the package from the list of installed packages
        self.packages.remove(name);
        self.save_installed_packages()?;

        println!("Removed package: {}", name);
        Ok(())
    }

    pub fn list_installed_packages(&self) -> Vec<String> {
        self.packages.keys().cloned().collect()
    }

    fn save_installed_packages(&self) -> Result<(), Box<dyn Error>> {
        let packages: Vec<_> = self.packages.values().cloned().collect();
        let json = serde_json::to_string_pretty(&packages)?;
        fs::write(self.install_dir.join("installed_packages.json"), json)?;
        Ok(())
    }

    pub fn load_installed_packages(&mut self) -> Result<(), Box<dyn Error>> {
        let file_path = self.install_dir.join("installed_packages.json");
        if file_path.exists() {
            let json = fs::read_to_string(file_path)?;
            let packages: Vec<Package> = serde_json::from_str(&json)?;
            for package in packages {
                self.add_package(package);
            }
        }
        Ok(())
    }
}
