use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::io::copy;
use std::path::{Path, PathBuf};
use semver::{Version, VersionReq};

#[derive(Debug)]
pub enum YamaError {
    Io(std::io::Error),
    Network(String),
    Registry(String),
    Resolution(String),
    Serialization(serde_json::Error),
    PackageNotFound(String),
}

impl fmt::Display for YamaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YamaError::Io(e) => write!(f, "IO error: {}", e),
            YamaError::Network(e) => write!(f, "Network error: {}", e),
            YamaError::Registry(e) => write!(f, "Registry error: {}", e),
            YamaError::Resolution(e) => write!(f, "Dependency resolution error: {}", e),
            YamaError::Serialization(e) => write!(f, "Serialization error: {}", e),
            YamaError::PackageNotFound(p) => write!(f, "Package not found: {}", p),
        }
    }
}

impl std::error::Error for YamaError {}

impl From<std::io::Error> for YamaError {
    fn from(e: std::io::Error) -> Self {
        YamaError::Io(e)
    }
}

impl From<serde_json::Error> for YamaError {
    fn from(e: serde_json::Error) -> Self {
        YamaError::Serialization(e)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_req: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<Dependency>,
    pub url: String,
    pub checksum: String,
}

extern "C" {
    fn download_package_ffi(url: *const libc::c_char, destination: *const libc::c_char) -> libc::c_int;
    fn verify_checksum_ffi(file_path: *const libc::c_char, expected_checksum: *const libc::c_char) -> libc::c_int;
}

pub trait Registry {
    fn fetch_metadata(&self, name: &str, version_req: &VersionReq) -> Result<PackageMetadata, YamaError>;
}

pub struct MockRegistry;

impl Registry for MockRegistry {
    fn fetch_metadata(&self, name: &str, version_req: &VersionReq) -> Result<PackageMetadata, YamaError> {
        let packages = vec![
            PackageMetadata {
                name: "yama".to_string(),
                version: "1.0.0".to_string(),
                dependencies: vec![Dependency { name: "libfoo".to_string(), version_req: "^1.0".to_string() }],
                url: "https://example.com/yama-1.0.0.zip".to_string(),
                checksum: "hash1".to_string(),
            },
            PackageMetadata {
                name: "libfoo".to_string(),
                version: "1.2.0".to_string(),
                dependencies: vec![Dependency { name: "libbar".to_string(), version_req: ">=2.0".to_string() }],
                url: "https://example.com/libfoo-1.2.0.zip".to_string(),
                checksum: "hash2".to_string(),
            },
            PackageMetadata {
                name: "libbar".to_string(),
                version: "2.0.1".to_string(),
                dependencies: vec![],
                url: "https://example.com/libbar-2.0.1.zip".to_string(),
                checksum: "hash3".to_string(),
            },
        ];

        packages.into_iter()
            .filter(|p| p.name == name)
            .find(|p| {
                let v = Version::parse(&p.version).unwrap();
                version_req.matches(&v)
            })
            .ok_or_else(|| YamaError::PackageNotFound(format!("{} matching {}", name, version_req)))
    }
}

pub struct LocalRegistry {
    pub packages: Vec<PackageMetadata>,
}

impl Registry for LocalRegistry {
    fn fetch_metadata(&self, name: &str, version_req: &VersionReq) -> Result<PackageMetadata, YamaError> {
        self.packages.iter()
            .filter(|p| p.name == name)
            .find(|p| {
                let v = Version::parse(&p.version).unwrap();
                version_req.matches(&v)
            })
            .cloned()
            .ok_or_else(|| YamaError::PackageNotFound(format!("{} matching {}", name, version_req)))
    }
}

pub struct PackageManager {
    install_dir: PathBuf,
    installed_packages: HashMap<String, PackageMetadata>,
    registry: Box<dyn Registry>,
}

impl PackageManager {
    pub fn new(install_dir: PathBuf, registry: Box<dyn Registry>) -> Self {
        PackageManager {
            install_dir,
            installed_packages: HashMap::new(),
            registry,
        }
    }

    pub fn install(&mut self, name: &str, version_req: Option<&str>) -> Result<(), YamaError> {
        let req = if let Some(r) = version_req {
            VersionReq::parse(r).map_err(|e| YamaError::Resolution(e.to_string()))?
        } else {
            VersionReq::STAR
        };

        println!("Resolving dependencies: {} ({})", name, req);
        
        let mut to_install = Vec::new();
        self.resolve(name, &req, &mut to_install, &mut HashSet::new())?;

        for pkg in to_install {
            if self.installed_packages.contains_key(&pkg.name) {
                continue;
            }
            self.download_and_extract(&pkg)?;
            self.installed_packages.insert(pkg.name.clone(), pkg);
        }

        self.save_state()?;
        Ok(())
    }

    fn resolve(
        &self,
        name: &str,
        req: &VersionReq,
        to_install: &mut Vec<PackageMetadata>,
        visited: &mut HashSet<String>,
    ) -> Result<(), YamaError> {
        if visited.contains(name) {
            return Ok(());
        }
        visited.insert(name.to_string());

        let pkg = self.registry.fetch_metadata(name, req)?;
        
        for dep in &pkg.dependencies {
            let dep_req = VersionReq::parse(&dep.version_req)
                .map_err(|e| YamaError::Resolution(e.to_string()))?;
            self.resolve(&dep.name, &dep_req, to_install, visited)?;
        }

        to_install.push(pkg);
        Ok(())
    }

    fn download_and_extract(&self, pkg: &PackageMetadata) -> Result<(), YamaError> {
        let dest_path = self.install_dir.join(format!("{}-{}.zip", pkg.name, pkg.version));
        
        fs::create_dir_all(&self.install_dir)?;

        use std::ffi::CString;
        let c_url = CString::new(pkg.url.as_str()).unwrap();
        let c_dest = CString::new(dest_path.to_str().unwrap()).unwrap();
        
        unsafe {
            download_package_ffi(c_url.as_ptr(), c_dest.as_ptr());
        }

        if pkg.url.starts_with("file://") {
            let path = pkg.url.trim_start_matches("file://");
            fs::copy(path, &dest_path)?;
        } else {
            let resp = ureq::get(&pkg.url).call().map_err(|e| YamaError::Network(e.to_string()))?;
            let mut dest = fs::File::create(&dest_path)?;
            copy(&mut resp.into_body().into_reader(), &mut dest)?;
        }

        let c_checksum = CString::new(pkg.checksum.as_str()).unwrap();
        unsafe {
            if verify_checksum_ffi(c_dest.as_ptr(), c_checksum.as_ptr()) != 0 {
                return Err(YamaError::Registry("Checksum verification failed".to_string()));
            }
        }

        self.extract(&dest_path, &pkg.name)?;
        Ok(())
    }

    fn extract(&self, zip_path: &Path, name: &str) -> Result<(), YamaError> {
        let extraction_dir = self.install_dir.join(name);
        fs::create_dir_all(&extraction_dir)?;

        let file = fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| YamaError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| YamaError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
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

        Ok(())
    }

    pub fn list_installed(&self) -> Vec<String> {
        self.installed_packages.keys().cloned().collect()
    }

    pub fn remove(&mut self, name: &str) -> Result<(), YamaError> {
        if !self.installed_packages.contains_key(name) {
            return Err(YamaError::PackageNotFound(name.to_string()));
        }

        let package_dir = self.install_dir.join(name);
        if package_dir.exists() {
            fs::remove_dir_all(package_dir)?;
        }

        self.installed_packages.remove(name);
        self.save_state()?;
        Ok(())
    }

    fn save_state(&self) -> Result<(), YamaError> {
        let packages: Vec<_> = self.installed_packages.values().cloned().collect();
        let json = serde_json::to_string_pretty(&packages)?;
        fs::write(self.install_dir.join("installed_packages.json"), json)?;
        Ok(())
    }

    pub fn load_state(&mut self) -> Result<(), YamaError> {
        let file_path = self.install_dir.join("installed_packages.json");
        if file_path.exists() {
            let json = fs::read_to_string(file_path)?;
            let packages: Vec<PackageMetadata> = serde_json::from_str(&json)?;
            for package in packages {
                self.installed_packages.insert(package.name.clone(), package);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;

    fn create_dummy_zip(path: &Path) {
        let file = fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("hello.txt", zip::write::FileOptions::<()>::default()).unwrap();
        zip.write_all(b"Hello, world!").unwrap();
        zip.finish().unwrap();
    }

    #[test]
    fn test_install_and_remove() {
        let dir = tempdir().unwrap();
        let install_dir = dir.path().join("packages");
        let zip_path = dir.path().join("test.zip");
        create_dummy_zip(&zip_path);

        let registry = LocalRegistry {
            packages: vec![
                PackageMetadata {
                    name: "test-pkg".to_string(),
                    version: "1.0.0".to_string(),
                    dependencies: vec![],
                    url: format!("file://{}", zip_path.to_str().unwrap()),
                    checksum: "hash".to_string(),
                }
            ]
        };

        let mut pm = PackageManager::new(install_dir.clone(), Box::new(registry));
        pm.install("test-pkg", None).expect("Install failed");

        assert!(pm.list_installed().contains(&"test-pkg".to_string()));
        assert!(install_dir.join("test-pkg/hello.txt").exists());

        pm.remove("test-pkg").expect("Remove failed");
        assert!(!pm.list_installed().contains(&"test-pkg".to_string()));
        assert!(!install_dir.join("test-pkg").exists());
    }

    #[test]
    fn test_dependency_resolution() {
        let dir = tempdir().unwrap();
        let install_dir = dir.path().join("packages");
        let zip_path = dir.path().join("test.zip");
        create_dummy_zip(&zip_path);

        let registry = LocalRegistry {
            packages: vec![
                PackageMetadata {
                    name: "a".to_string(),
                    version: "1.0.0".to_string(),
                    dependencies: vec![Dependency { name: "b".to_string(), version_req: "^1.0".to_string() }],
                    url: format!("file://{}", zip_path.to_str().unwrap()),
                    checksum: "hash-a".to_string(),
                },
                PackageMetadata {
                    name: "b".to_string(),
                    version: "1.1.0".to_string(),
                    dependencies: vec![],
                    url: format!("file://{}", zip_path.to_str().unwrap()),
                    checksum: "hash-b".to_string(),
                }
            ]
        };

        let mut pm = PackageManager::new(install_dir, Box::new(registry));
        pm.install("a", None).expect("Install failed");

        let installed = pm.list_installed();
        assert!(installed.contains(&"a".to_string()));
        assert!(installed.contains(&"b".to_string()));
    }

    #[test]
    fn test_circular_dependency() {
        let dir = tempdir().unwrap();
        let install_dir = dir.path().join("packages");
        let zip_path = dir.path().join("test.zip");
        create_dummy_zip(&zip_path);

        let registry = LocalRegistry {
            packages: vec![
                PackageMetadata {
                    name: "a".to_string(),
                    version: "1.0.0".to_string(),
                    dependencies: vec![Dependency { name: "b".to_string(), version_req: "*".to_string() }],
                    url: format!("file://{}", zip_path.to_str().unwrap()),
                    checksum: "hash-a".to_string(),
                },
                PackageMetadata {
                    name: "b".to_string(),
                    version: "1.0.0".to_string(),
                    dependencies: vec![Dependency { name: "a".to_string(), version_req: "*".to_string() }],
                    url: format!("file://{}", zip_path.to_str().unwrap()),
                    checksum: "hash-b".to_string(),
                }
            ]
        };

        let mut pm = PackageManager::new(install_dir, Box::new(registry));
        pm.install("a", None).expect("Circular dependency should be handled");
        
        let installed = pm.list_installed();
        assert!(installed.contains(&"a".to_string()));
        assert!(installed.contains(&"b".to_string()));
    }

    #[test]
    fn test_invalid_version_req() {
        let dir = tempdir().unwrap();
        let install_dir = dir.path().join("packages");
        let mut pm = PackageManager::new(install_dir, Box::new(MockRegistry));
        let result = pm.install("yama", Some("not-a-version"));
        assert!(matches!(result, Err(YamaError::Resolution(_))));
    }

    #[test]
    fn test_package_not_found() {
        let dir = tempdir().unwrap();
        let install_dir = dir.path().join("packages");
        let mut pm = PackageManager::new(install_dir, Box::new(MockRegistry));
        let result = pm.install("non-existent-package", None);
        assert!(matches!(result, Err(YamaError::PackageNotFound(_))));
    }

    #[test]
    fn test_load_save_state() {
        let dir = tempdir().unwrap();
        let install_dir = dir.path().join("packages");
        let zip_path = dir.path().join("test.zip");
        create_dummy_zip(&zip_path);

        let registry_data = vec![
            PackageMetadata {
                name: "p1".to_string(),
                version: "1.0.0".to_string(),
                dependencies: vec![],
                url: format!("file://{}", zip_path.to_str().unwrap()),
                checksum: "h1".to_string(),
            }
        ];

        {
            let mut pm = PackageManager::new(install_dir.clone(), Box::new(LocalRegistry { packages: registry_data.clone() }));
            pm.install("p1", None).unwrap();
        }

        let mut pm2 = PackageManager::new(install_dir, Box::new(LocalRegistry { packages: registry_data }));
        pm2.load_state().unwrap();
        assert!(pm2.list_installed().contains(&"p1".to_string()));
    }

    #[test]
    fn test_remove_non_existent() {
        let dir = tempdir().unwrap();
        let install_dir = dir.path().join("packages");
        let mut pm = PackageManager::new(install_dir, Box::new(MockRegistry));
        let result = pm.remove("ghost");
        assert!(matches!(result, Err(YamaError::PackageNotFound(_))));
    }
}
