use crate::import::*;

impl From<toml::de::Error> for LibraryError {
    fn from(err: toml::de::Error) -> Self {
        LibraryError::ParseError(err.to_string())
    }
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LibraryError::MissingField(field) => write!(f, "Missing field: {}", field),
            LibraryError::MissingSection(section) => write!(f, "Missing section: {}", section),
            LibraryError::PathNotFound(path) => write!(f, "Path not found: {}", path),
            LibraryError::InvalidVersion(ver) => write!(f, "Invalid version: {}", ver),
            LibraryError::MissingLibraries(libs) => write!(f, "Missing libraries: {:?}", libs),
            LibraryError::ParseError(err) => write!(f, "Parse error: {}", err),
            LibraryError::MissingPackageJson(path) => write!(f, "Missing package.json: {:?}", path),
            LibraryError::FileReadError(path, msg) => write!(f, "File read error at {:?}: {}", path, msg),
            LibraryError::JsonParseError(path, msg) => write!(f, "JSON parse error at {:?}: {}", path, msg),
            LibraryError::MissingLibraryFiles(lib, files) => write!(f, "Missing files in {}: {:?}", lib, files),
        }
    }
}


impl LibraryManager {
    pub fn print_missing_files_error(lib_name: &str, lib_path: &PathBuf, missing: &Vec<String>) {
        eprintln!("[Error]: Missing files in library '{}'!", lib_name);
        eprintln!("|");
        eprintln!("| Library path: {:?}", lib_path);
        eprintln!("|");
        eprintln!("| Missing files:");
        
        for file in missing {
            eprintln!("|   - {}", file);
        }
        
        eprintln!("|");
        eprintln!(" help:");
        eprintln!("  Reinstall the library or check if the package is corrupted");
        eprintln!("|-----");
    }

    pub fn print_missing_libraries_error(missing: &Vec<(String, String)>) {
        eprintln!("[Error]: Libraries not found!");
        eprintln!("|");
        
        for (name, version) in missing {
            eprintln!("| {} = \"{}\"", name, version);
        }
        
        eprintln!("|");
        eprintln!(" help:");
        
        for (name, version) in missing {
            eprintln!("  install \"{}\" using \"package install {} -{}\"", 
                     name, name, version);
        }
        
        eprintln!("|-----");
    }
}