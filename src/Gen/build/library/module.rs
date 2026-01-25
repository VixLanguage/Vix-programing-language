use crate::import::*;

impl Codegen {
    pub fn codegen_module_call(
        &mut self, 
        module: &str, 
        func: &str, 
        args: &[Expr], 
        body: &mut String, 
        loc: SourceLocation
    ) -> Result<(String, Type), ()> {
         
        let full_func_name = format!("{}_{}", module, func);
        
        println!("[DEBUG] codegen_module_call: module={}, func={}, full_name={}", 
                 module, func, full_func_name);
        
         
        let return_type = if let Some((_params, ret_ty)) = self.user_functions.get(&full_func_name) {
            println!("[DEBUG] Found user function: {} -> {:?}", full_func_name, ret_ty);
            ret_ty.clone()
        } else if let Some(ext_info) = self.extern_functions.get(&full_func_name) {
            println!("[DEBUG] Found extern function: {} -> {:?}", full_func_name, ext_info.return_type);
            ext_info.return_type.clone()
        } else if let Some((_params, ret_ty, _)) = self.module_functions.get(&(module.to_string(), func.to_string())) {
            println!("[DEBUG] Found module function: {} -> {:?}", full_func_name, ret_ty);
            ret_ty.clone()
        } else {
            println!("[DEBUG] Function '{}' not found in module '{}'", func, module);
            self.diagnostics.warning(
                "UndefinedModuleFunction",
                &format!("Function '{}' not found in module '{}'", func, module),
                ErrorContext {
                    primary_location: loc.clone(),
                    secondary_locations: vec![],
                    help_message: Some(format!(
                        "Make sure the function '{}' is defined and exported in module '{}'",
                        func, module
                    )),
                    suggestions: vec![
                        format!("Check if '{}' is public in the module", func),
                        "Verify the module was imported correctly".to_string(),
                    ],
                }
            );
            Type::Void
        };
        
         
        let mut arg_vars = Vec::new();
        for arg in args {
            let (var, _ty) = self.codegen_expr(arg, body)?;
            arg_vars.push(var);
        }
        
        let tmp = self.fresh_var();
        let c_type = return_type.to_c_type(&self.arch, &mut self.type_registry);
        let args_str = arg_vars.join(", ");
        
         
        if matches!(return_type, Type::Void) {
            body.push_str(&format!("{}({});\n", full_func_name, args_str));
             
            body.push_str(&format!("int32_t {} = 0;\n", tmp));
            Ok((tmp, Type::Void))
        } else {
            body.push_str(&format!("{} {} = {}({});\n", c_type, tmp, full_func_name, args_str));
            Ok((tmp, return_type))
        }
    }
    
    pub fn codegen_module(&mut self, module: &Stmt) {
        if let Stmt::ModuleDef { name, body, is_public: _ } = module {
            let init_func_name = format!("{}_init", name);
            self.module_init_functions.push(init_func_name.clone());
            
            let mut init_body = String::new();
            self.ir.forward_decls.push_str(&format!("void {}();\n", init_func_name));
            
            for stmt in body {
                match stmt {
                    Stmt::Function(f) => {
                         
                        let full_func_name = format!("{}_{}", name, f.name);
                        
                         
                        self.user_functions.insert(
                            full_func_name.clone(),
                            (
                                f.params.iter()
                                    .map(|(pname, pty, _)| (pname.clone(), pty.clone()))
                                    .collect(),
                                f.return_type.clone()
                            )
                        );
                        
                         
                        self.module_functions.insert(
                            (name.clone(), f.name.clone()),
                            (
                                f.params.iter()
                                    .map(|(pname, pty, _)| (pname.clone(), pty.clone()))
                                    .collect(),
                                f.return_type.clone(),
                                f.is_public
                            )
                        );
                        
                         
                        self.codegen_function(f, true);
                         
                        self.codegen_function(f, false);
                    }
                    Stmt::StructDef(s) => { 
                        self.codegen_struct_definition(s).map_err(|_| ()).ok(); 
                    }
                    Stmt::EnumDef(e) => { 
                        self.codegen_enum_definition(e).map_err(|_| ()).ok(); 
                    }
                    Stmt::ModuleDef { .. } => { 
                        self.codegen_module(stmt); 
                    }
                    _ => {
                        self.codegen_stmt(stmt, &mut init_body).ok();
                    }
                }
            }
            
            let code = format!("void {}() {{\n{}\n}}\n", init_func_name, init_body);
            self.ir.functions.push_str(&code);
        }
    }
}