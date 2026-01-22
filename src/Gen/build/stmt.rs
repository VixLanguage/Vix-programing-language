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

            let c_type = field.ty.to_c_type(&self.arch, &mut self.type_registry);

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

        Ok(())
    }


    pub fn codegen_typed_declaration_impl(&mut self, name: &str, ty: &Type, value: &Expr, body: &mut String, loc: SourceLocation, is_mutable: bool) -> Result<(), ()> {
            if matches!(ty, Type::StdStr)
                && let Expr::String(s) = value {
                    let c_name = format!("var_{}", name);
                    
                    if s.is_empty() {
                         
                        self.ensure_malloc_helpers();
                        self.ensure_std_str();
                        body.push_str(&format!("String {} = String_new();\n", c_name));
                        self.vars.insert(name.to_string(), (c_name, Type::StdStr));
                        return Ok(());
                    } else {
                         
                        self.ensure_malloc_helpers();
                        self.ensure_std_str();
                        let (val_var, _) = self.codegen_expr(value, body)?;
                        body.push_str(&format!("String {} = String_from_str({});\n", c_name, val_var));
                        self.vars.insert(name.to_string(), (c_name, Type::StdStr));
                        return Ok(());
                    }
                }

        
        if let Type::Union { variants } = ty {
            self.type_registry.ensure_union_defined(variants, &self.arch);
            let (val_var, val_ty) = self.codegen_expr(value, body)?;
            let mut matched_idx = None;

            for (idx, variant) in variants.iter().enumerate() {
                if self.types_compatible(variant, &val_ty) {
                    matched_idx = Some(idx);
                    break;
                }
            }
            
            if let Some(idx) = matched_idx {
                let union_type_name = self.type_registry.get_union_type_name(variants, &self.arch);
                let c_name = format!("var_{}", name);
                
                let decl_prefix = if is_mutable { "" } else { "const " };
                body.push_str(&format!("{}{} {};\n", decl_prefix, union_type_name, c_name));
                body.push_str(&format!("{}.tag = {};\n", c_name, idx));
                body.push_str(&format!("{}.data.variant_{} = {};\n", c_name, idx, val_var));
                
                self.vars.insert(name.to_string(), (c_name, ty.clone()));
                return Ok(());
            } else {
                self.diagnostics.error(
                    "UnionTypeMismatch",
                    &format!("Value of type '{}' does not match any variant in the union type", val_ty.name()),
                    ErrorContext {
                        primary_location: loc,
                        secondary_locations: vec![],
                        help_message: Some(format!(
                            "Union type expects one of: {}",
                            variants.iter().map(|v| v.name()).collect::<Vec<_>>().join(", ")
                        )),
                        suggestions: vec![
                            "Change the value to match one of the union variants".to_string(),
                            "Cast the value to the correct type".to_string(),
                        ],
                    }
                );
                return Err(());
            }
        }
        
        let (val_var, val_ty) = self.codegen_expr(value, body).check_error();
        
        let effective_ty = if matches!(ty, Type::Auto) {
            val_ty.clone()
        } else {
            ty.clone()
        };
        let ty = &effective_ty;

        let c_name = format!("var_{}", name);
        let base_c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
        
        let decl_type = if !is_mutable {
            if base_c_type.starts_with("const ") {
                base_c_type.clone()
            } else {
                match ty {
                    Type::Array { size: Some(_), .. } | Type::MultiArray { .. } => base_c_type.clone(),
                    Type::Str { .. } => base_c_type.clone(), 
                    Type::Struct { .. } => base_c_type.clone(),
                    _ => format!("const {}", base_c_type)
                }
            }
        } else {
            base_c_type.clone()
        };


        match ty {
            Type::Void => {
                self.diagnostics.error(
                    "VoidVariable",
                    &format!("Variable '{}' cannot have void type", name),
                    void_variable_error(name, loc)
                );
                return Err(());
            }
            
            Type::Array { element, size: Some(size) } => {
                if matches!(**element, Type::Void) {
                    self.diagnostics.error(
                        "VoidArrayElement",
                        &format!("Array '{}' cannot have void elements", name),
                        void_array_error(loc)
                    );
                    return Err(());
                }
                
                let elem_c_type = element.to_c_type(&self.arch, &mut self.type_registry);
                
                body.push_str(&format!("{} {}[{}];\n", elem_c_type, c_name, size));
                body.push_str(&format!("memcpy({}, {}, sizeof({}));\n", c_name, val_var, c_name));
            }
            
            Type::Array { element, size: None } => {
                if matches!(**element, Type::Void) {
                    self.diagnostics.error(
                        "VoidArrayElement",
                        &format!("Array '{}' cannot have void elements", name),
                        void_array_error(loc)
                    );
                    return Err(());
                }
                
                
                let elem_c_type = element.to_c_type(&self.arch, &mut self.type_registry);
                let slice_type = self.type_registry.generate_slices(element, &self.arch);
                
                
                if !self.types_compatible(&val_ty, ty) {
                    
                    match (&val_ty, element.as_ref()) {
                        (Type::Array { element: val_elem, .. }, Type::Str { .. }) 
                            if matches!(**val_elem, Type::StdStr) => {
                            
                            self.diagnostics.error(
                                "TypeMismatch",
                                "Cannot assign String[] to str[]. Elements must be converted explicitly.",
                                ErrorContext {
                                    primary_location: loc.clone(),
                                    secondary_locations: vec![],
                                    help_message: Some("Use .as_str() to convert String to str:\n    arr += [x.as_str()]".to_string()),
                                    suggestions: vec![
                                        "Convert String elements to str using .as_str()".to_string(),
                                    ],
                                }
                            );
                            return Err(());
                        }
                        
                        
                        (Type::StdStr, Type::Str { .. }) => {
                            self.diagnostics.error(
                                "TypeMismatch",
                                "Cannot assign String to str[]. Expected array.",
                                ErrorContext {
                                    primary_location: loc.clone(),
                                    secondary_locations: vec![],
                                    help_message: Some(format!(
                                        "Did you mean:\n    mut {}: String = ...\n    or\n    mut {}: str[] = [...]",
                                        name, name
                                    )),
                                    suggestions: vec![
                                        format!("Change type to 'String' instead of 'str[]'"),
                                        format!("Wrap value in array: [{}]", name),
                                    ],
                                }
                            );
                            return Err(());
                        }
                        
                        _ => {
                            self.diagnostics.error(
                                "TypeMismatch",
                                &format!("Cannot initialize {} with {}", ty.name(), val_ty.name()),
                                type_mismatch_error(&ty.name(), &val_ty.name(), loc.clone(), value.location())
                            );
                            return Err(());
                        }
                    }
                }
                
                body.push_str(&format!("{} {} = {};\n", slice_type, c_name, val_var));
            }
            
            Type::MultiArray { element, dimensions } => {
                if matches!(**element, Type::Void) {
                    self.diagnostics.error(
                        "VoidArrayElement",
                        &format!("Multi-array '{}' cannot have void elements", name),
                        void_array_error(loc)
                    );
                    return Err(());
                }
                
                let elem_c_type = element.to_c_type(&self.arch, &mut self.type_registry);
                let dims_str = dimensions.iter().map(|d| format!("[{}]", d)).collect::<String>();

                body.push_str(&format!("{} {}{};\n", elem_c_type, c_name, dims_str));
                body.push_str(&format!("memcpy({}, {}, sizeof({}));\n", c_name, val_var, c_name));
            }
            
            Type::Tuple { fields } => {
                for (i, field) in fields.iter().enumerate() {
                    if matches!(field, Type::Void) {
                        self.diagnostics.error(
                            "VoidTupleField",
                            &format!("Tuple field {} in variable '{}' cannot be void", i, name),
                            ErrorContext {
                                primary_location: loc.clone(),
                                secondary_locations: vec![],
                                help_message: Some(format!("Tuple fields must have concrete types. Field {} is void.", i)),
                                suggestions: vec![
                                    format!("Change field {} to a concrete type", i),
                                    "Use Option<T> for optional fields".to_string(),
                                ],
                            }
                        );
                        return Err(());
                    }
                }
                
                body.push_str(&format!("{} {} = {};\n", decl_type, c_name, val_var));
            }
            
            Type::Union { .. } => {
                 
                unreachable!("Union types should be handled at the start of the function");
            }
            
            Type::Ptr(inner) if matches!(**inner, Type::Void) => {
                body.push_str(&format!("{} {} = {};\n", decl_type, c_name, val_var));
            }
            Type::StdStr => {
                let (val_var, val_ty) = self.codegen_expr(value, body)?;
                
                self.ensure_malloc_helpers();
                self.ensure_std_str();
                
                let c_name = format!("var_{}", name);
                
                match val_ty {
                    Type::Str { .. } | Type::ConstStr => {
                        body.push_str(&format!("String {} = String_from_str({});\n", c_name, val_var));
                    }

                    Type::StdStr => {
                        body.push_str(&format!("String {} = {};\n", c_name, val_var));
                    }

                    _ => {
                        self.diagnostics.error(
                            "TypeMismatch",
                            &format!("Cannot initialize String with type {}", val_ty.name()),
                            ErrorContext {
                                primary_location: loc.clone(),
                                secondary_locations: vec![],
                                help_message: Some("String can only be initialized from str or String".to_string()),
                                suggestions: vec![],
                            }
                        );
                        return Err(());
                    }
                }
                
                self.vars.insert(name.to_string(), (c_name, Type::StdStr));
            }
            _ => {
                if !self.types_compatible(ty, &val_ty) {
                    let value_loc = value.location();
                    
                    self.diagnostics.error(
                        "TypeMismatch",
                        &format!("Cannot initialize variable '{}' of type {} with value of type {}", 
                                name, ty.name(), val_ty.name()),
                        type_mismatch_error(
                            &ty.name(),
                            &val_ty.name(),
                            loc.clone(),
                            value_loc,
                        )
                    );
                    return Err(());
                }
                
                match (ty, &val_ty) {
                    (Type::ConstStr, Type::Str { .. }) => {
                        body.push_str(&format!("{} {} = {}.ptr;\n", decl_type, c_name, val_var));
                    }
                    _ => {
                        body.push_str(&format!("{} {} = {};\n", decl_type, c_name, val_var));
                    }
                }
            }
        }
        
        if matches!(ty, Type::Owned(_)) {
            self.owned_vars.insert(name.to_string());
        }
        self.vars.insert(name.to_string(), (c_name, ty.clone()));
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
            self.codegen_constructor(
                &impl_block.struct_name, 
                &impl_block.constructor_params, 
                constructor_body, 
                only_signatures
            ).ok();
        } else if !impl_block.constructor_params.is_empty() {
             
            let field_inits: Vec<(String, Expr)> = impl_block.constructor_params
                .iter()
                .map(|(name, _)| (name.clone(), Expr::Var(name.clone())))
                .collect();
            
            self.codegen_constructor(
                &impl_block.struct_name,
                &impl_block.constructor_params,
                &field_inits,
                only_signatures
            ).ok();
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
            let c_type =  ty.to_c_type(&self.arch, &mut self.type_registry);
            self.vars.insert(name.clone(), (c_name.clone(), ty.clone()));

            format!("{} {}", c_type, c_name)
        }).collect();
        
        func_code.push_str(&param_strs.join(", "));
        func_code.push_str(") {\n");
        func_code.push_str(&format!("    {} instance;\n", struct_name));
        
         
        for (field_name, field_expr) in body {
            let mut temp_body = String::new();
            let (val_var, _) = self.codegen_expr(field_expr, &mut temp_body).map_err(|_| ())?;
            
            func_code.push_str(&temp_body);
            func_code.push_str(&format!("    instance.{} = {};\n", field_name, val_var));
        }
        
         
        for (name, _) in params {
            self.vars.remove(name);
        }
        
        func_code.push_str("    return instance;\n");
        func_code.push_str("}\n\n");
        
         
        let param_types: Vec<(String, Type)> = params.iter().map(|(n, t)| (n.clone(), t.clone())).collect();
        
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
                params_str.push(format!("{} param_{}", p_ty.to_c_type(&self.arch, &mut self.type_registry), p_name));
            }
            let sig = format!("{} {}({});\n", struct_name, func_name, params_str.join(", "));
            self.ir.forward_decls.push_str(&sig);
            return Ok(());
        }

        self.ir.functions.push_str(&func_code);
        Ok(())
    }

    
    pub fn codegen_impl_method(&mut self, method: &ImplMethod, struct_name: &str, only_signatures: bool) {
        self.vars.clear();
        self.var_count = 0;
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
        let params_for_registry: Vec<(String, Type)> = method.params.iter().map(|(n, t, _)| (n.clone(), t.clone())).collect();
        let is_instance = method.self_modifier.is_some();

        self.impl_methods.insert(
            (struct_name.to_string(), method.name.clone()),
            (params_for_registry, method.return_type.clone(), is_instance)
        );

        if only_signatures {
            let mut params_str = Vec::new();
            
            if is_instance {
                params_str.push(format!("{}* self", struct_name));
            }
            for (p_name, p_ty, _) in &method.params {
                params_str.push(format!("{} {}", p_ty.to_c_type(&self.arch, &mut self.type_registry), p_name));
            }
            let c_return_type = method.return_type.to_c_type(&self.arch, &mut self.type_registry);
            let sig = format!("{} {}({});\n", c_return_type, method_name, params_str.join(", "));
            self.ir.forward_decls.push_str(&sig);
            return;
        }

        
        self.current_return_type = Some(method.return_type.clone());

        
        match &method.return_type {
            Type::Result { ok, err } => {
                
                if let Type::Tuple { fields } = ok.as_ref()
                    && let Some(tuple_def) = self.type_registry.generate_tuple_definition(fields, &self.config.arch)
                        && !self.ir.forward_decls.contains(&tuple_def) {
                            self.ir.forward_decls.push_str(&tuple_def);
                            self.ir.forward_decls.push('\n');
                        }
                
                
                if let Type::Tuple { fields } = err.as_ref()
                    && let Some(tuple_def) = self.type_registry.generate_tuple_definition(fields, &self.config.arch)
                        && !self.ir.forward_decls.contains(&tuple_def) {
                            self.ir.forward_decls.push_str(&tuple_def);
                            self.ir.forward_decls.push('\n');
                        }
                
                
                if let Some(result_def) = self.type_registry.generate_result_definition(ok, err, &self.config.arch)
                    && !self.ir.forward_decls.contains(&result_def) {
                        self.ir.forward_decls.push_str(&result_def);
                        self.ir.forward_decls.push('\n');
                    }
            }
            Type::Option { inner } => {
                
                if let Type::Tuple { fields } = inner.as_ref()
                    && let Some(tuple_def) = self.type_registry.generate_tuple_definition(fields, &self.config.arch)
                        && !self.ir.forward_decls.contains(&tuple_def) {
                            self.ir.forward_decls.push_str(&tuple_def);
                            self.ir.forward_decls.push('\n');
                        }
                
                
                if let Some(option_def) = self.type_registry.generate_option_definition(inner, &self.config.arch)
                    && !self.ir.forward_decls.contains(&option_def) {
                        self.ir.forward_decls.push_str(&option_def);
                        self.ir.forward_decls.push('\n');
                    }
            }
            Type::Tuple { fields } => {
                
                if let Some(tuple_def) = self.type_registry.generate_tuple_definition(fields, &self.config.arch)
                    && !self.ir.forward_decls.contains(&tuple_def) {
                        self.ir.forward_decls.push_str(&tuple_def);
                        self.ir.forward_decls.push('\n');
                    }
            }
            _ => {}
        }

        
        let return_c_type = method.return_type.to_c_type(&self.arch, &mut self.type_registry);

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
            let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
            let c_name = name.clone();

            self.vars.insert(name.clone(), (c_name.clone(), ty.clone()));
            
            format!("{} {}", c_type, c_name)
        }).collect();
        
        func_code.push_str(&param_strs.join(", "));
        func_code.push_str(") {\n");
        
        
        let mut body_code = String::new();
        for (i, stmt) in method.body.iter().enumerate() {
            if i == method.body.len() - 1 && !matches!(method.return_type, Type::Void)
                && let Stmt::Expr(expr) = stmt
                    && let Ok((res_var, _)) = self.codegen_expr(expr, &mut body_code) {
                        body_code.push_str(&format!("    return {};\n", res_var));
                        continue;
                    }
            let _ = self.codegen_stmt(stmt, &mut body_code);
        }
        func_code.push_str(&body_code);
        
        
        if matches!(method.return_type, Type::Void)
            && !body_code.contains("return") {
                func_code.push_str("    return;\n");
            }
        
        func_code.push_str("}\n\n");
        
        
        let params_for_registry: Vec<(String, Type)> = method.params.iter().map(|(n, t, _)| (n.clone(), t.clone())).collect();

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
        if !only_signatures {
            self.vars.clear();
            self.var_count = 0;
        }
        let param_types: Vec<(String, Type)> = func.params.iter().map(|(n, t, _)| (n.clone(), t.clone())).collect();
        self.user_functions.insert(func.name.clone(), (param_types, func.return_type.clone()));
        let c_return_type = func.return_type.to_c_type(&self.arch, &mut self.type_registry);
        let c_func_name = if func.name == "main" { 
            "vix_main".to_string() 
        } else {
            func.name.clone() 
        };

         
        let mut params_str = Vec::new();
        let mut func_code = String::new();
        let mut body_code = String::new();
        
        for (p_name, p_ty, _) in &func.params {
            let c_p_type = p_ty.to_c_type(&self.arch, &mut self.type_registry);
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

        self.current_return_type = Some(func.return_type.clone());
        func_code.push_str(&format!("{} {}(", c_return_type, c_func_name));
        func_code.push_str(&params_str.join(", "));
        func_code.push_str(") {\n");

        for (i, stmt) in func.body.iter().enumerate() {
            if i == func.body.len() - 1 && !matches!(func.return_type, Type::Void)
                && let Stmt::Expr(expr) = stmt
                    && let Ok((res_var, _)) = self.codegen_expr(expr, &mut body_code) {
                        body_code.push_str(&format!("    return {};\n", res_var));
                        continue;
                    }
            self.codegen_stmt(stmt, &mut body_code).ok();
        }

        self.current_return_type = None;
        func_code.push_str(&body_code);

        if matches!(func.return_type, Type::Void)
            && !body_code.contains("return") {
                func_code.push_str("    return;\n");
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
                self.codegen_stmt(stmt, body).ok();
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
            self.codegen_stmt(stmt, body).ok();
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

    pub fn codegen_while(&mut self, cond: &Expr, loop_body: &[Stmt], body: &mut String, loc: SourceLocation) -> Result<(), ()> {
        let loop_label = self.fresh_label();
        let end_label = self.fresh_label();
        
        body.push_str(&format!("{}:\n", loop_label));
        
        
        let (cond_var, cond_ty) = match self.codegen_expr(cond, body) {
            Ok(result) => result,
            Err(_) => {
                panic!("WHILE LOOP PANIC: Failed to codegen condition!\n\
                    Condition expr: {:?}\n\
                    Available vars: {:?}\n\
                    Location: {:?}", 
                    cond, 
                    self.vars.keys().collect::<Vec<_>>(),
                    loc);
            }
        };
        
        println!("[DEBUG] While condition generated: var={}, type={:?}", cond_var, cond_ty);
        
        body.push_str(&format!("if (!{}) goto {};\n", cond_var, end_label));

        let mut loop_body_code = String::new();
        println!("[DEBUG] Processing {} statements in while body", loop_body.len());
        for (idx, stmt) in loop_body.iter().enumerate() {
            println!("[DEBUG] While body stmt {}: {:?}", idx, stmt);
            match self.codegen_stmt(stmt, &mut loop_body_code) {
                Ok(_) => {},
                Err(_) => {
                    panic!("WHILE LOOP PANIC: Failed to codegen statement {} in loop body!\n\
                        Statement: {:?}\n\
                        Available vars: {:?}", 
                        idx, stmt, self.vars.keys().collect::<Vec<_>>());
                }
            }
        }
        body.push_str(&loop_body_code);
        
        body.push_str(&format!("goto {};\n", loop_label));
        body.push_str(&format!("{}:\n", end_label));

        Ok(()) 
    }

        pub fn codegen_for(&mut self, var: &str, iter: &Expr, loop_body: &[Stmt], body: &mut String, loc: SourceLocation) -> Result<(), ()> {
        let (iter_var, iter_ty) = self.codegen_expr(iter, body)?;
        println!("[DEBUG] codegen_for: var={}, iter_ty={:?}", var, iter_ty);
        
        let loop_label = self.fresh_label();
        let end_label = self.fresh_label();
        let idx_var = self.fresh_var();
        
        let (size_expr, elem_access, elem_type) = match &iter_ty {
            Type::Str { .. } => {
                (
                    format!("{}.len", iter_var),
                    format!("{}.ptr[{}]", iter_var, idx_var),
                    Type::char8()
                )
            }
            Type::Array { element, size: Some(size) } => {
                (format!("{}", size), format!("{}[{}]", iter_var, idx_var), *element.clone())
            }
            Type::Array { element, size: None } => {
                (format!("{}.len", iter_var), format!("{}.ptr[{}]", iter_var, idx_var), *element.clone())
            }
            _ => {
                ("1".to_string(), format!("{}[{}]", iter_var, idx_var), Type::i32())
            }
        };
        
        let c_name = format!("var_{}", var);
        let c_type = elem_type.to_c_type(&self.arch, &mut self.type_registry);
        
        body.push_str(&format!("size_t {} = 0;\n", idx_var));
        body.push_str(&format!("{}:\n", loop_label));
        body.push_str(&format!("if ({} >= {}) goto {};\n", idx_var, size_expr, end_label));
        body.push_str(&format!("{} {} = {};\n", c_type, c_name, elem_access));
        
        self.vars.insert(var.to_string(), (c_name, elem_type));
        
        for stmt in loop_body {
            self.codegen_stmt(stmt, body).ok();
        }
        
        body.push_str(&format!("{}++;\n", idx_var));
        body.push_str(&format!("goto {};\n", loop_label));
        body.push_str(&format!("{}:\n", end_label));
        
        Ok(())
    }


        pub fn codegen_tuple_unpack(&mut self, names: &[String], value: &Expr, body: &mut String, loc: SourceLocation) -> Result<(), ()> {
            let (val_var, val_ty) = self.codegen_expr(value, body).check_error();
            
            match val_ty {
                Type::Tuple { fields } => {
                    if fields.len() != names.len() {
                        return Err(());
                    }
                    
                    for (i, name) in names.iter().enumerate() {
                        let field_ty = &fields[i];
                        let c_name = format!("var_{}", name);
                        let c_type = field_ty.to_c_type(&self.arch, &mut self.type_registry);
                        
                        body.push_str(&format!("{} {} = {}.field_{};\n", c_type, c_name, val_var, i));
                        self.vars.insert(name.clone(), (c_name, field_ty.clone()));
                    }
                    Ok(())
                }
                _ => Err(())
            }
        }

        pub fn codegen_if_let(
        &mut self, 
        pattern: &Expr, 
        value: &Expr, 
        then_block: &[Stmt], 
        else_block: &Option<Vec<Stmt>>, 
        body: &mut String
    ) -> Result<(), ()> {
        println!("[DEBUG] codegen_if_let called!");
        
        
        let (val_var, val_ty) = self.codegen_expr(value, body)?;
        println!("[DEBUG] val_var={}, val_ty={:?}", val_var, val_ty.name());
        
        
        match pattern {
            Expr::Call(pattern_name, pattern_args) => {
                println!("[DEBUG] Pattern is Call: {}", pattern_name);
                
                match pattern_name.as_str() {
                    "Ok" | "ok" => {
                        self.codegen_if_let_ok_pattern(pattern_args, &val_var, &val_ty, then_block, else_block, body)
                    }
                    "Err" | "err" => {
                        self.codegen_if_let_err_pattern(pattern_args, &val_var, &val_ty, then_block, else_block, body)
                    }
                    "Some" | "some" => {
                        self.codegen_if_let_some_pattern(pattern_args, &val_var, &val_ty, then_block, else_block, body)
                    }
                    _ => {
                        eprintln!("[ERROR] Unknown pattern: {}", pattern_name);
                        Err(())
                    }
                }
            }
            Expr::ResultOk(inner) => {
                println!("[DEBUG] Pattern is ResultOk");
                if let Expr::Var(binding_name) = inner.as_ref() {
                    self.codegen_if_let_ok_pattern(&[Expr::Var(binding_name.clone())], &val_var, &val_ty, then_block, else_block, body)
                } else {
                    eprintln!("[ERROR] ResultOk pattern argument must be a variable");
                    Err(())
                }
            }
            Expr::ResultErr(inner) => {
                println!("[DEBUG] Pattern is ResultErr");
                if let Expr::Var(binding_name) = inner.as_ref() {
                    self.codegen_if_let_err_pattern(&[Expr::Var(binding_name.clone())], &val_var, &val_ty, then_block, else_block, body)
                } else {
                    eprintln!("[ERROR] ResultErr pattern argument must be a variable");
                    Err(())
                }
            }
            Expr::Some(inner) => {
                println!("[DEBUG] Pattern is Some");
                if let Expr::Var(binding_name) = inner.as_ref() {
                    self.codegen_if_let_some_pattern(&[Expr::Var(binding_name.clone())], &val_var, &val_ty, then_block, else_block, body)
                } else {
                    eprintln!("[ERROR] Some pattern argument must be a variable");
                    Err(())
                }
            }
            _ => {
                eprintln!("[ERROR] Pattern is not a Call, ResultOk, ResultErr, or Some expression");
                Err(())
            }
        }
    }

        fn codegen_if_let_ok_pattern(
        &mut self,
        pattern_args: &[Expr],
        val_var: &str,
        val_ty: &Type,
        then_block: &[Stmt],
        else_block: &Option<Vec<Stmt>>,
        body: &mut String
    ) -> Result<(), ()> {
        
        if pattern_args.len() != 1 {
            eprintln!("[ERROR] Ok pattern expects 1 argument");
            return Err(());
        }
        
        let binding_name = if let Expr::Var(name) = &pattern_args[0] {
            name.clone()
        } else {
            eprintln!("[ERROR] Ok pattern argument must be a variable");
            return Err(());
        };
        
        
        let ok_type = if let Type::Result { ok, .. } = val_ty {
            ok.as_ref().clone()
        } else {
            eprintln!("[ERROR] Expected Result type, got {:?}", val_ty.name());
            return Err(());
        };
        
        let ok_c_type = ok_type.to_c_type(&self.arch, &mut self.type_registry);
        
        
        body.push_str(&format!("if ({}.tag == 0) {{\n", val_var));
        body.push_str(&format!("    {} {} = {}.data.ok;\n", ok_c_type, binding_name, val_var));
        
        
        self.vars.insert(binding_name.clone(), (binding_name.clone(), ok_type));
        
        
        for stmt in then_block {
            self.codegen_stmt(stmt, body)?;
        }
        
        body.push('}');
        
        
        if let Some(else_stmts) = else_block {
            body.push_str(" else {\n");
            for stmt in else_stmts {
                self.codegen_stmt(stmt, body)?;
            }
            body.push('}');
        }
        
        body.push('\n');
        Ok(())
    }

        fn codegen_if_let_err_pattern(
        &mut self,
        pattern_args: &[Expr],
        val_var: &str,
        val_ty: &Type,
        then_block: &[Stmt],
        else_block: &Option<Vec<Stmt>>,
        body: &mut String
    ) -> Result<(), ()> {
        
        if pattern_args.len() != 1 {
            return Err(());
        }
        
        let binding_name = if let Expr::Var(name) = &pattern_args[0] {
            name.clone()
        } else {
            return Err(());
        };
        
        let err_type = if let Type::Result { err, .. } = val_ty {
            err.as_ref().clone()
        } else {
            return Err(());
        };
        
        let err_c_type = err_type.to_c_type(&self.arch, &mut self.type_registry);
        
        body.push_str(&format!("if ({}.tag == 1) {{\n", val_var));
        body.push_str(&format!("    {} {} = {}.data.err;\n", err_c_type, binding_name, val_var));
        
        self.vars.insert(binding_name.clone(), (binding_name.clone(), err_type));
        
        for stmt in then_block {
            self.codegen_stmt(stmt, body)?;
        }
        
        body.push('}');
        
        if let Some(else_stmts) = else_block {
            body.push_str(" else {\n");
            for stmt in else_stmts {
                self.codegen_stmt(stmt, body)?;
            }
            body.push('}');
        }
        
        body.push('\n');
        Ok(())
    }

        fn codegen_if_let_some_pattern(
        &mut self,
        pattern_args: &[Expr],
        val_var: &str,
        val_ty: &Type,
        then_block: &[Stmt],
        else_block: &Option<Vec<Stmt>>,
        body: &mut String
    ) -> Result<(), ()> {
        
        if pattern_args.len() != 1 {
            return Err(());
        }
        
        let binding_name = if let Expr::Var(name) = &pattern_args[0] {
            name.clone()
        } else {
            return Err(());
        };
        
        let inner_type = if let Type::Option { inner } = val_ty {
            inner.as_ref().clone()
        } else {
            return Err(());
        };
        
        let inner_c_type = inner_type.to_c_type(&self.arch, &mut self.type_registry);
        
        body.push_str(&format!("if ({}.tag == 1) {{\n", val_var));
        body.push_str(&format!("    {} {} = {}.value;\n", inner_c_type, binding_name, val_var));
        
        self.vars.insert(binding_name.clone(), (binding_name.clone(), inner_type));
        
        for stmt in then_block {
            self.codegen_stmt(stmt, body)?;
        }
        
        body.push('}');
        
        if let Some(else_stmts) = else_block {
            body.push_str(" else {\n");
            for stmt in else_stmts {
                self.codegen_stmt(stmt, body)?;
            }
            body.push('}');
        }
        
        body.push('\n');
        Ok(())
    }
}