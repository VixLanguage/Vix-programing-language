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
            (Type::ConstStr, Type::Str { .. }) => {
                 
                body.push_str(&format!("{} = {}.ptr;\n", c_name, val_var));
            }
            _ => {
                 
                body.push_str(&format!("{} = {};\n", c_name, val_var));
            }
        }
        
        Ok(())
    }



    pub fn codegen_compound_assign(&mut self, name: &str, op: &str, value: &Expr, body: &mut String, loc: SourceLocation) -> Result<(), ()> {
        let (val_var, val_ty) = self.codegen_expr(value, body).check_error();
        
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

            if op == "+=" {
                match (&var_ty, &val_ty) {
                    (Type::Array { element: arr_elem, .. }, Type::Array { element: val_elem, .. }) => {
                        if !self.types_compatible(arr_elem, val_elem) {
                            self.diagnostics.error(
                                "TypeMismatch",
                                &format!("Cannot append {}[] to {}[]", val_elem.name(), arr_elem.name()),
                                ErrorContext {
                                    primary_location: loc,
                                    secondary_locations: vec![],
                                    help_message: Some("Array elements must have the same type".to_string()),
                                    suggestions: vec![
                                        "Convert elements to matching type".to_string(),
                                    ],
                                }
                            );
                            return Err(());
                        }
                        
                        let c_name_clone = c_name.clone();
                        let arr_elem_clone = arr_elem.clone();

                        self.codegen_extend(&c_name_clone, &val_var, &arr_elem_clone, body);
                        return Ok(());
                    }

                    (Type::StdStr, Type::Str { .. } | Type::ConstStr) => {
                        body.push_str(&format!("String_push_str(&{}, {});\n", c_name, val_var));
                        return Ok(());
                    }
                    (Type::StdStr, Type::StdStr) => {
                        body.push_str(&format!("String_push_str(&{}, String_as_str(&{}));\n", c_name, val_var));
                        return Ok(());
                    }
                    (Type::StdStr, Type::Char { .. }) => {
                        body.push_str(&format!("String_push(&{}, {});\n", c_name, val_var));
                        return Ok(());
                    }
                    
                    _ => {}
                }
            }
            
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
        eprintln!("[DEBUG] codegen_call_expr: func_name={}", func);

        if self.structs.contains_key(func) {
            let constructor_name = format!("{}_new", func);
            
            let mut arg_vars = Vec::new();
            
            if args.is_empty() {
                if let Some(struct_info) = self.structs.get(func) {
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
                    let (var, _ty) = self.codegen_expr(arg, body).check_error();
                    arg_vars.push(var);
                }
            }
            
            let tmp = self.fresh_var();
            let args_str = arg_vars.join(", ");
            
            body.push_str(&format!("{} {} = {}({});\n", func, tmp, constructor_name, args_str));
            return Ok((tmp, Type::Struct { name: func.to_string() }));
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
                        Type::StdStr => {
                            format_str.push_str("%.*s");
                            arg_list.push(format!("(int){}.len", var));
                            arg_list.push(format!("{}.ptr", var));
                        }
                        Type::Float { .. } => {
                            format_str.push_str("%f");
                            arg_list.push(var);
                        }
                        Type::Str { .. } => {
                            format_str.push_str("%s");
                            arg_list.push(format!("{}.ptr", var));
                            if let Expr::String(s) = arg && s.contains('\r') { has_r = true; }
                        }
                        Type::ConstStr => {
                            format_str.push_str("%s");
                            arg_list.push(var);
                            if let Expr::String(s) = arg && s.contains('\r') { has_r = true; }
                        }
                        Type::Bool => {
                            format_str.push_str("%s");
                            arg_list.push(format!("({} ? \"true\" : \"false\")", var));
                        }
                        Type::Char { .. } => {
                            format_str.push_str("%c");
                            arg_list
                            .push(var);
                        }
                        Type::Array { element, .. } => {
                            let loop_var = self.fresh_var();
                            body.push_str(&format!("for (size_t {} = 0; {} < {}.len; {}++) {{\n", loop_var, loop_var, var, loop_var));
                            body.push_str(&format!("  if ({} > 0) printf(\" \");\n", loop_var));

                            match element.as_ref() {
                                Type::Str { .. } | Type::ConstStr => {
                                    body.push_str(&format!("  printf(\"%.*s\", (int){}.ptr[{}].len, {}.ptr[{}].ptr);\n",  var, loop_var, var, loop_var));
                                }
                                Type::Int { .. } => {
                                    body.push_str(&format!("  printf(\"%d\", {}.ptr[{}]);\n", var, loop_var));
                                }
                                _ => {
                                    body.push_str(&format!("printf(\"%p\", {}.ptr[{}]);\n", var, loop_var));
                                }
                            }
                            body.push_str("}\n");
                            continue;  
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
                Ok((tmp, Type::i32()))
            }
            "format" => {
                let mut arg_vars = Vec::new();
                for arg in args {
                    let (var, ty) = self.codegen_expr(arg, body).check_error();
                    match ty {
                        Type::Str { .. } => arg_vars.push(var),  
                        Type::ConstStr => arg_vars.push(format!("vix_string_from_const({})", var)),
                        _ => arg_vars.push(var),
                    }
                }
                let tmp = self.fresh_var();
                let args_str = arg_vars.join(", ");
                let ty = Type::Str { len_type: Box::new(Type::i64()) };
                let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("{} {} = vix_format({});\n", c_type, tmp, args_str));
                Ok((tmp, ty))
            }
            
            "chars" => {
                if args.len() != 1 { return Err(()); }
                self.codegen_chars(&args[0], body)
            }

            "str" | "string" | "String" => {
                if args.len() != 1 { return Err(()); }
                self.ensure_string_typedef();
                self.ensure_to_string_helpers();
                let (var, ty) = self.codegen_expr(&args[0], body).check_error();
                eprintln!("[DEBUG] string() arg type: {:?}", ty.name());
                match ty {
                    Type::Int { .. } => {
                        let tmp = self.fresh_var();
                        let ty = Type::Str { len_type: Box::new(Type::i64()) };
                        let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                        body.push_str(&format!("{} {} = vix_int_to_str({});\n", c_type, tmp, var));
                        Ok((tmp, ty))
                    }
                    Type::Str { .. } => Ok((var, ty)),
                    Type::ConstStr => {
                        let tmp = self.fresh_var();
                        let ty = Type::Str { len_type: Box::new(Type::i64()) };
                        let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                        body.push_str(&format!("{} {} = vix_string_from_const({});\n", c_type, tmp, var));
                        Ok((tmp, ty))
                    }
                    _ => {
                         
                         
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
                self.codegen_make_panic(&args[0], body)
            }
            "as_bytes" => {
                if args.len() != 1 { return Err(()); }
                self.ensure_type_defined(&Type::u8());
                let (obj_var, obj_ty) = self.codegen_expr(&args[0], body)?;
                let tmp = self.fresh_var();
                let slice_ty = Type::Array { element: Box::new(Type::u8()), size: None };
                let slice_name = slice_ty.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("{} {} = {{ .ptr = (uint8_t*){}.ptr, .len = {}.len }};\n", 
                    slice_name, tmp, obj_var, obj_var));
                 
                Ok((tmp, slice_ty))
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
                Ok((tmp, Type::Ptr(Box::new(Type::Const(Box::new(inner_type))))))
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
                Ok((tmp, Type::MutRef(Box::new(inner_type))))
            }

            "to_string" => {
                if args.len() != 1 { return Err(()); }
                let (obj_var, obj_ty) = self.codegen_expr(&args[0], body).check_error();
                
                self.ensure_string_typedef();
                self.ensure_to_string_helpers();
                
                let ty = Type::Str { len_type: Box::new(Type::i64()) };
                let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                let tmp = self.fresh_var();
                
                match &obj_ty {
                    Type::Int { .. } => {
                        body.push_str(&format!("{} {} = vix_int_to_str((int64_t){});\n", c_type, tmp, obj_var));
                    }
                    Type::Float { .. } => {
                        body.push_str(&format!("{} {} = vix_float_to_str((double){});\n", c_type, tmp, obj_var));
                    }
                    Type::Bool => {
                        body.push_str(&format!("{} {} = vix_bool_to_str({});\n", c_type, tmp, obj_var));
                    }
                    Type::Str { .. } | Type::ConstStr => {
                        return Ok((obj_var, ty));
                    }
                    _ => {
                         body.push_str(&format!("{} {} = {{ .ptr = \"<object>\", .len = 8 }};\n", c_type, tmp));
                    }
                }
                
                Ok((tmp, ty))
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
            let c_type = field_ty.to_c_type(&self.arch, &mut self.type_registry);
            body.push_str(&format!("{} {} = {}.{};\n", c_type, tmp, obj_var, field));
            return Ok((tmp, field_ty));
        }

        if let Some((_params, ret_ty, is_instance)) = self.impl_methods.get(&(obj_ty.name(), field.to_string())).cloned() {
            let tmp = self.fresh_var();
            let c_type = ret_ty.to_c_type(&self.arch, &mut self.type_registry);
            let method_name = format!("{}_{}", obj_ty.name(), field);
            let arg = if is_instance { format!("&{}", obj_var) } else { obj_var };
            body.push_str(&format!("{} {} = {}({});\n", c_type, tmp, method_name, arg));
            return Ok((tmp, ret_ty));
        } else if let Some((_params, ret_ty)) = self.user_functions.get(field).cloned() {
            let tmp = self.fresh_var();
            let c_type = ret_ty.to_c_type(&self.arch, &mut self.type_registry);
            body.push_str(&format!("{} {} = {}({});\n", c_type, tmp, field, obj_var));
            return Ok((tmp, ret_ty));
        } else if let Some(ext_info) = self.extern_functions.get(field).cloned() {
            let tmp = self.fresh_var();
            let c_type = ext_info.return_type.to_c_type(&self.arch, &mut self.type_registry);
            body.push_str(&format!("{} {} = {}({});\n", c_type, tmp, field, obj_var));
            return Ok((tmp, ext_info.return_type));
        }

        let struct_name = match &obj_ty {
            Type::Struct { name } => name.clone(),
            Type::Ref(inner) | Type::MutRef(inner) | Type::Ptr(inner) => {
                if let Type::Struct { name } = &**inner {
                    name.clone()
                } else {
                    return Err(())
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
        let (arr_var, arr_ty) = self.codegen_expr(arr, body).check_error();
        let (val_var, _val_ty) = self.codegen_expr(value, body).check_error();
        
         
        let is_slice = matches!(arr_ty, Type::Array { size: None, .. } | Type::Str { .. });
        
        let mut index_str = if is_slice {
            format!("{}.ptr", arr_var)
        } else {
            arr_var.clone()
        };
        
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
        match func {
             
            "print" | "println" | "plan" | "chars" | "str" | "string" | 
            "have" | "contain" | "contains" | "has" | "is_not_empty" | 
            "collect" | "contain_all" | "index" | "index_of" | "array" | 
            "some" | "none" | "ok" | "err" | "unwrap" | "unwrap_or" | 
            "is_some" | "is_none" | "array_get" | "wait" | "tuple" | 
            "is_empty" | "filter" | "panic" | "as_bytes" | "as_ptr" | 
            "as_mut_ptr" | "to_string" => {
                self.codegen_call_expr(func, args, body, loc)?;
                return Ok(());
            }
            _ => {}
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
            
            if let Some(params) = &param_types
                && let Some(param_ty) = params.get(i)
                    && (matches!(param_ty, Type::ConstStr) || matches!(param_ty, Type::Ptr(_) | Type::RawPtr(_) | Type::Ref(_) | Type::MutRef(_))) && matches!(ty, Type::Str { .. }) {
                        var = format!("{}.ptr", var);
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

            body.push_str(&format!("{} {} = {}({});\n", 
                type_name, tmp, constructor_name, args_str));
            
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

        if op == "+="
            && let Type::Struct { name: struct_name } = struct_ty
                && let Some(struct_info) = self.structs.get(struct_name)
                    && let Some((_, field_ty, _)) = struct_info.fields.iter()
                        .find(|(fname, _, _)| fname == field)
                        && matches!(field_ty, Type::StdStr) {
                            match &val_ty {
                                Type::StdStr => {
                                    body.push_str(&format!("String_extend(&{}{}{}, &{});\n", obj_var, access_op, field, val_var));
                                }
                                Type::Str { .. } | Type::ConstStr => {
                                    body.push_str(&format!("String_push_str(&{}{}{}, {});\n", obj_var, access_op, field, val_var));
                                }
                                Type::Char { .. } => {
                                    body.push_str(&format!("String_push(&{}{}{}, {});\n", obj_var, access_op, field, val_var));
                                }
                                _ => {
                                    self.ensure_to_string_helpers();
                                    let tmp = self.fresh_var();
                                    body.push_str(&format!("String {} = to_string_generic({});\n", tmp, val_var));
                                    body.push_str(&format!("String_push_str(&{}{}{}, {});\n", 
                                        obj_var, access_op, field, tmp));
                                }
                            }
                            return Ok(());
                        }

        body.push_str(&format!("{}{}{} {} {};\n", obj_var, access_op, field, op, val_var));
        Ok(())
    }
}
