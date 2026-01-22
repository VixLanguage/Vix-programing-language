use crate::import::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetOS {
    Windows,
    Linux,
    MacOS,
    FreeBSD,
    Unknown,
}

impl TargetOS {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            TargetOS::Windows
        } else if cfg!(target_os = "linux") {
            TargetOS::Linux
        } else if cfg!(target_os = "macos") {
            TargetOS::MacOS
        } else if cfg!(target_os = "freebsd") {
            TargetOS::FreeBSD
        } else {
            TargetOS::Unknown
        }
    }

    pub fn executable_extension(&self) -> &'static str {
        match self {
            TargetOS::Windows => ".exe",
            _ => "",
        }
    }

    pub fn object_extension(&self) -> &'static str {
        match self {
            TargetOS::Windows => ".obj",
            _ => ".o",
        }
    }

    pub fn executable_prefix(&self) -> &'static str {
        match self {
            TargetOS::Windows => "",
            _ => "./",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "windows" | "win" => Some(TargetOS::Windows),
            "linux" => Some(TargetOS::Linux),
            "macos" | "mac" | "darwin" => Some(TargetOS::MacOS),
            "freebsd" => Some(TargetOS::FreeBSD),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TargetOS::Windows => "Windows",
            TargetOS::Linux => "Linux",
            TargetOS::MacOS => "macOS",
            TargetOS::FreeBSD => "FreeBSD",
            TargetOS::Unknown => "Unknown",
        }
    }
}

pub struct Clang;

impl Clang {
    fn create_cfg_stub() -> Result<PathBuf, String> {
        let stub_c = r#"
unsigned int __guard_eh_cont_count = 0;
void* __guard_eh_cont_table = 0;
"#;

        let stub_path = Path::new("cfg_stub.c");
        let obj_path = PathBuf::from("cfg_stub.obj");
        let _ = fs::remove_file(stub_path);
        let _ = fs::remove_file(&obj_path);

        fs::write(stub_path, stub_c).map_err(|e| format!("Failed to write CFG stub: {}", e))?;

        let mut cmd = Command::new("clang");
        cmd.arg("-c")
            .arg(stub_path)
            .arg("-o")
            .arg(&obj_path)
            .arg("-O2");

        let output = cmd.output().map_err(|e| format!("Failed to execute clang for CFG stub: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!(
                "CFG stub compilation failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
                stdout, stderr
            ));
        }

        Ok(obj_path)
    }

    pub fn compile_to_object(
        c_code: &str,
        output_path: &Path,
        target_os: Option<TargetOS>,
    ) -> Result<(), String> {
        let mut cmd = Command::new("clang");
        let target = target_os.unwrap_or_else(TargetOS::current);
        let obj_path = if output_path.extension().is_none() {
            output_path.with_extension(target.object_extension().trim_start_matches('.'))
        } else {
            output_path.to_path_buf()
        };
        let c_path = Path::new("output.c");

        fs::write(c_path, c_code).map_err(|e| format!("Failed to write C source: {}", e))?;

        cmd.arg("-c")
            .arg(c_path)
            .arg("-o")
            .arg(&obj_path)
            .arg("-O2")
            .arg("-std=c17")
            .arg("-Wall")
            .arg("-Wextra");

        if target == TargetOS::Windows {
            cmd.arg("-D_CRT_SECURE_NO_WARNINGS");
        }

        let output = cmd.output().map_err(|e| format!("Failed to execute clang: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!("Object compilation failed:\nSTDOUT:\n{}\nSTDERR:\n{}\n", stdout, stderr));
        }

        Ok(())
    }

    pub fn link_executable(
        object_files: &[&Path],
        output_name: &str,
        extra_libs: &[String],
        target_os: Option<TargetOS>,
    ) -> Result<(), String> {
        let mut cmd = Command::new("clang");
        let target = target_os.unwrap_or_else(TargetOS::current);
        let exe_path = format!("{}{}", output_name, target.executable_extension());

        let cfg_stub = if target == TargetOS::Windows {
            Some(Self::create_cfg_stub()?)
        } else {
            None
        };

        if let Some(ref stub) = cfg_stub {
            cmd.arg(stub);
        }

        for obj in object_files {
            cmd.arg(obj);
        }

        cmd.arg("-o").arg(&exe_path);

        Self::add_platform_specific_args(&mut cmd, target);

        for lib in extra_libs {
            cmd.arg(format!("-l{}", lib));
        }

        let output = cmd.output().map_err(|e| format!("Linking failed: {}", e))?;

        if let Some(stub) = cfg_stub {
            let _ = fs::remove_file(stub);
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Linking failed:\n{}", stderr));
        }

        println!("   {} Executable linked: {}", "success:".green(), exe_path);
        Ok(())
    }

    pub fn generate_and_compile(
        c_code: &str,
        output_name: &str,
        extra_libs: &[String],
        target_os: Option<TargetOS>,
    ) -> Result<(), String> {
        let target = target_os.unwrap_or_else(TargetOS::current);
        let exe_path = format!("{}{}", output_name, target.executable_extension());
        let c_path = Path::new("output.c");

        let cfg_stub = if target == TargetOS::Windows {
            Some(Self::create_cfg_stub()?)
        } else {
            None
        };

        fs::write(c_path, c_code).map_err(|e| format!("Failed to write C source: {}", e))?;

        let mut cmd = Command::new("clang");

        if let Some(ref stub) = cfg_stub {
            cmd.arg(stub);
        }

        cmd.arg(c_path)
            .arg("-o")
            .arg(&exe_path)
            .arg("-O2")
            .arg("-std=c17")
            .arg("-Wall")
            .arg("-Wextra");

        if target == TargetOS::Windows {
            cmd.arg("-D_CRT_SECURE_NO_WARNINGS");
        }

        Self::add_platform_specific_args(&mut cmd, target);

        for lib in extra_libs {
            cmd.arg(format!("-l{}", lib));
        }

        let output = cmd.output().map_err(|e| format!("Clang failed: {}", e))?;

        if let Some(stub) = cfg_stub {
            let _ = fs::remove_file(stub);
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!("Compilation failed:\nSTDOUT:\n{}\nSTDERR:\n{}\n", stdout, stderr));
        }

        println!("   {} Executable built: {}", "✓".green(), exe_path);
        Ok(())
    }

    pub fn add_platform_specific_args(cmd: &mut Command, target_os: TargetOS) {
        match target_os {
            TargetOS::Windows => {
                cmd.args([
                    "-Xlinker", "/SUBSYSTEM:CONSOLE",
                    "-lmsvcrt",
                    "-lvcruntime",
                    "-lucrt",
                    "-luser32", "-lgdi32", "-lkernel32", "-ladvapi32",
                    "-lshell32", "-lole32", "-loleaut32", "-luuid", "-lws2_32"
                ]);
            }
            TargetOS::Linux => {
                cmd.args(["-lpthread", "-ldl", "-lm"]);
            }
            TargetOS::MacOS => {
                cmd.args([
                    "-framework", "CoreFoundation",
                    "-framework", "Security",
                    "-lpthread", "-lm"
                ]);
            }
            TargetOS::FreeBSD => {
                cmd.args(["-lpthread", "-lm"]);
            }
            TargetOS::Unknown => {}
        }
    }

    pub fn get_msvc_env() -> Vec<(&'static str, String)> {
        vec![]
    }

    pub fn run_executable(exe_name: &str, target_os: Option<TargetOS>) -> Result<(), String> {
        let target = target_os.unwrap_or_else(TargetOS::current);
        let exe_path = format!("{}{}{}", target.executable_prefix(), exe_name,target.executable_extension());
        let output = Command::new(&exe_path).output().map_err(|e| format!("Failed to run {}: {}", exe_path, e))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stdout.is_empty() {
            print!("{}", stdout);
        }

        if !stderr.is_empty() {
            eprint!("{}", stderr);
        }

        let exit_code = output.status.code().unwrap_or(-1);
        if exit_code != 0 {
            println!("\n{} Program exited with code: {}", "Error:".red(), exit_code);
            return Err(format!("Program failed with exit code {}", exit_code));
        } else {
            println!("{} Compilion compiled successfuly", "success:".green());
        }

        Ok(())
    }

    pub fn compile_with_libraries(
        source: &str,
        output_name: &str,
        target_os: Option<TargetOS>,
    ) -> Result<(), String> {
        println!("   {} Lexing source code...", "→".bright_cyan());
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        if !lexer.errors.is_empty() {
            eprintln!("   {} Lexer errors:", "Error:".red());
            for error in &lexer.errors {
                eprintln!("      {}", error.message);
            }
            return Err("Lexer failed".to_string());
        }

        println!("   {} Parsing tokens...", "→".bright_cyan());
        let parser = Parser::new(tokens, source.to_string(), lexer.spans.clone());
        let (program, structs, enums, externs, _, _, _, impls, _, _, import_decls) = parser.parse();

        println!("   {} Processing imports...", "→".bright_cyan());
        let footprint_packs = LibraryManager::process_imports_from_decls(&import_decls, target_os)?;
        
        LibraryManager::validate_imports(&import_decls, &footprint_packs).ok();

        let mut library_includes = Vec::new();
        let mut library_functions = Vec::new();
        let mut library_objects = Vec::new();
        
        for pack in &footprint_packs {
            println!("   {} Linking library: {} v{}", "→".bright_cyan(), pack.name, pack.version);
            
            for include in &pack.includes {
                if !library_includes.contains(include) {
                    library_includes.push(include.clone());
                }
            }
            
            for sig in &pack.function_signatures {
                println!("      {} {}", "→".bright_black(), sig.name);
                library_functions.push(sig.clone());
            }
            
            let lib_path = PathBuf::from(&pack.source_library);
            if lib_path.exists() {
                println!("      {} Object file: {}", "✓".green(), lib_path.display());
                library_objects.push(lib_path);
            } else {
                eprintln!("      {} Object file not found: {}", "Error:".red(), lib_path.display());
                return Err(format!("Library object file not found: {}", lib_path.display()));
            }
        }

        println!("   {} Generating C code...", "→".bright_cyan());
        let arch = ArchConfig::x86_64();
        let mut codegen = Codegen::new(arch, source.to_string(), "main.vix".to_string());
        
        codegen.set_import_context(&import_decls);
        
        let c_code = codegen.codegen_program_full(
            &program,
            &structs,
            &enums,
            &impls,
            &externs,
            &library_includes,
            &library_functions,
        )?;

        println!("   {} Compiling to object file...", "→".bright_cyan());
        let main_obj = PathBuf::from("main.o");
        Clang::compile_to_object(&c_code, &main_obj, target_os)
            .map_err(|e| format!("Failed to compile: {}", e))?;

        println!("   {} Linking executable...", "→".bright_cyan());
        let mut all_objects: Vec<&Path> = vec![main_obj.as_path()];
        for lib_obj in &library_objects {
            all_objects.push(lib_obj.as_path());
        }
        
        Clang::link_executable(
            &all_objects,
            output_name,
            &[],
            target_os,
        )?;
        
        let _ = fs::remove_file("main.o");
        let _ = fs::remove_file("output.c");
        
        println!("   {} Build complete!", "✓".green());
        Ok(())
    }
}