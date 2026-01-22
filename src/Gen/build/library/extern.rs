use crate::import::*;

impl Codegen {
    fn map_abi_to_calling_convention(&self, abi: &str) -> String {
        match abi.to_lowercase().as_str() {
            "c" => "".to_string(),
            "stdcall" | "std" => "__stdcall".to_string(),
            "cdecl" => "__cdecl".to_string(),
            "fastcall" => "__fastcall".to_string(),
            "vectorcall" => "__vectorcall".to_string(),
            "rust" => "".to_string(), 
            "win64" | "windows" => "__stdcall".to_string(),
            "winapi" => "__stdcall".to_string(),
            "system" => {
                if cfg!(windows) {
                    "__stdcall".to_string()
                } else {
                    "".to_string()
                }
            }
            
            custom => {
                if custom.starts_with("__") {
                    custom.to_string()
                } else {
                    format!("__{}", custom)
                }
            }
        }
    }
    
    fn generate_abi_attributes(&self, abi: &str) -> String {
        match abi.to_lowercase().as_str() {
            "rust" => {
                "#ifdef __cplusplus\nextern \"C\" {\n#endif\n".to_string()
            }
            "c++" | "cpp" => {
                "#ifdef __cplusplus\nextern \"C++\" {\n#endif\n".to_string()
            }
            _ => String::new()
        }
    }

    fn codegen_extern_function(&mut self, abi: &str, func: &ExternFunction, library: Option<&str>) -> Result<(), ()> {
        let loc = self.default_location();
        let calling_convention = self.map_abi_to_calling_convention(abi);
        let abi_attrs = self.generate_abi_attributes(abi);
        let return_type = func.return_type.to_c_type(&self.arch, &mut self.type_registry);
        let mut params_str = Vec::new();
        
        if !abi_attrs.is_empty() {
            self.ir.forward_decls.push_str(&abi_attrs);
        }
        

        for (param_name, param_type) in &func.params {
            let underlying_type = match param_type {
                Type::Const(inner) => inner.as_ref(),
                other => other,
            };
            
            if matches!(underlying_type, Type::Void) {
                self.diagnostics.error(
                    "VoidParameter",
                    &format!("Parameter '{}' in extern function '{}' cannot be void", param_name, func.name),
                    ErrorContext {
                        primary_location: loc.clone(),
                        secondary_locations: vec![],
                        help_message: Some("Extern function parameters must have concrete types.".to_string()),
                        suggestions: vec![
                            format!("Change parameter '{}' to a concrete type", param_name),
                            "Use void* for generic pointers".to_string(),
                        ],
                    }
                );
                return Err(());
            }
            
            let c_type = param_type.to_c_type(&self.arch, &mut self.type_registry);
            params_str.push(format!("{} {}", c_type, param_name));
        }
        
        let extern_decl = 
            if calling_convention.is_empty() {
                format!(
                    "extern {} {}({});\n",
                    return_type,
                    func.name,
                    params_str.join(", ")
                )
            } else {
                format!(
                    "extern {} {} {}({});\n",
                    return_type,
                    calling_convention,
                    func.name,
                    params_str.join(", ")
                )
            };
       
        let _known_abis = ["c", "stdcall", "cdecl", "fastcall", "vectorcall", "rust", "system"];


        
        self.ir.forward_decls.push_str(&extern_decl);
        
        if matches!(abi.to_lowercase().as_str(), "rust" | "c++" | "cpp") {
            self.ir.forward_decls.push_str("#ifdef __cplusplus\n}\n#endif\n");
        }
        
        self.extern_functions.insert(
            func.name.clone(),
            ExternFunctionMap {
                params: func.params.clone(),
                return_type: func.return_type.clone(),
                abi: abi.to_string(),
                library: library.map(|s| s.to_string()),
            }
        );
        
        Ok(())
    }

    pub fn codegen_externs(&mut self, externs: &[ExternDecl]) -> Result<(), ()> {
        for ext in externs {
            match ext {
                ExternDecl::Single { abi, func } => {
                    self.codegen_extern_function(abi, func, None)?;
                }
                ExternDecl::Block { abi, library, functions } => {
                    self.track_library(library);
                    for func in functions {
                        self.codegen_extern_function(abi, func, Some(library))?;
                    }
                }
                ExternDecl::SingleWithBody { .. } => {
                    
                }
            }
        }
        Ok(())
    }
    
    fn track_library(&mut self, library: &str) {
        let lib_name = self.extract_library_name(library);
        
        if !self.linked_libraries.contains(&lib_name) {
            self.linked_libraries.push(lib_name);
        }
    }
    
    fn extract_library_name(&self, library: &str) -> String {
        
        let library = library.trim();
        let filename = if library.contains('/') || library.contains('\\') {
            library.split(&['/', '\\'][..]).next_back().unwrap_or(library)
                } else {
            library
        };
        
        filename
            .trim_end_matches(".dll")
            .trim_end_matches(".lib")
            .trim_end_matches(".so")
            .trim_end_matches(".dylib")
            .trim_end_matches(".a")
            .to_string()
    }
    
    pub fn get_linked_libraries(&self) -> &[String] {
        &self.linked_libraries
    }
}