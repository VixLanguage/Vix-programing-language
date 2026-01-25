use crate::import::*;

impl LibraryManager {
    pub fn parse_package(package_content: &str) -> Result<PackageInfo, LibraryError> {
        let mut missing_libs = Vec::new();
        let mut found_dependencies = Vec::new();
        
        let config: toml::Value = toml::from_str(package_content)?;
        let name = config["Information"]["Name"].as_str().ok_or(LibraryError::MissingField("Name"))?.to_string();
        let version = config["Information"]["Version"].as_str().ok_or(LibraryError::MissingField("Version"))?.to_string();
        let lib_path_str = config["Library"]["path"].as_str().ok_or(LibraryError::MissingField("path"))?;
        let lib_path = PathBuf::from(lib_path_str); 
        let packages = config["package"].as_table().ok_or(LibraryError::MissingSection("package"))?;
        
        
        if !lib_path.exists() {
            return Err(LibraryError::PathNotFound(lib_path.to_string_lossy().to_string()));
        }
        
        for (pkg_name, pkg_version) in packages {
            let version_str = pkg_version.as_str()
                .ok_or_else(|| LibraryError::InvalidVersion(pkg_name.clone()))?;
            
            let pkg_path = lib_path.join(format!("{}-{}", pkg_name, version_str));
        
            if !pkg_path.exists() {
                missing_libs.push((pkg_name.clone(), version_str.to_string()));
            } else {
                found_dependencies.push(DependencyInfo {
                    name: pkg_name.clone(),
                    version: version_str.to_string(),
                    path: pkg_path,
                });
            }
        }
        
        if !missing_libs.is_empty() {
            Self::print_missing_libraries_error(&missing_libs);
            return Err(LibraryError::MissingLibraries(missing_libs));
        }
        
        println!("Success: All dependencies found for '{}' v{}", name, version);
        
        Ok(PackageInfo {
            project_name: name,
            project_version: version,
            library_path: lib_path,
            dependencies: found_dependencies,
        })
    }
}