use Vix::import::*;
use std::env;
use std::fs;
use std::path::Path;

const VERSION: &str = "Alpha 3.0v";

fn print_help() {
    println!("{}", "Vix Compiler".bright_cyan().bold());
    println!("Usage: vix [command] [options]");
    println!();
    println!("Commands:");
    println!("  run [--target OS]     Compile and run the program");
    println!("  build [--target OS]   Compile the program without running");
    println!("  path                  Show the Vix installation directory");
    println!("  version               Show the version information");
    println!("  help                  Show this help message");
    println!();
    println!("Options:");
    println!("  --debug               Enable debug output");
    println!("  --target <OS>         Target operating system (windows, linux, macos, freebsd)");
    println!("  --output <name>       Output executable name (default: program)");
    println!();
    println!("Examples:");
    println!("  vix run                      # Compile and run for current OS");
    println!("  vix run --target windows     # Compile for Windows");
    println!("  vix build --target linux     # Build for Linux");
    println!("  vix run --debug              # Run with debug output");
    println!("  vix path                     # Show installation directory");
    println!("  vix version                  # Show version");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let command = if args.len() > 1 {
        args[1].as_str()
    } else {
        "run"
    };

    if command == "version" || command == "--version" || command == "-v" {
        println!("Vix Compiler {}", VERSION);
        return;
    }

    if command == "help" || command == "--help" || command == "-h" {
        print_help();
        return;
    }

    if command == "path" {
        match env::current_exe() {
            Ok(exe_path) => {
                if let Some(parent) = exe_path.parent() {
                    if parent.file_name().and_then(|n| n.to_str()) == Some("bin") {
                        if let Some(vix_root) = parent.parent() {
                            println!("{}", vix_root.display());
                        } else {
                            println!("{}", parent.display());
                        }
                    } else {
                        println!("{}", parent.display());
                    }
                } else {
                    eprintln!("Error: Could not determine parent directory");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error: Could not get executable path: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    let debug_mode = args.contains(&"--debug".to_string());
    let should_run = command == "run";

    let target_os = if let Some(pos) = args.iter().position(|arg| arg == "--target") {
        if let Some(os_str) = args.get(pos + 1) {
            match TargetOS::from_string(os_str) {
                Some(os) => Some(os),
                None => {
                    eprintln!("Error: Unknown target OS '{}'. Valid options: windows, linux, macos, freebsd", os_str);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Error: --target flag requires an OS name");
            std::process::exit(1);
        }
    } else {
        None
    };

    let output_name = if let Some(pos) = args.iter().position(|arg| arg == "--output") {
        args.get(pos + 1).map(|s| s.as_str()).unwrap_or("program")
    } else {
        "program"
    };

    let current_os = TargetOS::current();
    let target = target_os.unwrap_or(current_os);
    let src_dir = Path::new("src");
    
    if !src_dir.exists() {
        eprintln!("{} src/ directory not found!", "Error:".red());
        std::process::exit(1);
    }

    let mut source_files = Vec::new();
    let src_path = fs::canonicalize(src_dir).unwrap_or(src_dir.to_path_buf());
    println!("DEBUG: Searching for sources in: {:?}", src_path);

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("vix") {
                source_files.push(path);
            }
        }
    }

    if source_files.is_empty() {
        eprintln!("{} No .vix files found in src/", "Error:".red());
        std::process::exit(1);
    }

    let mut all_import_decls = Vec::new();
    for source_file in &source_files {
        let source_code = match fs::read_to_string(source_file) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("   {} Failed to read {}: {}", "Error:".red(), source_file.display(), e);
                std::process::exit(1);
            }
        };

        let mut lexer = Lexer::new(&source_code);
        let tokens = lexer.tokenize();

        if !lexer.errors.is_empty() {
            for err in &lexer.errors {
                eprintln!("   {} in {}: {}", "Error:".red(), source_file.display(), err.message);
            }
            std::process::exit(1);
        }

        let parser = Parser::new(tokens, source_code, lexer.spans);
        let (_, _, _, _, _, _, _, _, _, _, import_decls) = parser.parse();
        all_import_decls.extend(import_decls);
    }

    let footprint_packs = if !all_import_decls.is_empty() {
        match LibraryManager::process_imports_from_decls(&all_import_decls, Some(target)) {
            Ok(packs) => {
                if let Err(e) = LibraryManager::validate_imports(&all_import_decls, &packs) {
                    eprintln!("{} Import validation failed: {:?}", "Error:".red(), e);
                    std::process::exit(1);
                }
                packs
            }
            Err(e) => {
                eprintln!("{} Library processing failed: {:?}", "Error:".red(), e);
                std::process::exit(1);
            }
        }
    } else {
        Vec::new()
    };

    let arch = ArchConfig::x86_64();

    let combined_source = String::new();
    let main_filename = source_files.first().map(|p| p.display().to_string()).unwrap_or_else(|| "main.vix".to_string());

    let mut combined_source_code = String::new();
    for source_file in &source_files {
        let source_code = fs::read_to_string(source_file).unwrap();
        combined_source_code.push_str(&source_code);
        combined_source_code.push_str("\n\n");
    }

    let mut lexer = Lexer::new(&combined_source_code);
    let tokens = lexer.tokenize();
    let parser = Parser::new(tokens, combined_source_code.clone(), lexer.spans.clone());
    let (program, all_structs, all_enums, all_externs, _, _, _, all_impls, _, _, _) = parser.parse();
    let all_functions = program.functions;
    let all_constants = program.constants;
    let combined_source = combined_source_code;
    let all_modules = program.modules.clone();

    if all_functions.is_empty() {
        eprintln!("{} No functions found to compile", "Error:".red());
        std::process::exit(1);
    }
    
    let mut all_library_includes = Vec::new();
    let mut all_library_functions = Vec::new();
    
    for pack in &footprint_packs {
        all_library_includes.extend(pack.includes.clone());
        all_library_functions.extend(pack.function_signatures.clone());
    }

    if debug_mode {
        println!("   {} Extracted {} function signatures", "→".bright_black(), all_library_functions.len());
        for sig in &all_library_functions {
            println!("      {} {}({}) -> {}", "→".bright_black(), sig.name, 
                sig.parameters.iter().map(|(n, t)| format!("{}: {}", n, t)).collect::<Vec<_>>().join(", "),
                sig.return_type);
        }
    }

    let mut codegen = Codegen::new(arch, combined_source, main_filename);

    for func_sig in &all_library_functions {
        let params_str = if func_sig.parameters.is_empty() {
            "void".to_string()
        } else {
            func_sig.parameters.iter()
                .map(|(name, ty)| format!("{} {}", ty, name))
                .collect::<Vec<_>>()
                .join(", ")
        };
        
        let decl = format!("{} {}({});", 
            func_sig.return_type, 
            func_sig.name, 
            params_str
        );
        codegen.ir.add_forward_decl(decl.clone());
        
        if debug_mode {
            println!("   {} Added forward declaration: {}", "success:".bright_green(), decl);
        }
    }
    
    let program = Program { functions: all_functions, constants: all_constants, modules: all_modules };

    let c_code = match codegen.codegen_program_full(
        &program, 
        &all_structs, 
        &all_enums, 
        &all_impls, 
        &all_externs, 
        &all_library_includes,
        &all_library_functions
    ) {
        Ok(code) => code,
        Err(_) => {
            eprintln!("{} Code generation failed", "Error:".red());
            std::process::exit(1);
        }
    };

    let linked_libs = codegen.get_linked_libraries();
    if debug_mode && !linked_libs.is_empty() {
        println!("   {} Libraries to link: {:?}", "success:".bright_green(), linked_libs);
    }

    println!("   {} Compiling main program to object file", "success:".bright_green());
    
    let main_obj = Path::new("main.o");
    match Clang::compile_to_object(&c_code, main_obj, Some(target)) {
        Ok(_) => println!("   {} Main object file created: {}", "success:".green(), main_obj.display()),
        Err(e) => {
            eprintln!("{} Main compilation failed: {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }

    let mut object_files: Vec<&Path> = vec![main_obj];
    for pack in &footprint_packs {
        let lib_path = Path::new(&pack.source_library);
        if lib_path.exists() {
            object_files.push(lib_path);
            if debug_mode {
                println!("   {} Linking library: {}", "→".bright_black(), lib_path.display());
            }
        } else {
            eprintln!("{} Library object file not found: {}", "Warning:".yellow(), lib_path.display());
        }
    }

    println!("   {} Linking executable with {} object file(s)", "→".bright_cyan(), object_files.len());
    match Clang::link_executable(&object_files, output_name, linked_libs, Some(target)) {
        Ok(_) => {
            if should_run {
                if target != current_os {
                    println!("\n{} Cannot run executable compiled for {} on {}", "Warning:".yellow(), target.display_name(), current_os.display_name());
                } else if let Err(e) = Clang::run_executable(output_name, Some(target)) {
                    eprintln!("\n{} Runtime error: {}", "Error:".red(), e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("{} Linking failed: {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}