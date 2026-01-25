use crate::import::*;    

impl LibraryManager {
    pub fn process_package_json(dependencies: &Vec<DependencyInfo>) -> Result<Vec<LibraryMetadata>, LibraryError> {
        let mut metadata_list = Vec::new();
        
        for dep in dependencies {
            let package_json_path = dep.path.join("package.json");
            
            if !package_json_path.exists() {
                return Err(LibraryError::MissingPackageJson(dep.path.clone()));
            }
            
            let content = fs::read_to_string(&package_json_path)
                .map_err(|e| LibraryError::FileReadError(package_json_path.clone(), e.to_string()))?;
            let package_json: PackageJson = serde_json::from_str(&content)
                .map_err(|e| LibraryError::JsonParseError(package_json_path.clone(), e.to_string()))?;

             
            let mut verified_scripts = Vec::new();
            let src_dir = dep.path.join("src");
            if src_dir.exists() && src_dir.is_dir() {
                Self::collect_source_files(&src_dir, &mut verified_scripts)?;
            }
            
             
            let mut verified_syntax = Vec::new();
            let syntax_dir = dep.path.join("syntax");
            if syntax_dir.exists() && syntax_dir.is_dir() {
                Self::collect_syntax_files(&syntax_dir, &mut verified_syntax)?;
            }

             
            let mut verified_errors = Vec::new();
            let error_dir = dep.path.join("syntax");
            if error_dir.exists() && error_dir.is_dir() {
                Self::collect_error_files(&error_dir, &mut verified_errors)?;
            }
            
            metadata_list.push(LibraryMetadata {
                name: package_json.information.name.clone(),
                version: package_json.information.version.clone(),
                publisher: package_json.information.pulicher.clone(),
                path: dep.path.clone(),
                package_json: package_json.clone(),
                verified_scripts,
                verified_syntax,
                verified_errors,
                includes: package_json.include.clang.clone(),
            });
        }
        
        Ok(metadata_list)
    }

     
    fn collect_source_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), LibraryError> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = fs::read_dir(dir)
            .map_err(|e| LibraryError::FileReadError(dir.to_path_buf(), e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| LibraryError::FileReadError(dir.to_path_buf(), e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                 
                Self::collect_source_files(&path, files)?;
            } else if let Some(ext) = path.extension() {
                 
                if ext == "vix" || ext == "x" || ext == "c" || ext == "cpp" || ext == "ll" {
                    files.push(path);
                }
            }
        }

        Ok(())
    }

}