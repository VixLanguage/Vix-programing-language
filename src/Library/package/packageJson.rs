use crate::import::*;    
    

impl LibraryManager {
    pub fn process_package_json(dependencies: &Vec<DependencyInfo>) -> Result<Vec<LibraryMetadata>, LibraryError> {
        let mut metadata_list = Vec::new();
        
        for dep in dependencies {
            let package_json_path = dep.path.join("package.json");
            
            if !package_json_path.exists() {
                return Err(LibraryError::MissingPackageJson(dep.path.clone()));
            }
            
            let mut verified_scripts = Vec::new();
            let mut verified_syntax = Vec::new();
            let mut verified_errors = Vec::new();
            let mut missing_files = Vec::new();
            
            let content = fs::read_to_string(&package_json_path).map_err(|e| LibraryError::FileReadError(package_json_path.clone(), e.to_string()))?;
            let package_json: PackageJson = serde_json::from_str(&content).map_err(|e| LibraryError::JsonParseError(package_json_path.clone(), e.to_string()))?;

            
            for script in &package_json.src.scripts {
                let script_path = dep.path.join("src").join(script);
                if script_path.exists() {
                    verified_scripts.push(script_path);
                } else {
                    missing_files.push(format!("src/{}", script));
                }
            }
            
            for syntax_file in &package_json.syntax.syntax {
                let syntax_path = dep.path.join("syntax").join(syntax_file);
                if syntax_path.exists() {
                    verified_syntax.push(syntax_path);
                } else {
                    missing_files.push(format!("syntax/{}", syntax_file));
                }
            }

            for error_file in &package_json.syntax.error {
                let error_path = dep.path.join("syntax").join(error_file);
                if error_path.exists() {
                    verified_errors.push(error_path);
                } else {
                    missing_files.push(format!("syntax/{}", error_file));
                }
            }
            
            if !missing_files.is_empty() {
                 
                 
                return Err(LibraryError::MissingLibraryFiles(dep.name.clone(), missing_files));
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
}