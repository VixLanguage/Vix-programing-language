use crate::import::*;

impl Codegen {
    pub fn codegen_tuple(&mut self, elements: &[Expr], body: &mut String) -> Result<(String, Type), ()> {
         
        let mut element_types = Vec::new();
        let mut element_vars = Vec::new();
        
        for elem in elements {
            let (var, ty) = self.codegen_expr(elem, body)?;
            element_vars.push(var);
            element_types.push(ty);
        }

        if let Some(tuple_def) = self.type_registry.generate_tuple_definition(&element_types, &self.config.arch)
            && !self.ir.forward_decls.contains(&tuple_def) {
                self.ir.forward_decls.push_str(&tuple_def);
                self.ir.forward_decls.push('\n');
            }
        
        let tuple_type = Type::Tuple { fields: element_types.clone() };
        let c_type = tuple_type.to_c_type(&self.arch, &mut self.type_registry);
        let tmp = self.fresh_var();
        
         
        body.push_str(&format!("{} {} = {{", c_type, tmp));
        
        for (i, (var, ty)) in element_vars.iter().zip(&element_types).enumerate() {
            if i > 0 {
                body.push_str(", ");
            }
            
             
            match ty {
                Type::Str { .. } => {
                     
                    body.push_str(&format!(" .field_{} = {}", i, var));
                }
                _ => {
                     
                    body.push_str(&format!(" .field_{} = {}", i, var));
                }
            }
        }
        
        body.push_str(" };\n");
        
        Ok((tmp, tuple_type))
    }


    pub fn codegen_method_call(
        &mut self,
        obj: &Expr,
        method: &str,
        args: &[Expr],
        body: &mut String,
        _loc: SourceLocation,
    ) -> Result<(String, Type), ()> {
        let (obj_var, obj_ty) = self.codegen_expr(obj, body)?;
        let struct_name = match &obj_ty {
            Type::Struct { name } => Some(name.clone()),
            Type::Ref(inner) | Type::MutRef(inner) => {
                if let Type::Struct { name } = inner.as_ref() {
                    Some(name.clone())
                } else { 
                    None 
                }
            }
            _ => None,
        };

        let method_info = if let Some(sn) = &struct_name {
            if let Some((_, ret_ty, is_inst)) = self.impl_methods.get(&(sn.clone(), method.to_string())) {
                Some((ret_ty.clone(), *is_inst, format!("{}_{}", sn, method)))
            } else { None }
        } else { None };

        println!("[DEBUG] codegen_method_call: method={}", method);
        
        let (return_type, is_instance, method_full_name) = if let Some(mi) = method_info {
            mi
        } else if let Some((_, ret_ty)) = self.user_functions.get(method) {
            (ret_ty.clone(), true, method.to_string())
        } else if let Some(ext) = self.extern_functions.get(method) {
            (ext.return_type.clone(), true, method.to_string())
        } else {
             
            let mut found = None;
            if let Some(sn) = &struct_name {
                let prefixed = format!("{}_{}", sn, method);
                if let Some((_, ret_ty)) = self.user_functions.get(&prefixed) {
                    found = Some((ret_ty.clone(), true, prefixed));
                }
            }
            
             
            if let Some(f) = found {
                f
            } else {
                 
                (Type::i32(), true, method.to_string())
            }
        };

        let mut arg_vars = Vec::new();
        
        if is_instance {
            if matches!(obj_ty, Type::Ref(_) | Type::MutRef(_)) || obj_ty.is_ptr() {
                arg_vars.push(obj_var);
            } else {
                let is_struct_method = if let Some(sn) = &struct_name {
                    self.impl_methods.contains_key(&(sn.clone(), method.to_string()))
                } else { false };

                if is_struct_method {
                    arg_vars.push(format!("&{}", obj_var));
                } else {
                    arg_vars.push(obj_var);
                }
            }
        }

        for arg in args {
            let (var, _ty) = self.codegen_expr(arg, body)?;
            arg_vars.push(var);
        }

        let tmp = self.fresh_var();
        let args_str = arg_vars.join(", ");
        let c_type = return_type.to_c_type(&self.arch, &mut self.type_registry);
        
        if matches!(return_type, Type::Void) {
            body.push_str(&format!("{}({});\n", method_full_name, args_str));
            Ok(("".to_string(), Type::Void))
        } else {
            body.push_str(&format!("{} {} = {}({});\n", c_type, tmp, method_full_name, args_str));
            Ok((tmp, return_type))
        }
    }

    pub fn codegen_make_panic(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (msg_var, _) = self.codegen_expr(expr, body) ?;
        let tmp = self.fresh_var();
        
        body.push_str(&format!("fprintf(stderr, \"panic: %s\\n\", {});\n", msg_var));
        body.push_str("exit(1);\n");
        body.push_str(&format!("int {} = 0;\n", tmp));
        
        Ok((tmp, Type::Void))
    }
    
    pub fn codegen_string_compare(&mut self, left: &str, right: &str, op: &str, body: &mut String) -> String {
        let tmp = self.fresh_var();
        let cmp_var = self.fresh_var();
        
        body.push_str(&format!("int {} = strcmp({}, {});\n", cmp_var, left, right));
        
        let condition = match op {
            "==" => format!("{} == 0", cmp_var),
            "!=" => format!("{} != 0", cmp_var),
            "<" => format!("{} < 0", cmp_var),
            "<=" => format!("{} <= 0", cmp_var),
            ">" => format!("{} > 0", cmp_var),
            ">=" => format!("{} >= 0", cmp_var),
            _ => format!("{} == 0", cmp_var),
        };
        
        body.push_str(&format!("bool {} = {};\n", tmp, condition));
        tmp
    }
}

