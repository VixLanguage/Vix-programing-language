use crate::import::*;

impl LibraryManager {
    pub fn load_libraries(package_content: &str) -> Result<(PackageInfo, Vec<LibraryMetadata>), LibraryError> {
        let package_info = Self::parse_package(package_content)?;
        let libraries = Self::load_library_metadata(&package_info)?;
        
        Ok((package_info, libraries))
    }
    
    pub fn load_library_metadata(package_info: &PackageInfo) -> Result<Vec<LibraryMetadata>, LibraryError> {
        let mut missing_dirs = Vec::new();
        
        for dep in &package_info.dependencies {
            if !dep.path.exists() {
                missing_dirs.push((dep.name.clone(), dep.version.clone()));
            }
        }
        
        if !missing_dirs.is_empty() {
            Self::print_missing_libraries_error(&missing_dirs);
            return Err(LibraryError::MissingLibraries(missing_dirs));
        }
        
        Self::process_package_json(&package_info.dependencies)
    }
}