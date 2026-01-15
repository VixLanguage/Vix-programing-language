use crate::import::*;

impl Codegen {
        pub fn codegen_struct_definition(&mut self, struct_def: &StructDef) -> Result<(), ()> {
        self.ensure_string_typedef();
        
        let loc = self.default_location();

        if self.structs.contains_key(&struct_def.name) {
            self.diagnostics.error(
                "DuplicateStruct",
                &format!("Struct '{}' is already defined", struct_def.name),
                ErrorContext {
                    primary_location: loc.clone(),
                    secondary_locations: vec![],
                    help_message: Some("Each struct must have a unique name.".to_string()),
                    suggestions: vec![
                        format!("Rename this struct to something else"),
                        "Check for duplicate struct definitions".to_string(),
                    ],
                }
            );
        }
        
        let mut struct_code = format!("typedef struct {} {{\n", struct_def.name);
        let mut fields_info = Vec::new();
        
        for field in &struct_def.fields {
            if matches!(field.ty, Type::Void) {
                self.diagnostics.error(
                    "VoidField",
                    &format!("Field '{}' in struct '{}' cannot be void", field.name, struct_def.name),
                    ErrorContext {
                        primary_location: loc.clone(),
                        secondary_locations: vec![],
                        help_message: Some("Struct fields must have concrete types.".to_string()),
                        suggestions: vec![
                            format!("Change field '{}' to a concrete type", field.name),
                        ],
                    }
                );
            }
            
             
            let c_type = match &field.ty {
                Type::Str { .. } => "String".to_string(),
                _ => field.ty.to_c_type(&self.arch)
            };
            
            struct_code.push_str(&format!("    {} {};\n", c_type, field.name));
            fields_info.push((field.name.clone(), field.ty.clone(), field.is_public));
        }
        
        struct_code.push_str(&format!("}} {};\n\n", struct_def.name));
        
        self.structs.insert(
            struct_def.name.clone(),
            StructInfo {
                fields: fields_info,
                llvm_type: struct_def.name.clone(),
            }
        );
        
        self.ir.forward_decls.push_str(&struct_code);
        
        // Auto-generate default constructor
        let constructor_name = format!("{}_new", struct_def.name);
        
        // Generate signature
        let mut param_strs = Vec::new();
        let mut params_c = Vec::new();
        for field in &struct_def.fields {
             let c_type = match &field.ty {
                Type::Str { .. } => "String".to_string(),
                _ => field.ty.to_c_type(&self.arch)
            };
            params_c.push(format!("{} {}", c_type, field.name));
            param_strs.push(field.name.clone());
        }
        
        let signature = format!("{} {}({})", struct_def.name, constructor_name, params_c.join(", "));
        self.ir.forward_decls.push_str(&format!("{};\n", signature));
        
        // Generate body
        let mut func_code = String::new();
        func_code.push_str(&format!("{} {{\n", signature));
        func_code.push_str(&format!("    {} instance;\n", struct_def.name));
        
        for field in &struct_def.fields {
            func_code.push_str(&format!("    instance.{} = {};\n", field.name, field.name));
        }
        
        func_code.push_str("    return instance;\n");
        func_code.push_str("}\n\n");
        
        self.ir.functions.push_str(&func_code);

        Ok(())
    }

    pub fn codegen_impl_block(&mut self, impl_block: &ImplBlock, only_signatures: bool) -> Result<()> {
        let loc = self.default_location();
        
        if !self.structs.contains_key(&impl_block.struct_name) {
            self.diagnostics.error(
                "UndefinedStruct",
                &format!("Struct '{}' is not defined", impl_block.struct_name),
                ErrorContext {
                    primary_location: loc.clone(),
                    secondary_locations: vec![],
                    help_message: Some(format!("Cannot implement methods for undefined struct '{}'.", impl_block.struct_name)),
                    suggestions: vec![
                        format!("Define struct '{}' before this impl block", impl_block.struct_name),
                        "Check for typos in the struct name".to_string(),
                    ],
                }
            );
        }
        
        if let Some(constructor_body) = &impl_block.constructor_body {
            self.codegen_constructor(&impl_block.struct_name, &impl_block.constructor_params, constructor_body, only_signatures);
        }
        
        for method in &impl_block.methods {
            self.codegen_impl_method(method, &impl_block.struct_name, only_signatures);
        }
        Ok(())
    }
    
    pub fn codegen_constructor(
        &mut self, 
        struct_name: &str, 
        params: &[(String, Type)], 
        body: &[(String, Expr)],
        only_signatures: bool
    ) -> Result<(), ()> {
        self.ensure_string_typedef();
        
        let mut func_code = String::new();
        let func_name = format!("{}_new", struct_name);
        
         
        func_code.push_str(&format!("{} {}(", struct_name, func_name));
        
         
        let param_strs: Vec<String> = params.iter().map(|(name, ty)| {
            let c_name = format!("param_{}", name);
            self.vars.insert(name.clone(), (c_name.clone(), ty.clone()));
            
            let c_type = match ty {
                Type::Str { .. } => "String".to_string(),
                _ => ty.to_c_type(&self.arch)
            };
            
            format!("{} {}", c_type, c_name)
        }).collect();
        
        func_code.push_str(&param_strs.join(", "));
        func_code.push_str(") {\n");
        func_code.push_str(&format!("    {} instance;\n", struct_name));
        
         
        for (field_name, field_expr) in body {
            let mut temp_body = String::new();
            let (val_var, _) = self.codegen_expr(field_expr, &mut temp_body)
                .map_err(|_| ())?;
            
            func_code.push_str(&temp_body);
            func_code.push_str(&format!("    instance.{} = {};\n", field_name, val_var));
        }
        
         
        for (name, _) in params {
            self.vars.remove(name);
        }
        
        func_code.push_str("    return instance;\n");
        func_code.push_str("}\n\n");
        
         
        let param_types: Vec<(String, Type)> = params.iter()
            .map(|(n, t)| (n.clone(), t.clone()))
            .collect();
        
        self.user_functions.insert(
            func_name.clone(),
            (param_types, Type::Struct { name: struct_name.to_string() })
        );
        
        if only_signatures {
            let func_name = format!("{}_new", struct_name);
             
            for (_, p_ty) in params {
                self.ensure_type_defined(p_ty);
            }
            
            let mut params_str = Vec::new();
            for (p_name, p_ty) in params {
                params_str.push(format!("{} param_{}", p_ty.to_c_type(&self.arch), p_name));
            }
            let sig = format!("{} {}({});\n", struct_name, func_name, params_str.join(", "));
            self.ir.forward_decls.push_str(&sig);
            return Ok(());
        }

        self.ir.functions.push_str(&func_code);
        Ok(())
    }


    
    pub fn codegen_impl_method(&mut self, method: &ImplMethod, struct_name: &str, only_signatures: bool) {
    self.ensure_string_typedef();
    
     
    for (param_name, param_type, _) in &method.params {
        if matches!(param_type, Type::Void) {
            self.diagnostics.error(
                "VoidParameter",
                &format!("Parameter '{}' in method '{}' cannot be void", param_name, method.name),
                ErrorContext {
                    primary_location: self.default_location(),
                    secondary_locations: vec![],
                    help_message: Some("Method parameters must have concrete types.".to_string()),
                    suggestions: vec![format!("Change parameter '{}' to a concrete type", param_name)],
                }
            );
        }
    }
    
    let mut func_code = String::new();
    let method_name = format!("{}_{}", struct_name, method.name);

     
    let params_for_registry: Vec<(String, Type)> = method.params.iter()
        .map(|(n, t, _)| (n.clone(), t.clone()))
        .collect();

    let is_instance = method.self_modifier.is_some();
    self.impl_methods.insert(
        (struct_name.to_string(), method.name.clone()),
        (params_for_registry, method.return_type.clone(), is_instance)
    );

    if only_signatures {
         
        self.ensure_type_defined(&method.return_type);
        for (_, ty, _) in &method.params {
            self.ensure_type_defined(ty);
        }

        let mut params_str = Vec::new();
        if is_instance {
            params_str.push(format!("{}* self", struct_name));
        }
        for (p_name, p_ty, _) in &method.params {
            params_str.push(format!("{} {}", p_ty.to_c_type(&self.arch), p_name));
        }
        let c_return_type = method.return_type.to_c_type(&self.arch);
        let sig = format!("{} {}({});\n", c_return_type, method_name, params_str.join(", "));
        self.ir.forward_decls.push_str(&sig);
        return;
    }

     
    self.current_return_type = Some(method.return_type.clone());

     
    match &method.return_type {
        Type::Result { ok, err } => {
             
            if let Type::Tuple { fields } = ok.as_ref() {
                if let Some(tuple_def) = self.type_registry.generate_tuple_definition(fields, &self.config.arch) {
                    if !self.ir.forward_decls.contains(&tuple_def) {
                        self.ir.forward_decls.push_str(&tuple_def);
                        self.ir.forward_decls.push_str("\n");
                    }
                }
            }
            
             
            if let Type::Tuple { fields } = err.as_ref() {
                if let Some(tuple_def) = self.type_registry.generate_tuple_definition(fields, &self.config.arch) {
                    if !self.ir.forward_decls.contains(&tuple_def) {
                        self.ir.forward_decls.push_str(&tuple_def);
                        self.ir.forward_decls.push_str("\n");
                    }
                }
            }
            
             
            if let Some(result_def) = self.type_registry.generate_result_definition(ok, err, &self.config.arch) {
                if !self.ir.forward_decls.contains(&result_def) {
                    self.ir.forward_decls.push_str(&result_def);
                    self.ir.forward_decls.push_str("\n");
                }
            }
        }
        Type::Option { inner } => {
             
            if let Type::Tuple { fields } = inner.as_ref() {
                if let Some(tuple_def) = self.type_registry.generate_tuple_definition(fields, &self.config.arch) {
                    if !self.ir.forward_decls.contains(&tuple_def) {
                        self.ir.forward_decls.push_str(&tuple_def);
                        self.ir.forward_decls.push_str("\n");
                    }
                }
            }
            
             
            if let Some(option_def) = self.type_registry.generate_option_definition(inner, &self.config.arch) {
                if !self.ir.forward_decls.contains(&option_def) {
                    self.ir.forward_decls.push_str(&option_def);
                    self.ir.forward_decls.push_str("\n");
                }
            }
        }
        Type::Tuple { fields } => {
             
            if let Some(tuple_def) = self.type_registry.generate_tuple_definition(fields, &self.config.arch) {
                if !self.ir.forward_decls.contains(&tuple_def) {
                    self.ir.forward_decls.push_str(&tuple_def);
                    self.ir.forward_decls.push_str("\n");
                }
            }
        }
        _ => {}
    }

     
    let return_c_type = method.return_type.to_c_type(&self.arch);

    func_code.push_str(&format!("{} {}(", return_c_type, method_name));
    
     
    if let Some(self_mod) = &method.self_modifier {
        let self_param = match self_mod {
            SelfModifier::Immutable => format!("{}* self", struct_name),
            SelfModifier::Mutable => format!("{}* self", struct_name),
            SelfModifier::Reference => format!("{}* self", struct_name),
            SelfModifier::Borrow => format!("{}* self", struct_name),
        };
        
        func_code.push_str(&self_param);
        
         
        let self_type = match self_mod {
            SelfModifier::Mutable | SelfModifier::Borrow => {
                Type::MutRef(Box::new(Type::Struct { name: struct_name.to_string() }))
            }
            _ => {
                Type::Ref(Box::new(Type::Struct { name: struct_name.to_string() }))
            }
        };
        
        self.vars.insert("self".to_string(), ("self".to_string(), self_type));
        
        if !method.params.is_empty() {
            func_code.push_str(", ");
        }
    }
    
     
    let param_strs: Vec<String> = method.params.iter().map(|(name, ty, _)| {
        let c_type = match ty {
            Type::Str { .. } => "String".to_string(),
            _ => ty.to_c_type(&self.arch)
        };
        let c_name = name.clone();
        
         
        self.vars.insert(name.clone(), (c_name.clone(), ty.clone()));
        
        format!("{} {}", c_type, c_name)
    }).collect();
    
    func_code.push_str(&param_strs.join(", "));
    func_code.push_str(") {\n");
    
     
    let mut body_code = String::new();
    for stmt in &method.body {
        let _ = self.codegen_stmt(stmt, &mut body_code);
    }
    func_code.push_str(&body_code);
    
     
    if matches!(method.return_type, Type::Void) {
        if !body_code.contains("return") {
            func_code.push_str("    return;\n");
        }
    }
    
    func_code.push_str("}\n\n");
    
     
    let params_for_registry: Vec<(String, Type)> = method.params.iter()
        .map(|(n, t, _)| (n.clone(), t.clone()))
        .collect();

    self.impl_methods.insert(
        (struct_name.to_string(), method.name.clone()),
        (params_for_registry.clone(), method.return_type.clone(), is_instance)
    );
    
     
    self.vars.remove("self");
    for (param_name, _, _) in &method.params {
        self.vars.remove(param_name);
    }
    
    self.current_return_type = None;  
    
    self.ir.functions.push_str(&func_code);
}

    pub fn codegen_function(&mut self, func: &Function, only_signatures: bool) {
        self.vars.clear();
        self.var_count = 0;
        let loc = self.default_location();

         
        let param_types: Vec<(String, Type)> = func.params.iter()
            .map(|(n, t, _)| (n.clone(), t.clone()))
            .collect();
        self.user_functions.insert(func.name.clone(), (param_types, func.return_type.clone()));

        let c_return_type = func.return_type.to_c_type(&self.arch);
        let c_func_name = if func.name == "main" { "vix_main".to_string() } else { func.name.clone() };
        let mut params_str = Vec::new();
        for (p_name, p_ty, _) in &func.params {
            let c_p_type = p_ty.to_c_type(&self.arch);
            params_str.push(format!("{} var_{}", c_p_type, p_name));
             
            self.vars.insert(p_name.clone(), (format!("var_{}", p_name), p_ty.clone()));
        }

        if only_signatures {
            let sig = format!("{} {}({});\n", c_return_type, c_func_name, params_str.join(", "));
            if !self.ir.forward_decls.contains(&sig) {
                self.ir.forward_decls.push_str(&sig);
            }
            return;
        }

         
        let mut func_code = String::new();
        self.current_return_type = Some(func.return_type.clone());
        func_code.push_str(&format!("{} {}(", c_return_type, c_func_name));
        func_code.push_str(&params_str.join(", "));
        func_code.push_str(") {\n");

        let mut body_code = String::new();
        for stmt in &func.body {
            self.codegen_stmt(stmt, &mut body_code);
        }

        self.current_return_type = None;
        func_code.push_str(&body_code);

        if matches!(func.return_type, Type::Void) {
            if !body_code.contains("return") {
                func_code.push_str("    return;\n");
            }
        }

        func_code.push_str("}\n\n");
        self.ir.functions.push_str(&func_code);
    }


    pub fn codegen_if(
        &mut self,
        cond: &Expr,
        then_body: &[Stmt],
        else_body: &Option<Vec<Stmt>>,
        body: &mut String,
    ) -> Result<(), ()> {
        let (cond_var, _cond_ty) = self.codegen_expr(cond, body) .check_error();

        body.push_str(&format!("if ({}) {{\n", cond_var));

        for stmt in then_body {
            self.codegen_stmt(stmt, body);
        }

        body.push_str("}\n");

        if let Some(else_stmts) = else_body {
            body.push_str("else {\n");
            for stmt in else_stmts {
                self.codegen_stmt(stmt, body);
            }
            body.push_str("}\n");
        }

        Ok(())
    }
    pub fn codegen_enum_definition(&mut self, enum_def: &EnumDef) -> Result<(), ()> {
        let mut variants = Vec::new();
        for variant in &enum_def.variants {
            match variant {
                EnumVariant::Simple(name) => variants.push((name.clone(), None)),
                EnumVariant::Tuple(name, types) => {
                    variants.push((name.clone(), Some(Type::Tuple { fields: types.clone() })));
                }
                EnumVariant::Struct(name, fields) => {
                     let payload_name = format!("{}_{}_Payload", enum_def.name, name);
                     let struct_def = StructDef {
                        name: payload_name.clone(),
                        fields: fields.clone(),
                        is_public: enum_def.is_public,
                     };
                     
                     
                    let reg_fields = fields.iter().map(|f| (f.name.clone(), f.ty.clone())).collect();
                    self.type_registry.register_struct(payload_name.clone(), reg_fields);
                    self.codegen_struct_definition(&struct_def);

                     variants.push((name.clone(), Some(Type::Struct { name: payload_name })));
                }
            }
        }
        self.type_registry.register_enum(enum_def.name.clone(), variants, enum_def.is_public);
        
        if let Some(def_code) = self.type_registry.generate_enum_definition(&enum_def.name, &self.config.arch) {
            self.ir.forward_decls.push_str(&def_code);
        }

        Ok(())
    }

    
    pub fn codegen_scope(&mut self, stmts: &[Stmt], body: &mut String) -> Result<(), ()>{
        self.scope_depth += 1;
        let prev_owned_vars = self.owned_vars.clone();
        
        body.push_str("{\n");
        for stmt in stmts {
            self.codegen_stmt(stmt, body);
        }
        
        let current_owned = self.owned_vars.clone();
        for var_name in current_owned {
            if !prev_owned_vars.contains(&var_name) {
                if let Some((c_name, _)) = self.vars.get(&var_name) {
                    body.push_str(&format!("free({});\n", c_name));
                }
                self.owned_vars.remove(&var_name);
            }
        }
        
        body.push_str("}\n");
        self.scope_depth -= 1;

        Ok(())
    }

    pub fn codegen_while(&mut self, cond: &Expr, loop_body: &[Stmt], body: &mut String, _loc: SourceLocation)  -> Result<(), ()> {
        let loop_label = self.fresh_label();
        let end_label = self.fresh_label();
        
        body.push_str(&format!("{}:\n", loop_label));
        
        let (cond_var, _) = self.codegen_expr(cond, body).check_error();
        body.push_str(&format!("if (!{}) goto {};\n", cond_var, end_label));
        
        for stmt in loop_body {
            self.codegen_stmt(stmt, body);
        }
        
        body.push_str(&format!("goto {};\n", loop_label));
        body.push_str(&format!("{}:\n", end_label));

        Ok(()) 
    }

    pub fn codegen_for(&mut self, var: &str, iter: &Expr, loop_body: &[Stmt], body: &mut String, _loc: SourceLocation) -> Result<(), ()> {
        let (iter_var, iter_ty) = self.codegen_expr(iter, body) .check_error();
        println!("[DEBUG] codegen_for: var={}, iter_ty={:?}", var, iter_ty);
        println!("[DEBUG] loop_body length: {}", loop_body.len());
        for (i, stmt) in loop_body.iter().enumerate() {
            println!("[DEBUG] stmt[{}]: {:?}", i, stmt);
        }
        let loop_label = self.fresh_label();
        let end_label = self.fresh_label();
        let idx_var = self.fresh_var();
        let (size_expr, elem_access, elem_type) = match &iter_ty {
            Type::Array { element, size: Some(size) } => {
                (format!("{}", size), format!("{}[{}]", iter_var, idx_var), *element.clone())
            }
            Type::Array { element, size: None } => {
                (format!("{}.len", iter_var), format!("{}.ptr[{}]", iter_var, idx_var), *element.clone())
            }
        
            Type::MultiArray { element, dimensions } => {
                if dimensions.is_empty() {
                    ("0".to_string(), "NULL".to_string(), *element.clone())
                } else {
                    (format!("{}", dimensions[0]), format!("{}[{}]", iter_var, idx_var), *element.clone())
                }
            }
            Type::Result { ok, .. } => {
                 
                (format!("({}.tag == 0 ? 1 : 0)", iter_var), format!("{}.data.ok", iter_var), *ok.clone())
            }
            Type::Option { inner } => {
                 
                 
                (format!("({}.tag == 1 ? 1 : 0)", iter_var), format!("{}.value", iter_var), *inner.clone())
            }
            _ => {
                ("1".to_string(), format!("{}[{}]", iter_var, idx_var), Type::i32())
            }
        };
        
        let c_name = format!("var_{}", var);
        let c_type = elem_type.to_c_type(&self.arch);
        
        body.push_str(&format!("size_t {} = 0;\n", idx_var));
        body.push_str(&format!("{}:\n", loop_label));
        body.push_str(&format!("if ({} >= {}) goto {};\n", idx_var, size_expr, end_label));
        body.push_str(&format!("{} {} = {};\n", c_type, c_name, elem_access));
        
        self.vars.insert(var.to_string(), (c_name, elem_type));
        
        for stmt in loop_body {
            self.codegen_stmt(stmt, body);
        }
        
        body.push_str(&format!("{}++;\n", idx_var));
        body.push_str(&format!("goto {};\n", loop_label));
        body.push_str(&format!("{}:\n", end_label));
        
        Ok(())
    }

}