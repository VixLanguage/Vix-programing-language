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


    pub fn codegen_typed_declaration_impl(&mut self, name: &str, ty: &Type, value: &Expr, body: &mut String, loc: SourceLocation, is_mutable: bool) -> Result<(), ()> {
        let (val_var, val_ty) = self.codegen_expr(value, body) .check_error();
        let c_name = format!("var_{}", name);
        
        let base_c_type = ty.to_c_type(&self.arch);
        // Apply const if immutable, except where initialization via memcpy requires mutability
        let decl_type = if !is_mutable {
             match ty {
                 Type::Array { size: Some(_), .. } | Type::MultiArray { .. } => base_c_type.clone(),
                 Type::Str { .. } => base_c_type.clone(), 
                 _ => format!("const {}", base_c_type)
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
                
                let elem_c_type = element.to_c_type(&self.arch);
                
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
                
                let elem_c_type = element.to_c_type(&self.arch);
                let struct_decl = if !is_mutable { "const struct" } else { "struct" };
                body.push_str(&format!("{} {{ {}* ptr; size_t len; }} {} = {};\n", struct_decl, elem_c_type, c_name, val_var));
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
                
                let elem_c_type = element.to_c_type(&self.arch);
                let dims_str = dimensions.iter()
                    .map(|d| format!("[{}]", d))
                    .collect::<String>();

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
            
            Type::Union { variants } => {
                for (i, variant) in variants.iter().enumerate() {
                    if matches!(variant, Type::Void) {
                        self.diagnostics.error(
                            "VoidUnionVariant",
                            &format!("Union variant {} in variable '{}' cannot be void", i, name),
                            ErrorContext {
                                primary_location: loc.clone(),
                                secondary_locations: vec![],
                                help_message: Some(format!("Union variants must have concrete types. Variant {} is void.", i)),
                                suggestions: vec![
                                    format!("Change variant {} to a concrete type", i),
                                    "Remove the void variant".to_string(),
                                ],
                            }
                        );
                        return Err(());
                    }
                }
                
                body.push_str(&format!("{} {} = {};\n", decl_type, c_name, val_var));
            }
            
            Type::Ptr(inner) if matches!(**inner, Type::Void) => {
                body.push_str(&format!("{} {} = {};\n", decl_type, c_name, val_var));
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


    pub fn codegen_compound_assign(&mut self, name: &str, op: &str, value: &Expr, body: &mut String, loc: SourceLocation) -> Result<(), ()> {
        let (val_var, val_ty) = self.codegen_expr(value, body) .check_error();
        
        if let Some((c_name, var_ty)) = self.vars.get(name) {
            if matches!(var_ty, Type::Void) || matches!(val_ty, Type::Void) {
                self.diagnostics.error(
                    "VoidOperation",
                    "Cannot perform compound assignment on void type",
                    void_operation_error(op, loc)
                );
                return Err(());
            }
            
            let c_name = c_name.clone();
            body.push_str(&format!("{} {} {};\n", c_name, op, val_var));
            Ok(())
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
            Err(())
        }
    }
    pub fn codegen_call_expr(&mut self, func: &str, args: &[Expr], body: &mut String, loc: SourceLocation) -> Result<(String, Type), ()> {
        match func {
            "print" => {
                let mut format_str = String::new();
                let mut arg_list = Vec::new();
                let mut has_r = false;
                for (i, arg) in args.iter().enumerate() {
                    let (var, ty) = self.codegen_expr(arg, body).check_error();
                    match ty {
                        Type::Int { .. } => {
                            format_str.push_str("%d");
                            arg_list.push(var);
                        }
                        Type::Float { .. } => {
                            format_str.push_str("%f");
                            arg_list.push(var);
                        }
                        Type::Str { .. } => {
                            format_str.push_str("%s");
                            arg_list.push(format!("{}.ptr", var));
                            if let Expr::String(s) = arg { if s.contains('\r') { has_r = true; } }
                        }
                        Type::ConstStr => {
                            format_str.push_str("%s");
                            arg_list.push(var);
                            if let Expr::String(s) = arg { if s.contains('\r') { has_r = true; } }
                        }
                        Type::Bool => {
                            format_str.push_str("%s");
                            arg_list.push(format!("({} ? \"true\" : \"false\")", var));
                        }
                        Type::Char { .. } => {
                            format_str.push_str("%c");
                            arg_list.push(var);
                        }
                        _ => {
                            format_str.push_str("%p");
                            arg_list.push(var);
                        }
                    }
                    if i < args.len() - 1 {
                        format_str.push(' ');
                    }
                }
                if !has_r {
                    format_str.push_str("\\n");
                }
                
                let tmp = self.fresh_var();
                let args_final = if arg_list.is_empty() { String::new() } else { format!(", {}", arg_list.join(", ")) };
                body.push_str(&format!("int32_t {} = printf(\"{}\"{});\n", tmp, format_str, args_final));
                return Ok((tmp, Type::i32()));
            }
            
            "chars" => {
                if args.len() != 1 { return Err(()); }
                self.codegen_chars(&args[0], body)
            }
            "str" | "string" => {
                if args.len() != 1 { return Err(()); }
                let (var, ty) = self.codegen_expr(&args[0], body).check_error();
                match ty {
                    Type::Int { .. } => {
                        let tmp = self.fresh_var();
                        body.push_str(&format!("String {} = vix_int_to_str({});\n", tmp, var));
                        Ok((tmp, Type::Str { len_type: Box::new(Type::i64()) }))
                    }
                    Type::Str { .. } => Ok((var, ty)),
                    Type::ConstStr => {
                        let tmp = self.fresh_var();
                        body.push_str(&format!("String {} = vix_string_from_const({});\n", tmp, var));
                        Ok((tmp, Type::Str { len_type: Box::new(Type::i64()) }))
                    }
                    _ => {
                        // For other types, we might just return the address or something, 
                        // but let's stick to int per plan for now.
                        Err(())
                    }
                }
            }
            "have" | "contain" | "contains" | "has" => {
                if args.len() != 2 { return Err(()); }
                self.codegen_have(&args[0], &args[1], body)
            }
            "is_not_empty" => {
                if args.len() != 1 { return Err(()); }
                self.codegen_is_not_empty(&args[0], body)
            }
            "collect" => {
                if args.len() != 1 { return Err(()); }
                self.codegen_collect(&args[0], body)
            }
            "contain_all" => {
                if args.len() != 2 { return Err(()); }
                if let Expr::Array(items) = &args[1] {
                    self.codegen_contain_all(&args[0], items, body)
                } else {
                    Err(())
                }
            }
            "index" => {
                if args.len() < 2 { return Err(()); }
                let (arr, indices) = args.split_first().unwrap();
                self.codegen_index(arr, indices, body)
            }
            "index_of" => {
                if args.len() != 2 { return Err(()); }
                self.codegen_index_of(&args[0], &args[1], body)
            }

            "array" => {
                self.codegen_array(args, body)
            }
            "some" => {
                if args.len() != 1 { return Err(()); }
                self.codegen_some(&args[0], body)
            }
            "none" => {
                let expected = None;
                self.codegen_none(expected, body)
            }
            "ok" => {
                if args.len() != 1 { return Err(()); }
                self.codegen_result_ok(&args[0], body)
            }
            "err" => {
                if args.len() != 1 { return Err(()); }
                self.codegen_result_err(&args[0], body)
            }
            "unwrap" => {
                if args.len() != 1 { return Err(()); }
                self.codegen_unwrap(&args[0], body)
            }
            "unwrap_or" => {
                if args.len() != 2 { return Err(()); }
                self.codegen_unwrap_or(&args[0], &args[1], body)
            }
            "is_some" | "is_none" => {
                if args.len() != 1 { return Err(()); }
                self.codegen_option_method(&args[0], func, body)
            }
            "array_get" => {
                if args.len() != 2 { return Err(()); }
                self.codegen_array_get(&args[0], &args[1], body)
            }
            "wait" => {
                if args.len() != 1 { return Err(()); }
                self.codegen_wait(&args[0], body)
            }
            "tuple" => {
                self.codegen_tuple(args, body)
            }
            "is_empty" => {
                if args.len() != 1 { return Err(()); }
                self.codegen_is_empty(&args[0], body)
            }
            "filter" => {
                if args.len() != 2 { return Err(()); }
                self.codegen_filter(&args[0], &args[1], body)
            }
            "panic" => {
                if args.len() != 1 { return Err(()); }
                self.codegen_panic(&args[0], body)
            }
            _ => {
                self.codegen_std_call(func, args, body, loc)
            }
        }
    }

    pub fn codegen_member_access(&mut self, obj: &Expr, field: &str, body: &mut String, loc: SourceLocation) -> Result<(String, Type), ()> {
        let (obj_var, obj_ty) = self.codegen_expr(obj, body) .check_error();
        
        if let Type::Str { .. } = &obj_ty {
            let field_ty = if field == "ptr" {
                Type::ConstStr
            } else if field == "len" {
                Type::i64()
            } else {
                return Err(());
            };
            let tmp = self.fresh_var();
            let c_type = field_ty.to_c_type(&self.arch);
            body.push_str(&format!("{} {} = {}.{};\n", c_type, tmp, obj_var, field));
            return Ok((tmp, field_ty));
        }

        let struct_name = match &obj_ty {
            Type::Struct { name } => name.clone(),
            Type::Ref(inner) | Type::MutRef(inner) | Type::Ptr(inner) => {
                if let Type::Struct { name } = &**inner {
                    name.clone()
                } else {
                    return Err(());
                }
            }
            _ => return Err(()),
        };

        let field_ty = if let Some(struct_info) = self.structs.get(&struct_name) {
            if let Some(field_info) = struct_info.fields.iter().find(|f| f.0 == field) {
                field_info.1.clone()
            } else {
                Type::i32()  
            }
        } else {
            Type::i32()  
        };

        let tmp = self.fresh_var();
        let op = if matches!(obj_ty, Type::Ref(_) | Type::MutRef(_) | Type::Ptr(_)) { "->" } else { "." };
        
        let c_type = field_ty.to_c_type(&self.arch);
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
        let (arr_var, _arr_ty) = self.codegen_expr(arr, body).check_error();
        let (val_var, _val_ty) = self.codegen_expr(value, body).check_error();
        
        let mut index_str = arr_var.clone();
        
        for idx in indices {
            let (idx_var, _idx_ty) = self.codegen_expr(idx, body).check_error();
            index_str = format!("{}[{}]", index_str, idx_var);
        }
        
        body.push_str(&format!("{} = {};\n", index_str, val_var));

        Ok(())
    }

    pub fn codegen_member_assign(&mut self, obj: &Expr, field: &str, value: &Expr, body: &mut String, _loc: SourceLocation) -> Result<(), ()> {
        let (obj_var, obj_ty) = self.codegen_expr(obj, body).check_error();
        let (val_var, _val_ty) = self.codegen_expr(value, body).check_error();

        let op = if matches!(obj_ty, Type::Ref(_) | Type::MutRef(_) | Type::Ptr(_)) { "->" } else { "." };
        body.push_str(&format!("{}{}{} = {};\n", obj_var, op, field, val_var));

        Ok(())
    }

    pub fn codegen_call_stmt(&mut self, func: &str, args: &[Expr], body: &mut String, loc: SourceLocation) -> Result<(), ()> {
        if func == "print" {
            self.codegen_call_expr(func, args, body, loc).check_error();
            return Ok(());
        }

        let mut arg_vars = Vec::new();
        
         
        let param_types = if let Some(ext_info) = self.extern_functions.get(func) {
            Some(ext_info.params.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>())
        } else if let Some((params, _)) = self.user_functions.get(func) {
            Some(params.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>())
        } else {
            None
        };

        for (i, arg) in args.iter().enumerate() {
            let (mut var, ty) = self.codegen_expr(arg, body).check_error();
            
             
            if let Some(params) = &param_types {
                if let Some(param_ty) = params.get(i) {
                    if (matches!(param_ty, Type::ConstStr) || matches!(param_ty, Type::Ptr(_) | Type::RawPtr(_) | Type::Ref(_) | Type::MutRef(_))) && matches!(ty, Type::Str { .. }) {
                        var = format!("{}.ptr", var);
                    }
                }
            }
            
            arg_vars.push(var);
        }

        let args_str = arg_vars.join(", ");
        body.push_str(&format!("{}({});\n", func, args_str));

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
         
        let method_name = format!("{}_{}", type_name, method);
        
         
        let mut arg_vars = Vec::new();
        for arg in args {
            let (var, _ty) = self.codegen_expr(arg, body).check_error();
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
                    help_message: Some(format!("[DEBUG] Method '{}' does not exist for type '{}'", method, type_name)),
                    suggestions: vec![
                        format!("Check if '{}' is defined in the impl block for '{}'", method, type_name),
                        "Verify the method name is spelled correctly".to_string(),
                    ],
                }
            );
            return Err(());
        };
        
        let c_type = return_type.to_c_type(&self.arch);
        
         
        body.push_str(&format!("{} {} = {}({});\n", c_type, tmp, method_name, args_str));
        
        Ok((tmp, return_type))
    }

    
    pub fn codegen_cast(&mut self, expr: &Expr, target_ty: &Type, body: &mut String, loc: SourceLocation) -> Result<(String, Type), ()> {
        let (var, source_ty) = self.codegen_expr(expr, body) .check_error();

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
        
        let c_type = target_ty.to_c_type(&self.arch);
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
            (Type::Str { .. }, Type::ConstStr) => true,
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
        let (obj_var, obj_ty) = self.codegen_expr(obj, body)
            .map_err(|_| ())?;
        let (val_var, val_ty) = self.codegen_expr(value, body)
            .map_err(|_| ())?;

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

         
        if op == "+=" && matches!(val_ty, Type::Str { .. }) {
            if let Type::Struct { name: struct_name } = struct_ty {
                if let Some(struct_info) = self.structs.get(struct_name) {
                    if let Some((_, field_ty, _)) = struct_info.fields.iter()
                        .find(|(fname, _, _)| fname == field) 
                    {
                        if matches!(field_ty, Type::Str { .. }) {
                            let tmp = self.fresh_var();
                            
                             
                            if !self.ir.headers.contains("#include <string.h>") {
                                self.ir.headers.push_str("#include <string.h>\n");
                            }
                            if !self.ir.headers.contains("#include <stdlib.h>") {
                                self.ir.headers.push_str("#include <stdlib.h>\n");
                            }
                            
                             
                            body.push_str(&format!(
                                "size_t {}_len = strlen({}{}{}.ptr) + strlen({}.ptr) + 1;\n", 
                                tmp, obj_var, access_op, field, val_var
                            ));
                            body.push_str(&format!(
                                "char* {}_new = malloc({}_len);\n", 
                                tmp, tmp
                            ));
                            body.push_str(&format!(
                                "strcpy({}_new, {}{}{}.ptr);\n", 
                                tmp, obj_var, access_op, field
                            ));
                            body.push_str(&format!(
                                "strcat({}_new, {}.ptr);\n", 
                                tmp, val_var
                            ));
                            body.push_str(&format!(
                                "{}{}{}.ptr = {}_new;\n", 
                                obj_var, access_op, field, tmp
                            ));
                            body.push_str(&format!(
                                "{}{}{}.len = strlen({}_new);\n", 
                                obj_var, access_op, field, tmp
                            ));
                            return Ok(());
                        }
                    }
                }
            }
        }

         
        body.push_str(&format!("{}{}{} {} {};\n", obj_var, access_op, field, op, val_var));
        Ok(())
    }


}
