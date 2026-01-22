use crate::import::*;
impl std::error::Error for LibraryError {}

pub struct LibraryManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintPack {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub classes: Vec<String>,
    pub function_signatures: Vec<FunctionSignature>,
    pub includes: Vec<String>,
    pub functions: Vec<String>,
    pub source_library: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<(String, String)>,
    pub abi: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageJson {
    #[serde(rename = "Information")]
    pub information: PackageInformation,
    #[serde(rename = "include", default)]
    pub include: IncludeSection,
    pub src: SourceFiles,
    pub syntax: SyntaxFiles,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct IncludeSection {
    #[serde(rename = "Clang", default)]
    pub clang: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageInformation {
    pub name: String,
    pub version: String,
    pub pulicher: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceFiles {
    pub scripts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyntaxFiles {
    pub syntax: Vec<String>,
    pub error: Vec<String>,
}

pub struct LibraryMetadata {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub path: PathBuf,
    pub package_json: PackageJson,
    pub verified_scripts: Vec<PathBuf>,
    pub verified_syntax: Vec<PathBuf>,
    pub verified_errors: Vec<PathBuf>,
    pub includes: Vec<String>,
}

pub struct PackageInfo {
    pub project_name: String,
    pub project_version: String,
    pub library_path: PathBuf,
    pub dependencies: Vec<DependencyInfo>,
}

pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum LibraryError {
    MissingField(&'static str),
    MissingSection(&'static str),
    PathNotFound(String),
    InvalidVersion(String),
    MissingLibraries(Vec<(String, String)>),
    ParseError(String),
    MissingPackageJson(PathBuf),
    FileReadError(PathBuf, String),
    JsonParseError(PathBuf, String),
    MissingLibraryFiles(String, Vec<String>),
}

impl LibraryManager {
    pub fn get_vix_path() -> Result<PathBuf, LibraryError> {
        Ok(PathBuf::from("C:/Users/Client/Vix"))
    }

    pub fn load_footprint_libraries() -> Result<Vec<FootprintPack>, LibraryError> {
        let footprint_path = Self::get_vix_path()?.join("footprint").join("libraries.pack");

        if !footprint_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&footprint_path).map_err(|e| LibraryError::FileReadError(footprint_path.clone(), e.to_string()))?;
        let libraries: Vec<FootprintPack> = serde_json::from_str(&content).map_err(|e| LibraryError::JsonParseError(footprint_path, e.to_string()))?;

        Ok(libraries)
    }

    pub fn save_footprint_libraries(libraries: &Vec<FootprintPack>) -> Result<(), LibraryError> {
        let footprint_dir = Self::get_vix_path()?.join("footprint");
        let pack_path = footprint_dir.join("libraries.pack");
        let content = serde_json::to_string_pretty(libraries).map_err(|e| LibraryError::JsonParseError(pack_path.clone(), e.to_string()))?;

        fs::create_dir_all(&footprint_dir).map_err(|e| LibraryError::FileReadError(footprint_dir.clone(), e.to_string()))?;
        fs::write(&pack_path, content).map_err(|e| LibraryError::FileReadError(pack_path, e.to_string()))?;

        Ok(())
    }

    pub fn compile_library(
        &self,
        lib_metadata: &LibraryMetadata,
        target_os: Option<TargetOS>,
    ) -> Result<PathBuf, LibraryError> {
        let vix_path = Self::get_vix_path()?;
        let binary_dir = vix_path.join("library").join("binary");
        let output_name = format!("{}-{}", lib_metadata.name, lib_metadata.version);
        let binary_path = binary_dir.join(format!("{}.bin", output_name));

        fs::create_dir_all(&binary_dir).map_err(|e| LibraryError::FileReadError(binary_dir.clone(), e.to_string()))?;

        if binary_path.exists() {
            return Ok(binary_path);
        }


        let mut all_source = String::new();
        for script_path in &lib_metadata.verified_scripts {
            let ext = script_path.extension().and_then(|s| s.to_str()).unwrap_or("");

            match ext {
                "x" | "vix" => {
                    let source = fs::read_to_string(script_path)
                        .map_err(|e| LibraryError::FileReadError(script_path.clone(), e.to_string()))?;
                    all_source.push_str(&source);
                    all_source.push_str("\n\n");
                }
                "c" | "cpp" => {
                    return Self::compile_c_cpp_library(script_path, &binary_path, ext, target_os);
                }
                "ll" => {
                    return Self::compile_llvm_library(script_path, &binary_path, target_os);
                }
                _ => {}
            }
        }

        if !all_source.is_empty() {
            Self::compile_vix_library(&all_source, &binary_path, target_os, &lib_metadata.includes)?;
        }

        println!("   {} Library compiled successfully", "✓".green());
        Ok(binary_path)
    }


    fn compile_vix_library(
        source: &str,
        output_path: &PathBuf,
        target_os: Option<TargetOS>,
        library_includes: &[String],  
    ) -> Result<(), LibraryError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        if !lexer.errors.is_empty() {
            eprintln!("   {} Lexer errors:", "Error:".red());
            for error in &lexer.errors {
                eprintln!("      {}", error.message);
            }
            return Err(LibraryError::ParseError("Lexer failed".to_string()));
        }

        let parser = Parser::new(tokens, source.to_string(), lexer.spans.clone());
        let (program, structs, enums, externs, _, _, _, impls, _, _, _) = parser.parse();
        let arch = ArchConfig::x86_64();
        let mut codegen = Codegen::new(arch, source.to_string(), "library".to_string());
        
         
        let c_code = codegen.codegen_library(
            &program, 
            &structs, 
            &enums, 
            &impls, 
            &externs, 
            library_includes
        ).map_err(|e| LibraryError::ParseError(format!("Codegen failed: {}", e)))?;

        Clang::compile_to_object(&c_code, output_path, target_os)
            .map_err(LibraryError::ParseError)?;

        Ok(())
    }
        
     
     
     
    fn compile_c_cpp_library(
        source_path: &PathBuf,
        output_path: &PathBuf,
        lang: &str,
        _target_os: Option<TargetOS>,
    ) -> Result<PathBuf, LibraryError> {
        let mut cmd = Command::new("clang");
        if lang == "cpp" {
            cmd.arg("-xc++");
        }

         
        cmd.arg("-c")
           .arg(source_path)
           .arg("-o")
           .arg(output_path)
           .arg("-O3")
           .arg("-std=c17");
         

        let output = cmd.output().map_err(|e| LibraryError::ParseError(format!("Clang failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LibraryError::ParseError(format!("Compilation failed: {}", stderr)));
        }

        Ok(output_path.clone())
    }

     
    fn compile_llvm_library(
        source_path: &PathBuf,
        output_path: &PathBuf,
        _target_os: Option<TargetOS>,
    ) -> Result<PathBuf, LibraryError> {
        let obj_path = output_path.with_extension("o");

         
        let status = Command::new("llc")
            .arg(source_path)
            .arg("-filetype=obj")
            .arg("-o")
            .arg(&obj_path)
            .status()
            .map_err(|e| LibraryError::ParseError(format!("LLC failed: {}", e)))?;

        if !status.success() {
            return Err(LibraryError::ParseError("LLC compilation failed".to_string()));
        }

         
        fs::rename(&obj_path, output_path)
            .map_err(|e| LibraryError::FileReadError(output_path.clone(), e.to_string()))?;

        Ok(output_path.clone())
    }

     

    pub fn process_all_imports(
        source_files: &Vec<PathBuf>,
        target_os: Option<TargetOS>,
    ) -> Result<Vec<FootprintPack>, LibraryError> {
        let mut required_libs = HashSet::new();

        for source_file in source_files {
            let source = fs::read_to_string(source_file)
                .map_err(|e| LibraryError::FileReadError(source_file.clone(), e.to_string()))?;

            let imports = Self::extract_imports_from_source(&source);
            for (lib_name, _) in imports {
                required_libs.insert(lib_name);
            }
        }

        let mut footprint_packs = Vec::new();
        let vix_path = Self::get_vix_path()?;
        let library_path = vix_path.join("library");

        for lib_name in required_libs {
            let lib_dirs: Vec<_> = fs::read_dir(&library_path)
                .map_err(|e| LibraryError::FileReadError(library_path.clone(), e.to_string()))?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_str().unwrap_or("").starts_with(&lib_name))
                .collect();

            if lib_dirs.is_empty() {
                eprintln!("   {} Library not found: {}", "Error:".red(), lib_name);
                return Err(LibraryError::PathNotFound(lib_name));
            }

            let lib_dir = lib_dirs[0].path();
            let package_json_path = lib_dir.join("package.json");

            if !package_json_path.exists() {
                return Err(LibraryError::MissingPackageJson(lib_dir));
            }

            let content = fs::read_to_string(&package_json_path).map_err(|e| LibraryError::FileReadError(package_json_path.clone(), e.to_string()))?;
            let package_json: PackageJson = serde_json::from_str(&content).map_err(|e| LibraryError::JsonParseError(package_json_path.clone(), e.to_string()))?;
            let mut verified_scripts = Vec::new();
            
            for script in &package_json.src.scripts {
                let script_path = lib_dir.join("src").join(script);
                if script_path.exists() {
                    verified_scripts.push(script_path);
                }
            }

            let lib_metadata = LibraryMetadata {
                name: package_json.information.name.clone(),
                version: package_json.information.version.clone(),
                publisher: package_json.information.pulicher.clone(),
                path: lib_dir,
                package_json: package_json.clone(),
                verified_scripts,
                verified_syntax: Vec::new(),
                verified_errors: Vec::new(),
                includes: package_json.include.clang.clone(),
            };

            let binary_path = Self.compile_library(&lib_metadata, target_os)?;

             
            let (functions, classes) = Self::extract_library_symbols(&lib_metadata)?;
            let function_signatures = Self::extract_function_signatures(&lib_metadata)?;

            footprint_packs.push(FootprintPack {
                name: package_json.information.name,
                version: package_json.information.version,
                publisher: package_json.information.pulicher,
                classes,
                function_signatures,  
                functions,
                includes: package_json.include.clang.clone(),
                source_library: binary_path.to_string_lossy().to_string(),
            });
        }

        Self::save_footprint_libraries(&footprint_packs)?;

        Ok(footprint_packs)
    }

    pub fn extract_imports_from_decls(import_decls: &[ImportDecl]) -> Vec<(String, Option<String>)> {
        let mut imports = Vec::new();

        for decl in import_decls {
            match decl {
                ImportDecl::LibraryImport { name } => {
                    imports.push((name.clone(), None));
                }
                ImportDecl::FileImport { name, from } => {
                    imports.push((from.clone(), Some(name.clone())));
                }
                ImportDecl::WildcardImport { from } => {
                    imports.push((from.clone(), None));
                }
            }
        }

        imports
    }

    pub fn process_imports_from_decls(
        import_decls: &[ImportDecl],
        target_os: Option<TargetOS>,
    ) -> Result<Vec<FootprintPack>, LibraryError> {
        let imports = Self::extract_imports_from_decls(import_decls);
        let mut required_libs = HashSet::new();
        let mut footprint_packs = Vec::new();
        let vix_path = Self::get_vix_path()?;
        let library_path = vix_path.join("library");

        for (lib_name, _symbol) in imports {
            required_libs.insert(lib_name);
        }

        for lib_name in required_libs {
            println!("   {} Processing library: {}", "→".bright_cyan(), lib_name);

            let lib_dirs: Vec<_> = fs::read_dir(&library_path)
                .map_err(|e| LibraryError::FileReadError(library_path.clone(), e.to_string()))?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let dir_name = e.file_name();
                    let dir_str = dir_name.to_str().unwrap_or("");
                    dir_str.starts_with(&lib_name) || dir_str == lib_name
                })
                .collect();

            if lib_dirs.is_empty() {
                eprintln!("   {} Library not found: {}", "Error:".red(), lib_name);
                return Err(LibraryError::PathNotFound(lib_name));
            }

            let lib_dir = lib_dirs[0].path();
            let package_json_path = lib_dir.join("package.json");

            if !package_json_path.exists() {
                eprintln!("   {} Missing package.json for: {}", "Error:".red(), lib_name);
                return Err(LibraryError::MissingPackageJson(lib_dir));
            }

            let content = fs::read_to_string(&package_json_path).map_err(|e| LibraryError::FileReadError(package_json_path.clone(), e.to_string()))?;
            let package_json: PackageJson = serde_json::from_str(&content).map_err(|e| LibraryError::JsonParseError(package_json_path.clone(), e.to_string()))?;

            println!(
                "   {} Found library: {} v{}",
                "success:".green(),
                package_json.information.name,
                package_json.information.version
            );

            let mut verified_scripts = Vec::new();
            let mut missing_files = Vec::new();

            for script in &package_json.src.scripts {
                let script_path = lib_dir.join("src").join(script);
                if script_path.exists() {
                    verified_scripts.push(script_path);
                    println!("      {} {}", "->".bright_black(), script);
                } else {
                    missing_files.push(script.clone());
                }
            }

            if !missing_files.is_empty() {
                eprintln!("   {} Missing source files: {:?}", "Warning:".yellow(), missing_files);
            }

            let mut verified_syntax = Vec::new();
            for syntax_file in &package_json.syntax.syntax {
                let syntax_path = lib_dir.join("syntax").join(syntax_file);
                if syntax_path.exists() {
                    verified_syntax.push(syntax_path);
                }
            }

            let mut verified_errors = Vec::new();
            for error_file in &package_json.syntax.error {
                let error_path = lib_dir.join("syntax").join(error_file);
                if error_path.exists() {
                    verified_errors.push(error_path);
                }
            }

            let lib_metadata = LibraryMetadata {
                name: package_json.information.name.clone(),
                version: package_json.information.version.clone(),
                publisher: package_json.information.pulicher.clone(),
                path: lib_dir,
                package_json: package_json.clone(),
                verified_scripts,
                verified_syntax,
                verified_errors,
                includes: package_json.include.clang.clone(),
            };

            let binary_path = Self.compile_library(&lib_metadata, target_os)?;

            println!("   {} Binary created: {}", "success:".green(), binary_path.display());

             
            let (functions, classes) = Self::extract_library_symbols(&lib_metadata)?;
            let function_signatures = Self::extract_function_signatures(&lib_metadata)?;

            footprint_packs.push(FootprintPack {
                name: package_json.information.name,
                version: package_json.information.version,
                publisher: package_json.information.pulicher,
                classes,
                function_signatures,  
                functions,
                includes: package_json.include.clang.clone(),
                source_library: binary_path.to_string_lossy().to_string(),
            });
        }

        Self::save_footprint_libraries(&footprint_packs)?;
        println!("\n   {} All libraries processed successfully", "✓".green());

        Ok(footprint_packs)
    }

    fn extract_library_symbols(lib_metadata: &LibraryMetadata) -> Result<(Vec<String>, Vec<String>), LibraryError> {
        let mut functions = Vec::new();
        let mut classes = Vec::new();

        for script_path in &lib_metadata.verified_scripts {
            let source = fs::read_to_string(script_path)
                .map_err(|e| LibraryError::FileReadError(script_path.clone(), e.to_string()))?;

            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize();

            let mut i = 0;
            while i < tokens.len() {
                match &tokens[i] {
                    Token::Pub => {
                        i += 1;
                        if i < tokens.len() {
                            match &tokens[i] {
                                Token::Func => {
                                    i += 1;
                                    if i < tokens.len()
                                        && let Token::Identifier(name) = &tokens[i] {
                                            functions.push(name.clone());
                                        }
                                }
                                Token::Struct => {
                                    i += 1;
                                    if i < tokens.len()
                                        && let Token::Identifier(name) = &tokens[i] {
                                            classes.push(name.clone());
                                        }
                                }
                                _ => {}
                            }
                        }
                    }
                    Token::Impl => {
                        i += 1;
                        if i < tokens.len()
                            && let Token::Identifier(name) = &tokens[i]
                                && !classes.contains(name) {
                                    classes.push(name.clone());
                                }
                    }
                    _ => {}
                }
                i += 1;
            }
        }

        Ok((functions, classes))
    }

    
    pub fn extract_imports_from_source(source: &str) -> Vec<(String, Option<String>)> {
        let mut imports = Vec::new();
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        let mut i = 0;
        while i < tokens.len() {
            if tokens[i] == Token::Import {
                i += 1;

                if i < tokens.len()
                    && let Token::Identifier(symbol) = &tokens[i] {
                        let symbol_name = symbol.clone();
                        i += 1;

                        if i < tokens.len() && tokens[i] == Token::From {
                            i += 1;
                            if i < tokens.len()
                                && let Token::Identifier(lib) = &tokens[i] {
                                    imports.push((lib.clone(), Some(symbol_name)));
                                }
                        } else {
                            imports.push((symbol_name, None));
                        }
                    }
            }
            i += 1;
        }

        imports
    }

    pub fn validate_imports(
        import_decls: &[ImportDecl],
        footprint_packs: &[FootprintPack],
    ) -> Result<(), LibraryError> {
        for decl in import_decls {
            match decl {
                ImportDecl::FileImport { name, from } => {
                    let lib = footprint_packs.iter().find(|pack| pack.name == *from).ok_or_else(|| LibraryError::PathNotFound(from.clone()))?;
                    let symbol_exists = lib.functions.contains(name) || lib.classes.contains(name);

                    if !symbol_exists {
                        eprintln!("   {} Symbol '{}' not found in library '{}'", "Warning:".yellow(), name, from);
                        eprintln!("      Available functions: {:?}", lib.functions);
                        eprintln!("      Available classes: {:?}", lib.classes);
                    }
                }
                ImportDecl::LibraryImport { name } => {
                    let exists = footprint_packs.iter().any(|pack| pack.name == *name);
                    if !exists {
                        return Err(LibraryError::PathNotFound(name.clone()));
                    }
                }
                ImportDecl::WildcardImport { from } => {
                    let exists = footprint_packs.iter().any(|pack| pack.name == *from);
                    if !exists {
                        return Err(LibraryError::PathNotFound(from.clone()));
                    }
                }
            }
        }

        Ok(())
    }

        pub fn extract_function_signatures(lib_metadata: &LibraryMetadata) -> Result<Vec<FunctionSignature>, LibraryError> {
        let mut signatures = Vec::new();

        for script_path in &lib_metadata.verified_scripts {
            let source = fs::read_to_string(script_path)
                .map_err(|e| LibraryError::FileReadError(script_path.clone(), e.to_string()))?;

            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize();
            let parser = Parser::new(tokens, source.clone(), lexer.spans.clone());
            let (program, _, _, _, _, _, _, _, _, _, _) = parser.parse();

            println!("      {} Scanning {} functions in library", "→".bright_black(), program.functions.len());
            
             
            for func in &program.functions {
                if func.is_public {
                    let mut registry = TypeRegistry::new();
                    let return_type = func.return_type.to_c_type(&ArchConfig::x86_64(), &mut registry);
                    
                    let parameters: Vec<(String, String)> = func.params.iter()
                        .map(|(name, ty, _modifier)| (name.clone(), ty.to_c_type(&ArchConfig::x86_64(), &mut registry)))
                        .collect();

                    signatures.push(FunctionSignature {
                        name: func.name.clone(),
                        return_type: return_type.clone(),
                        parameters,
                        abi: "c".to_string(),
                    });
                    
                    println!("         {} Public function: {} -> {}", "✓".green(), func.name, return_type);
                }
            }
            
             
            for module in &program.modules {
                if let Stmt::ModuleDef { name: module_name, body, is_public } = module {
                    if !is_public {
                        continue;
                    }
                    
                    println!("      {} Scanning public module: {}", "→".bright_black(), module_name);
                    
                    for stmt in body {
                        if let Stmt::Function(func) = stmt
                            && func.is_public {
                                 
                                let prefixed_name = format!("{}_{}", module_name, func.name);
                                
                                let mut registry = TypeRegistry::new();
                                let return_type = func.return_type.to_c_type(&ArchConfig::x86_64(), &mut registry);
                                
                                let parameters: Vec<(String, String)> = func.params.iter()
                                    .map(|(name, ty, _modifier)| (name.clone(), ty.to_c_type(&ArchConfig::x86_64(), &mut registry)))
                                    .collect();

                                signatures.push(FunctionSignature {
                                    name: prefixed_name.clone(),
                                    return_type: return_type.clone(),
                                    parameters,
                                    abi: "c".to_string(),
                                });
                                
                                println!("         {} Module function: {} -> {}", "✓".green(), prefixed_name, return_type);
                            }
                    }
                }
            }
        }

        Ok(signatures)
    }

}