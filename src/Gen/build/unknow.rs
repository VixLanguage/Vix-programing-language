use crate::import::*;

impl Codegen {
        pub fn codegen_assign(
        &mut self,
        name: &str,
        value: &Expr,
        body: &mut String,
        loc: SourceLocation,
    ) -> Result<(), ()> {
         
        let (c_name, var_ty) = if let Some((c, t)) = self.vars.get(name) {
            (c.clone(), t.clone())
        } else {
            self.diagnostics.error(
                "UndefinedVariable",
                &format!("Cannot assign to undefined variable '{}'.", name),
                ErrorContext {
                    primary_location: loc.clone(),
                    secondary_locations: vec![],
                    help_message: Some(format!(
                        "Variable '{}' must be declared before assignment.", 
                        name
                    )),
                    suggestions: vec![
                        format!("Declare '{}' before assignment", name),
                        format!("Use 'let {} = ...' to declare and initialize", name),
                    ],
                }
            );
            return Err(());
        };
        
        let (val_var, val_ty) = self.codegen_expr(value, body)?;
        
         
        match (&var_ty, &val_ty) {
            (Type::ConstStr { .. }, Type::Str { .. }) => {
                 
                body.push_str(&format!("{} = {}.ptr;\n", c_name, val_var));
            }
            _ => {
                 
                body.push_str(&format!("{} = {};\n", c_name, val_var));
            }
        }
        
        Ok(())
    }

    pub fn codegen_compound_assign(&mut self, name: &str, op: &str, value: &Expr, body: &mut String, loc: SourceLocation) -> Result<(), ()> {
        let (val_var, val_ty) = self.codegen_expr(value, body)?;
        
        let (c_name, var_ty) = if let Some((c, t)) = self.vars.get(name) {
            (c.clone(), t.clone())
        } else {
            self.diagnostics.error(
                "UndefinedVariable",
                &format!("Variable '{}' is not defined", name),
                ErrorContext {
                    primary_location: loc,
                    secondary_locations: vec![],
                    help_message: Some(format!("Cannot perform compound assignment on undefined variable '{}'.", name)),
                    suggestions: vec![format!("Declare '{}' before using compound assignment", name)],
                }
            );
            return Err(());
        };

        if matches!(var_ty, Type::Void) || matches!(val_ty, Type::Void) {
            self.diagnostics.error(
                "VoidOperation",
                "Cannot perform compound assignment on void type",
                void_operation_error(op, loc)
            );
            return Err(());
        }

        if op == "+=" {
            match (&var_ty, &val_ty) {
                 
                (Type::Array { element: arr_elem, .. }, Type::Array { element: val_elem, .. }) => {
                    if self.types_compatible(arr_elem, val_elem) {
                        self.codegen_extend_unified(&c_name, &val_var, body);
                        return Ok(());
                    }

                     
                     
                     
                     
                     
                     if (matches!(**arr_elem, Type::Str { .. }) && matches!(**val_elem, Type::StdStr)) ||
                        (matches!(**arr_elem, Type::StdStr) && matches!(**val_elem, Type::Str { .. } | Type::ConstStr)) {
                             self.codegen_extend_unified(&c_name, &val_var, body);
                             return Ok(());
                     }

                    self.diagnostics.error(
                        "TypeMismatch",
                        &format!("Cannot append {}[] to {}[]", val_elem.name(), arr_elem.name()),
                         ErrorContext {
                            primary_location: loc,
                            secondary_locations: vec![],
                             help_message: Some("Array elements must have compatible types".to_string()),
                            suggestions: vec!["Convert elements to matching type".to_string()],
                        }
                    );
                    return Err(());
                }
                
                 
                (Type::Array { element: arr_elem, .. }, elem_ty) => {
                     
                    if self.types_compatible(arr_elem, elem_ty) {
                        self.codegen_push_unified(&c_name, &val_var, body);
                         return Ok(());
                    }
                    
                     
                    if matches!(**arr_elem, Type::Str { .. }) && matches!(*elem_ty, Type::StdStr) {
                          
                          
                          
                          
                          
                         self.codegen_push_unified(&c_name, &val_var, body);
                         return Ok(());
                    }
                }

                 
                (Type::Str { .. }, Type::Str { .. } | Type::ConstStr) => {
                     self.codegen_str_append_zero_alloc(&c_name, &val_var, body);
                     return Ok(());
                }

                (Type::Str { .. }, Type::StdStr) => {
                      
                      
                      
                     self.codegen_str_append_zero_alloc(&c_name, &val_var, body);
                     return Ok(());
                }
                (Type::StdStr, Type::Str { .. } | Type::ConstStr | Type::StdStr) => {
                    self.codegen_str_append_zero_alloc(&c_name, &val_var, body);
                    return Ok(());
                }

                _ => {}
            }
        }
        
        body.push_str(&format!("{} {} {};\n", c_name, op, val_var));
        Ok(())
    }

    pub fn codegen_call_expr(&mut self, func: &str, args: &[Expr], body: &mut String, loc: SourceLocation) -> Result<(String, Type), ()> {
        eprintln!("[DEBUG] codegen_call_expr: original func_name={}", func);
        
        
        let resolved_func = self.resolve_function_name(func);
        
        if resolved_func != func {
            eprintln!("[DEBUG] codegen_call_expr: resolved {} -> {}", func, resolved_func);
        }

        
        if self.structs.contains_key(&resolved_func) {
            let constructor_name = format!("{}_new", resolved_func);
            
            let mut arg_vars = Vec::new();
            
            if args.is_empty() {
                if let Some(struct_info) = self.structs.get(&resolved_func) {
                    for (_, field_ty, _) in &struct_info.fields {
                        let default_val = match field_ty {
                            Type::Int { .. } => "0",
                            Type::Float { .. } => "0.0",
                            Type::Bool => "false",
                            Type::Str { .. } => "((Slice_char){ .ptr = \"\", .len = 0 })",
                            Type::ConstStr => "\"\"",
                            _ => "{ 0 }",
                        };
                        arg_vars.push(default_val.to_string());
                    }
                }
            } else {
                for arg in args {
                    let (var, _ty) = self.codegen_expr(arg, body)?;
                    arg_vars.push(var);
                }
            }
            
            let tmp = self.fresh_var();
            let args_str = arg_vars.join(", ");
            
            body.push_str(&format!("{} {} = {}({});\n", resolved_func, tmp, constructor_name, args_str));
            return Ok((tmp, Type::Struct { name: resolved_func.to_string() }));
        }

        
        if self.linked_libraries.contains(&func.to_string()) {
            self.diagnostics.error(
                "InvalidLibraryCall",
                &format!("Cannot call library '{}' directly", func),
                ErrorContext {
                    primary_location: loc.clone(),
                    secondary_locations: vec![],
                    help_message: Some(format!(
                        "Library '{}' is an import, not a function. Use '{}.<module>.<function>()' to call functions from this library.",
                        func, func
                    )),
                    suggestions: vec![
                        format!("Use '{}.ModuleName.function_name()' instead", func),
                        "Check the library's exported modules and functions".to_string(),
                    ],
                }
            );
            return Err(());
        }

        
        match func {
            "as_bytes" => {
                if args.len() != 1 { return Err(()); }
                self.ensure_type_defined(&Type::u8());
                let (obj_var, _obj_ty) = self.codegen_expr(&args[0], body)?;
                let tmp = self.fresh_var();
                let slice_ty = Type::Array { element: Box::new(Type::u8()), size: None };
                let slice_name = slice_ty.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("{} {} = {{ .ptr = (uint8_t*){}.ptr, .len = {}.len }};\n", 
                    slice_name, tmp, obj_var, obj_var));
                return Ok((tmp, slice_ty));
            }
            "as_ptr" => {
                if args.len() != 1 { return Err(()); }
                let (obj_var, obj_ty) = self.codegen_expr(&args[0], body)?;
                let tmp = self.fresh_var();
                let inner_type = match &obj_ty {
                    Type::Array { element, .. } => *element.clone(),
                    Type::Str { .. } => Type::char8(),
                    _ => Type::Void,
                };
                body.push_str(&format!("const {}* {} = {}.ptr;\n", 
                    inner_type.to_c_type(&self.arch, &mut self.type_registry), tmp, obj_var));
                return Ok((tmp, Type::Ptr(Box::new(Type::Const(Box::new(inner_type))))));
            }
            "as_mut_ptr" => {
                if args.len() != 1 { return Err(()); }
                let (obj_var, obj_ty) = self.codegen_expr(&args[0], body)?;
                let tmp = self.fresh_var();
                let inner_type = match &obj_ty {
                    Type::Array { element, .. } => *element.clone(),
                    Type::Str { .. } => Type::char8(),
                    _ => Type::Void,
                };
                body.push_str(&format!("{}* {} = {}.ptr;\n", 
                    inner_type.to_c_type(&self.arch, &mut self.type_registry), tmp, obj_var));
                return Ok((tmp, Type::MutRef(Box::new(inner_type))));
            }
            "size_of" | "sizeof" => {
                if args.len() != 1 { return Err(()); }
                let (var, _ty) = self.codegen_expr(&args[0], body)?;
                let tmp = self.fresh_var();
                body.push_str(&format!("size_t {} = sizeof({});\n", tmp, var));
                return Ok((tmp, Type::Int { bits: 64, signed: false }));
            }
            _ => {
                
                
                let param_types = if let Some(ext_info) = self.extern_functions.get(&resolved_func) {
                    Some(ext_info.params.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>())
                } else if let Some((params, _)) = self.user_functions.get(&resolved_func) {
                    Some(params.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>())
                } else {
                    None
                };

                let mut arg_vars = Vec::new();
                
                for (i, arg) in args.iter().enumerate() {
                    let (mut var, ty) = self.codegen_expr(arg, body)?;
                    
                    
                    if let Some(params) = &param_types {
                        if let Some(param_ty) = params.get(i) {
                            let needs_ptr = match param_ty {
                                Type::ConstStr => true,
                                Type::Ptr(inner) => {
                                    matches!(inner.as_ref(), Type::Const(t) if matches!(t.as_ref(), Type::Char { .. }))
                                }
                                _ => false
                            };
                            
                            if needs_ptr && matches!(ty, Type::Str { .. }) {
                                var = format!("{}.ptr", var);
                            }
                        }
                    } else {
                        
                        if matches!(ty, Type::Str { .. }) {
                            var = format!("{}.ptr", var);
                        }
                    }
                    
                    arg_vars.push(var);
                }

                
                self.codegen_std_call(&resolved_func, args, body, loc)
            }
        }
    }

    pub fn codegen_member_access(&mut self, obj: &Expr, field: &str, body: &mut String) -> Result<(String, Type), ()> {
        let (obj_var, obj_ty) = self.codegen_expr(obj, body)?;
        
        let struct_name = match &obj_ty {
            Type::Struct { name } => name.clone(),
            Type::Ref(inner) | Type::MutRef(inner) => {
                if let Type::Struct { name } = &**inner {
                    name.clone()
                } else {
                    return Err(());
                }
            }
            _ => return Err(()),
        };

         
        let field_ty = if let Some(struct_info) = self.structs.get(&struct_name) {
            struct_info.fields.iter()
                .find(|f| f.0 == field)
                .map(|f| f.1.clone())
                .unwrap_or(Type::i32())
        } else {
            Type::i32()
        };

        let tmp = self.fresh_var();
        let op = if matches!(obj_ty, Type::Ref(_) | Type::MutRef(_)) { "->" } else { "." };
        
        let c_type = field_ty.to_c_type(&self.arch, &mut self.type_registry);
        body.push_str(&format!("{} {} = {}{}{};\n", c_type, tmp, obj_var, op, field));
        Ok((tmp, field_ty))
    }
    pub fn codegen_cast_target(&mut self, expr: &Expr, target: &CastTarget, body: &mut String, loc: SourceLocation) -> Result<(String, Type), ()> {
        if let CastTarget::Type(ty) = target {
            self.codegen_cast(expr, ty, body, loc)
        } else {
            Err(())
        }
    }

    pub fn codegen_index_assign(&mut self, arr: &Expr, indices: &[Expr], value: &Expr, body: &mut String, _loc: SourceLocation) -> Result<(), ()> {
        let (arr_var, arr_ty) = self.codegen_expr(arr, body)?;
        let (val_var, _val_ty) = self.codegen_expr(value, body)?;
        
         
        let is_slice = matches!(arr_ty, Type::Array { size: None, .. } | Type::Str { .. });
        
        let mut index_str = if is_slice {
            format!("{}.ptr", arr_var)
        } else {
            arr_var.clone()
        };
        
        for idx in indices {
            let (idx_var, _idx_ty) = self.codegen_expr(idx, body)?;
            index_str = format!("{}[{}]", index_str, idx_var);
        }
        
        body.push_str(&format!("{} = {};\n", index_str, val_var));

        Ok(())
    }

    pub fn codegen_member_assign(&mut self, obj: &Expr, field: &str, value: &Expr, body: &mut String, _loc: SourceLocation) -> Result<(), ()> {
        let (obj_var, obj_ty) = self.codegen_expr(obj, body)?;
        let (val_var, _val_ty) = self.codegen_expr(value, body)?;

        let op = if matches!(obj_ty, Type::Ref(_) | Type::MutRef(_)) { "->" } else { "." };
        body.push_str(&format!("{}{}{} = {};\n", obj_var, op, field, val_var));

        Ok(())
    }


    pub fn codegen_call_stmt(&mut self, func: &str, args: &[Expr], body: &mut String, loc: SourceLocation) -> Result<(), ()> {
    eprintln!("[DEBUG] codegen_call_stmt: original func_name={}", func);
    
     
    let resolved_func = self.resolve_function_name(func);
    
    if resolved_func != func {
        eprintln!("[DEBUG] codegen_call_stmt: resolved {} -> {}", func, resolved_func);
    }
    
    let mut arg_vars = Vec::new();
    
     
    let param_types = if let Some(ext_info) = self.extern_functions.get(&resolved_func) {
        Some(ext_info.params.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>())
    } else if let Some((params, _)) = self.user_functions.get(&resolved_func) {
        Some(params.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>())
    } else {
        None
    };

    for (i, arg) in args.iter().enumerate() {
        let (mut var, ty) = self.codegen_expr(arg, body)?;
        
         
        if let Some(params) = &param_types {
            if let Some(param_ty) = params.get(i) {
                let needs_ptr = match param_ty {
                    Type::ConstStr => true,
                    Type::Ptr(inner) => {
                        matches!(inner.as_ref(), Type::Const(t) if matches!(t.as_ref(), Type::Char { .. }))
                    }
                    _ => false
                };
                
                if needs_ptr && matches!(ty, Type::Str { .. }) {
                    var = format!("{}.ptr", var);
                }
            }
        } else {
             
            if matches!(ty, Type::Str { .. }) {
                var = format!("{}.ptr", var);
            }
        }
        
        arg_vars.push(var);
    }

    let args_str = arg_vars.join(", ");
    
     
    body.push_str(&format!("{}({});\n", resolved_func, args_str));

    Ok(())
}

    pub fn codegen_program(&mut self, functions: &[Function]) -> Result<(), ()> {
        for func in functions {
            self.codegen_function(func, false)
        }
 
        Ok(())
    }

    pub fn finalize(self) -> Result<String, ()> {
        if self.diagnostics.has_errors() {
            self.diagnostics.print_summary();
            Err(())
        } else {
            Ok(self.ir.finalize())
        }
    }

    pub fn codegen_static_method(
        &mut self, 
        type_name: &str, 
        method: &str, 
        args: &[Expr], 
        body: &mut String, 
        loc: SourceLocation
    ) -> Result<(String, Type), ()> {
        if method == "new" {
            println!("[DEBUG] Generating constructor call for {}", type_name);

            let constructor_name = format!("{}_new", type_name);

            let mut arg_vars = Vec::new();
            for arg in args {
                let (var, _ty) = self.codegen_expr(arg, body)?;
                arg_vars.push(var);
            }
            
            let tmp = self.fresh_var();
            let args_str = arg_vars.join(", ");

            body.push_str(&format!("{} {} = {}({});\n", type_name, tmp, constructor_name, args_str));
            
            return Ok((tmp, Type::Struct { name: type_name.to_string() }));
        }

        let mut arg_vars = Vec::new();
        let method_name = format!("{}_{}", type_name, method);
        for arg in args {
            let (var, _ty) = self.codegen_expr(arg, body)?;
            arg_vars.push(var);
        }
        
        let tmp = self.fresh_var();
        let args_str = arg_vars.join(", ");
        let return_type = if let Some((_, ret_ty, _)) = self.impl_methods.get(&(type_name.to_string(), method.to_string())) {
            ret_ty.clone()
        } else if let Some((_, ret_ty)) = self.user_functions.get(&method_name) {
            ret_ty.clone()
        } else {
            self.diagnostics.error(
                "UndefinedMethod",
                &format!("Static method '{}::{}' is not defined", type_name, method),
                ErrorContext {
                    primary_location: loc,
                    secondary_locations: vec![],
                    help_message: Some(format!("Method '{}' does not exist for type '{}'", method, type_name)),
                    suggestions: vec![
                        format!("Check if '{}' is defined in the impl block for '{}'", method, type_name),
                        "Verify the method name is spelled correctly".to_string(),
                    ],
                }
            );
            return Err(());
        };
        
        let c_type = return_type.to_c_type(&self.arch, &mut self.type_registry);

        body.push_str(&format!("{} {} = {}({});\n", c_type, tmp, method_name, args_str));
        
        Ok((tmp, return_type))
    }
        
    pub fn codegen_cast(&mut self, expr: &Expr, target_ty: &Type, body: &mut String, loc: SourceLocation) -> Result<(String, Type), ()> {
        let (var, source_ty) = self.codegen_expr(expr, body) ?;

        if matches!(source_ty, Type::Ptr(_)) && !matches!(target_ty, Type::Ptr(_) | Type::RawPtr(_)) {
            self.diagnostics.warning(
                "UnsafeCast",
                "Casting pointer to non-pointer type may be unsafe",
                ErrorContext {
                    primary_location: loc.clone(),
                    secondary_locations: vec![],
                    help_message: Some("This cast may lose pointer information.".to_string()),
                    suggestions: vec!["Ensure this cast is intentional".to_string()],
                }
            );
        }
        
        let c_type = target_ty.to_c_type(&self.arch, &mut self.type_registry);
        let tmp = self.fresh_var();

        body.push_str(&format!("{} {} = ({}){};\n", c_type, tmp, c_type, var));
        Ok((tmp, target_ty.clone()))
    }
    pub fn types_compatible(&self, ty1: &Type, ty2: &Type) -> bool {
        match (ty1, ty2) {
            (Type::Int { bits: b1, signed: s1 }, Type::Int { bits: b2, signed: s2 }) => b1 == b2 && s1 == s2,
            (Type::Float { bits: b1 }, Type::Float { bits: b2 }) => b1 == b2,
            (Type::Bool, Type::Bool) => true,
            (Type::Void, Type::Void) => true,
            (Type::Char { .. }, Type::Char { .. }) => true,
            (Type::Str { .. }, Type::Str { .. }) => true,
            (Type::ConstStr, Type::ConstStr) => true,
            (Type::ConstStr, Type::Str { .. }) => true,
            (Type::ConstStr, Type::StdStr) => true,
            (Type::Str { .. }, Type::ConstStr) => true,
            (Type::StdStr, Type::ConstStr) => true,
            (Type::StdStr, Type::StdStr) => true,
            (Type::Str { .. }, Type::StdStr) => true,
            (Type::StdStr, Type::Str { .. }) => true,
            (Type::Ptr(inner1), Type::Ptr(inner2)) => self.types_compatible(inner1, inner2),
            (Type::Struct { name: n1 }, Type::Struct { name: n2 }) => n1 == n2,
            (Type::Array { element: e1, size: s1 }, Type::Array { element: e2, size: s2 }) => {
                self.types_compatible(e1, e2) && (s1 == s2)
            },
            (Type::Tuple { fields: f1 }, Type::Tuple { fields: f2 }) => {
                if f1.len() != f2.len() { return false; }
                f1.iter().zip(f2.iter()).all(|(t1, t2)| self.types_compatible(t1, t2))
            },
            (Type::Option { inner: i1 }, Type::Option { inner: i2 }) => self.types_compatible(i1, i2),
            (Type::Result { ok: o1, err: e1 }, Type::Result { ok: o2, err: e2 }) => {
                self.types_compatible(o1, o2) && self.types_compatible(e1, e2)
            },
            (Type::Const(i1), Type::Const(i2)) => self.types_compatible(i1, i2),
            (Type::Const(i1), other) => self.types_compatible(i1, other),
            (other, Type::Const(i2)) => self.types_compatible(other, i2),
            (Type::HashMap { key: k1, value: v1 }, Type::HashMap { key: k2, value: v2 }) => {
                self.types_compatible(k1, k2) && self.types_compatible(v1, v2)
            },
            _ => false, 
        }
    }

    pub fn codegen_member_compound_assign(
        &mut self, 
        obj: &Expr, 
        field: &str, 
        op: &str, 
        value: &Expr, 
        body: &mut String, 
        loc: SourceLocation
    ) -> Result<(), ()> {
        let (obj_var, obj_ty) = self.codegen_expr(obj, body).map_err(|_| ())?;
        let (val_var, val_ty) = self.codegen_expr(value, body).map_err(|_| ())?;

        if matches!(obj_ty, Type::Void) {
            self.diagnostics.error(
                "VoidMemberAssign",
                "Cannot perform compound assignment on member of void type",
                ErrorContext {
                    primary_location: loc,
                    secondary_locations: vec![],
                    help_message: Some("Cannot modify void type members".to_string()),
                    suggestions: vec![],
                }
            );
            return Err(());
        }

        let struct_ty = match &obj_ty {
            Type::Ref(inner) | Type::MutRef(inner) => inner.as_ref(),
            _ => &obj_ty,
        };

        let access_op = if matches!(obj_ty, Type::Ref(_) | Type::MutRef(_)) {
            "->"
        } else {
            "."
        };

        if op == "+=" {
            if let Type::Struct { name: struct_name } = struct_ty {
                if let Some(struct_info) = self.structs.get(struct_name) {
                    if let Some((_, field_ty, _)) = struct_info.fields.iter()
                        .find(|(fname, _, _)| fname == field) 
                    {
                        if matches!(field_ty, Type::StdStr) {
                             self.diagnostics.error(
                                 "UnsupportedFeature",
                                 "String member compound assignment is not supported in No-OS mode.",
                                 ErrorContext {
                                     primary_location: loc,
                                     secondary_locations: vec![],
                                     help_message: Some("String type is deprecated.".to_string()),
                                     suggestions: vec![],
                                 }
                             );
                             return Err(());
                        }

                    }
                }
            }
        }

        body.push_str(&format!("{}{}{} {} {};\n", obj_var, access_op, field, op, val_var));
        Ok(())
    }
}
