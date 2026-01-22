use crate::import::*;

impl Codegen {
    pub fn codegen_chars(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (str_var, _) = self.codegen_expr(expr, body) .check_error();
        let tmp = self.fresh_var();
        let _func_name = format!("{}_chars", tmp); 
        let func_def = format!(
            "char* {}_chars(const char* str) {{
    size_t len = strlen(str);
    char* result = malloc((len + 1) * sizeof(char));
    if (result != NULL) {{
        strcpy(result, str);
    }}
    return result;
}}\n", tmp
        );
        
        self.ir.add_function(func_def);
        body.push_str(&format!("char* {} = {}_chars({});\n", tmp, tmp, str_var));
        
        Ok((tmp, Type::Array { element: Box::new(Type::Char { bits: 8, signed: true }), size: None }))
    }

    pub fn codegen_have(&mut self, obj: &Expr, item: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (obj_var, obj_ty) = self.codegen_expr(obj, body) .check_error();
        let (item_var, _) = self.codegen_expr(item, body) .check_error();
        let tmp = self.fresh_var();
        
        match &obj_ty {
            Type::Array { element: _, size: Some(size) } => {
                body.push_str(&format!("bool {} = false;\n", tmp));
                body.push_str(&format!("for (int i = 0; i < {}; i++) {{\n", size));
                body.push_str(&format!("    if ({}[i] == {}) {{\n", obj_var, item_var));
                body.push_str(&format!("        {} = true;\n", tmp));
                body.push_str("        break;\n");
                body.push_str("    }\n");
                body.push_str("}\n");
            }
            Type::Array { element: _, size: None } => {
                body.push_str(&format!("bool {} = false;\n", tmp));
                body.push_str(&format!("for (int i = 0; i < {}.len; i++) {{\n", obj_var));
                body.push_str(&format!("    if ({}.ptr[i] == {}) {{\n", obj_var, item_var));
                body.push_str(&format!("        {} = true;\n", tmp));
                body.push_str("        break;\n");
                body.push_str("    }\n");
                body.push_str("}\n");
            }
            Type::Ptr(_) => {
                body.push_str(&format!("bool {} = ({} == {});\n", tmp, obj_var, item_var));
            }
            _ => {
                body.push_str(&format!("bool {} = ({} == {});\n", tmp, obj_var, item_var));
            }
        }
        
        Ok((tmp, Type::Bool))
    }

    pub fn codegen_is_not_empty(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (var, ty) = self.codegen_expr(expr, body) .check_error();
        let tmp = self.fresh_var();
        
        match &ty {
            Type::Array { size: Some(size), .. } => {
                body.push_str(&format!("bool {} = ({} > 0);\n", tmp, size));
            }
            Type::Array { size: None, .. } => {
                body.push_str(&format!("bool {} = ({}.len > 0);\n", tmp, var));
            }
            Type::Ptr(_) => {
                body.push_str(&format!("bool {} = ({} != NULL);\n", tmp, var));
            }
            Type::Str {..} => {
                body.push_str(&format!("bool {} = (strlen({}) > 0);\n", tmp, var));
            }
            _ => {
                body.push_str(&format!("bool {} = ({} != 0);\n", tmp, var));
            }
        }
        
        Ok((tmp, Type::Bool))
    }

    pub fn codegen_collect(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (var, ty) = self.codegen_expr(expr, body) .check_error();
        let tmp = self.fresh_var();

        match &ty {
            Type::Array { .. } => {
                body.push_str(&format!("void* {} = {};\n", tmp, var));
            }
            _ => {
                body.push_str(&format!("void* {} = {};\n", tmp, var));
            }
        }
        Ok((tmp, Type::Ptr(Box::new(Type::Void))))
    }

    pub fn codegen_contain_all(&mut self, obj: &Expr, items: &[Expr], body: &mut String) -> Result<(String, Type), ()> {
        let (obj_var, obj_ty) = self.codegen_expr(obj, body) .check_error();
        let tmp = self.fresh_var();
    
        body.push_str(&format!("bool {} = true;\n", tmp));
        
        for (i, item) in items.iter().enumerate() {
            let (item_var, _) = self.codegen_expr(item, body) .check_error();
            let temp_check = format!("temp_check_{}", i);
            
            match &obj_ty {
                Type::Array { element: _, size: Some(size) } => {
                    body.push_str(&format!("bool {} = false;\n", temp_check));
                    body.push_str(&format!("for (int i = 0; i < {}; i++) {{\n", size));
                    body.push_str(&format!("    if ({}[i] == {}) {{\n", obj_var, item_var));
                    body.push_str(&format!("        {} = true;\n", temp_check));
                    body.push_str("        break;\n");
                    body.push_str("    }\n");
                    body.push_str("}\n");
                }
                Type::Array { element: _, size: None } => {
                    body.push_str(&format!("bool {} = false;\n", temp_check));
                    body.push_str(&format!("for (int i = 0; i < {}.len; i++) {{\n", obj_var));
                    body.push_str(&format!("    if ({}.ptr[i] == {}) {{\n", obj_var, item_var));
                    body.push_str(&format!("        {} = true;\n", temp_check));
                    body.push_str("        break;\n");
                    body.push_str("    }\n");
                    body.push_str("}\n");
                }
                _ => {
                    body.push_str(&format!("bool {} = ({} == {});\n", temp_check, obj_var, item_var));
                }
            }
            
            body.push_str(&format!("{} = {} && {};\n", tmp, tmp, temp_check));
        }
        
        Ok((tmp, Type::Bool))
    }

    pub fn codegen_contain(&mut self, obj: &Expr, item: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        self.codegen_have(obj, item, body)
    }

    pub fn codegen_index(&mut self, arr: &Expr, indices: &[Expr], body: &mut String) -> Result<(String, Type), ()> {
        let (arr_var, arr_ty) = self.codegen_expr(arr, body).check_error();
        let (elem_ty, is_slice, is_string) = match arr_ty {
            Type::Array { element, size } => (*element, size.is_none(), false),
            Type::Ptr(inner) => (*inner, false, false),
            Type::Str { .. } => (Type::char8(), true, true),
            _ => {
                return Err(());
            }
        };
        
        let c_type = elem_ty.to_c_type(&self.arch, &mut self.type_registry);
        
        if matches!(elem_ty, Type::Void) {
            return Err(());
        }
        
        let mut index_str = if is_slice || is_string {
            format!("{}.ptr", arr_var)
        } else {
            arr_var.clone()
        };
        
        for idx in indices {
            let (idx_var, _) = self.codegen_expr(idx, body).check_error();
            index_str = format!("{}[{}]", index_str, idx_var);
        }
        
        let tmp = self.fresh_var();
        body.push_str(&format!("{} {} = {};\n", c_type, tmp, index_str));
        Ok((tmp, elem_ty))
    }
    pub fn codegen_index_of(&mut self, obj: &Expr, item: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (obj_var, obj_ty) = self.codegen_expr(obj, body) .check_error();
        let (item_var, _) = self.codegen_expr(item, body) .check_error();
        let tmp = self.fresh_var();
        
        match &obj_ty {
            Type::Array { element: _, size: Some(size) } => {
                body.push_str(&format!("int {} = -1;\n", tmp));
                body.push_str(&format!("for (int i = 0; i < {}; i++) {{\n", size));
                body.push_str(&format!("    if ({}[i] == {}) {{\n", obj_var, item_var));
                body.push_str(&format!("        {} = i;\n", tmp));
                body.push_str("        break;\n");
                body.push_str("    }\n");
                body.push_str("}\n");
            }
            Type::Array { element: _, size: None } => {
                body.push_str(&format!("int {} = -1;\n", tmp));
                body.push_str(&format!("for (int i = 0; i < {}.len; i++) {{\n", obj_var));
                body.push_str(&format!("    if ({}.ptr[i] == {}) {{\n", obj_var, item_var));
                body.push_str(&format!("        {} = i;\n", tmp));
                body.push_str("        break;\n");
                body.push_str("    }\n");
                body.push_str("}\n");
            }
            _ => {
                body.push_str(&format!("int {} = ({} == {}) ? 0 : -1;\n", tmp, obj_var, item_var));
            }
        }
        
        Ok((tmp, Type::Int { bits: 32, signed: true }))
    }

    pub fn codegen_reference_to(&mut self, ty: &Type, body: &mut String) -> Result<(String, Type), ()> {
        let tmp: String = self.fresh_var();
        let type_name = ty.name();
        let type_id = type_name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
            
        body.push_str(&format!("uint32_t {} = {};\n", tmp, type_id));
        Ok((tmp, Type::u32()))
    }

    pub fn codegen_array(&mut self, elements: &[Expr], body: &mut String) -> Result<(String, Type), ()> {
        if elements.is_empty() {
            return Err(());
        }
        
        let mut elem_vars = Vec::new();
        let mut elem_type = None;
        
        for elem in elements {
            let (var, ty) = self.codegen_expr(elem, body).check_error();
            if matches!(ty, Type::Void) {
                return Err(());
            }
            if elem_type.is_none() {
                elem_type = Some(ty);
            }
            elem_vars.push(var);
        }
        
        let elem_type = elem_type.unwrap();
        
         
        if matches!(elem_type, Type::ConstStr) {
            let tmp = self.fresh_var();
            let elems_str = elem_vars.join(", ");
            
             
            body.push_str(&format!("const char* {}[] = {{{}}};\n", tmp, elems_str));
            
             
            let slice_tmp = self.fresh_var();
            body.push_str(&format!("struct {{ const char** ptr; size_t len; }} {} = {{ .ptr = {}, .len = {} }};\n", 
                slice_tmp, tmp, elements.len()));
            
            return Ok((slice_tmp, Type::Array { 
                element: Box::new(Type::ConstStr), 
                size: None 
            }));
        }
        
        let c_type = elem_type.to_c_type(&self.arch, &mut self.type_registry);
        let tmp = self.fresh_var();
        let elems_str = elem_vars.join(", ");
        
        body.push_str(&format!("{} {}[] = {{{}}};\n", c_type, tmp, elems_str));
        
        let slice_tmp = self.fresh_var();
        let slice_type_name = self.type_registry.generate_slices(&elem_type, &self.arch);
        
        if let Some(typedef) = self.type_registry.get_slice_typedef(&elem_type)
            && !self.ir.forward_decls.contains(&typedef) {
                self.ir.forward_decls.push_str(&typedef);
                self.ir.forward_decls.push('\n');
            }
        
        body.push_str(&format!("{} {} = {{ .ptr = {}, .len = {} }};\n", 
            slice_type_name, slice_tmp, tmp, elements.len()));
        
        Ok((slice_tmp, Type::Array { 
            element: Box::new(elem_type), 
            size: None 
        }))
    }


    pub fn codegen_unwrap(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (var, ty) = self.codegen_expr(expr, body) .check_error();
        let tmp = self.fresh_var();
        
        match &ty {
            Type::Option { inner } => {
                let c_type = inner.to_c_type(&self.arch, &mut self.type_registry);
 
                body.push_str(&format!("if ({}.tag == 0) {{\n", var));
                body.push_str("    fprintf(stderr, \"unwrap called on None\\n\");\n");
                body.push_str("    exit(1);\n");
                body.push_str("}\n");
                body.push_str(&format!("{} {} = {}.value;\n", c_type, tmp, var));
                Ok((tmp, *inner.clone()))
            }
            Type::Result { ok, .. } => {
                let c_type = ok.to_c_type(&self.arch, &mut self.type_registry);
 
                body.push_str(&format!("if ({}.tag != 0) {{\n", var));
                body.push_str("    fprintf(stderr, \"unwrap called on Err\\n\");\n");
                body.push_str("    exit(1);\n");
                body.push_str("}\n");
                body.push_str(&format!("{} {} = {}.data.ok;\n", c_type, tmp, var));
                Ok((tmp, *ok.clone()))
            }
            Type::Ptr(inner) => {
                let c_type = inner.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("if ({} == NULL) {{\n", var));
                body.push_str("    fprintf(stderr, \"unwrap called on null pointer\\n\");\n");
                body.push_str("    exit(1);\n");
                body.push_str("}\n");
                body.push_str(&format!("{} {} = *{};\n", c_type, tmp, var));
                Ok((tmp, *inner.clone()))
            }
            _ => {
 
                Ok((var, ty))
            }
        }
    }
    
    pub fn codegen_unwrap_or(&mut self, expr: &Expr, default: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (var, ty) = self.codegen_expr(expr, body) .check_error();
        let (default_var, default_ty) = self.codegen_expr(default, body) .check_error();
        let tmp = self.fresh_var();
        
        match &ty {
            Type::Option { inner } => {
                let c_type = inner.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("{} {} = ({}.tag == 1) ? {}.value : {};\n", c_type, tmp, var, var, default_var));
                Ok((tmp, *inner.clone()))
            }
            Type::Result { ok, .. } => {
                let c_type = ok.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("{} {} = ({}.tag == 0) ? {}.data.ok : {};\n", c_type, tmp, var, var, default_var));
                Ok((tmp, *ok.clone()))
            }
            Type::Ptr(_) => {
                body.push_str(&format!("auto {} = ({} != NULL) ? *{} : {};\n", tmp, var, var, default_var));
                Ok((tmp, default_ty))
            }
            _ => {
 
                Ok((var, ty))
            }
        }
    }
    
    pub fn codegen_option_method(&mut self, obj: &Expr, method: &str, body: &mut String) -> Result<(String, Type), ()> {
        let (obj_var, obj_ty) = self.codegen_expr(obj, body) .check_error();
        let tmp = self.fresh_var();
        
        match method {
            "is_some" => {
                match &obj_ty {
                    Type::Option { .. } => {
                        body.push_str(&format!("bool {} = ({}.tag == 1);\n", tmp, obj_var));
                    }
                    Type::Ptr(_) => {
                        body.push_str(&format!("bool {} = ({} != NULL);\n", tmp, obj_var));
                    }
                    _ => {
                        body.push_str(&format!("bool {} = true;\n", tmp));
                    }
                }
                Ok((tmp, Type::Bool))
            }
            "is_none" => {
                match &obj_ty {
                    Type::Option { .. } => {
                        body.push_str(&format!("bool {} = ({}.tag == 0);\n", tmp, obj_var));
                    }
                    Type::Ptr(_) => {
                        body.push_str(&format!("bool {} = ({} == NULL);\n", tmp, obj_var));
                    }
                    _ => {
                        body.push_str(&format!("bool {} = false;\n", tmp));
                    }
                }
                Ok((tmp, Type::Bool))
            }
            _ => {
 
                body.push_str(&format!("int32_t {} = 0;\n", tmp));
                Ok((tmp, Type::i32()))
            }
        }
    }

    pub fn codegen_array_get(&mut self, obj: &Expr, reference: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (obj_var, _obj_ty) = self.codegen_expr(obj, body).check_error();
        let (ref_var, ref_ty) = self.codegen_expr(reference, body).check_error();
        let tmp = self.fresh_var();
        let _ref_var = ref_var; 

        body.push_str(&format!("void* {} = array_get_by_type({}, {});\n", tmp, obj_var, _ref_var));
        Ok((tmp, Type::Option { inner: Box::new(ref_ty) }))
    }

    pub fn codegen_wait(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (var, ty) = self.codegen_expr(expr, body) .check_error();
        let tmp = self.fresh_var();
        body.push_str(&format!("{}* {} = &{};\n", ty.to_c_type(&self.arch, &mut self.type_registry), tmp, var));
        Ok((tmp, Type::Ptr(Box::new(ty))))
    }

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
        loc: SourceLocation,
    ) -> Result<(String, Type), ()> {
        let (obj_var, obj_ty) = self.codegen_expr(obj, body).check_error();
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
                match method {
                 "to_string" => {
                    self.ensure_string_typedef();
                    self.ensure_to_string_helpers();
                    let ty = Type::Str { len_type: Box::new(Type::i64()) };
                    let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                    let tmp = self.fresh_var();
                    match &obj_ty {
                        Type::Int { bits: 64, .. } => {
                            body.push_str(&format!("{} {} = vix_int_to_str((int64_t){});\n", c_type, tmp, obj_var));
                        }
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
                    return Ok((tmp, ty));
                 }
                 "as_ptr" => {
                    let inner_type = match &obj_ty {
                        Type::Array { element, .. } => *element.clone(),
                        Type::Str { .. } => Type::char8(),
                        _ => Type::Void,
                    };
                    let tmp = self.fresh_var();
                    let c_inner = inner_type.to_c_type(&self.arch, &mut self.type_registry);
                    body.push_str(&format!("const {}* {} = {}.ptr;\n", c_inner, tmp, obj_var));
                    return Ok((tmp, Type::Ptr(Box::new(Type::Const(Box::new(inner_type))))));
                 }
                 "as_mut_ptr" => {
                    let inner_type = match &obj_ty {
                        Type::Array { element, .. } => *element.clone(),
                        Type::Str { .. } => Type::char8(),
                        _ => Type::Void,
                    };
                    let tmp = self.fresh_var();
                    let c_inner = inner_type.to_c_type(&self.arch, &mut self.type_registry);
                    body.push_str(&format!("{}* {} = {}.ptr;\n", c_inner, tmp, obj_var));
                    return Ok((tmp, Type::MutRef(Box::new(inner_type))));
                 }
                 "as_bytes" => {
                    let tmp = self.fresh_var();
                    body.push_str(&format!("struct {{ uint8_t* ptr; size_t len; }} {} = {{ .ptr = (uint8_t*){}.ptr, .len = {}.len }};\n", 
                        tmp, obj_var, obj_var));
                    return Ok((tmp, Type::Array { 
                        element: Box::new(Type::Int { bits: 8, signed: false }), 
                        size: None 
                    }));
                 }
                 "repeat" => {
                    self.ensure_string_typedef();
                    self.ensure_repeat_helper();
                    (Type::Str { len_type: Box::new(Type::i64()) }, true, "vix_repeat".to_string())
                 }
                 _ => {
                      if let Some(sn) = &struct_name {
                         let prefixed_name = format!("{}_{}", sn, method);
                         if let Some((_, ret_ty)) = self.user_functions.get(&prefixed_name) {
                             (ret_ty.clone(), false, prefixed_name)
                         } else {
                             (Type::i32(), true, method.to_string())  
                         }
                      } else {
                         (Type::i32(), true, method.to_string())  
                      }
                 }
                }
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
            let (var, _ty) = self.codegen_expr(arg, body).check_error();
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


    pub fn codegen_is_empty(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (var, ty) = self.codegen_expr(expr, body) .check_error();
        let tmp = self.fresh_var();
        
        match &ty {
            Type::Array { size: Some(size), .. } => {
                body.push_str(&format!("bool {} = ({} == 0);\n", tmp, size));
            }
            Type::Array { size: None, .. } => {
                body.push_str(&format!("bool {} = ({}.len == 0);\n", tmp, var));
            }
            Type::Ptr(_) => {
                body.push_str(&format!("bool {} = ({} == NULL);\n", tmp, var));
            }
            Type::Str { .. } => {
                body.push_str(&format!("bool {} = (strlen({}) == 0);\n", tmp, var));
            }
            _ => {
                body.push_str(&format!("bool {} = ({} == 0);\n", tmp, var));
            }
        }
        
        Ok((tmp, Type::Bool))
    }

    pub fn codegen_filter(&mut self, obj: &Expr, reference: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (obj_var, _obj_ty) = self.codegen_expr(obj, body) .check_error();
        let (_ref_var, _) = self.codegen_expr(reference, body) .check_error();
        let tmp = self.fresh_var();
    
        body.push_str(&format!("void* {} = {};\n", tmp, obj_var));
        Ok((tmp, Type::Ptr(Box::new(Type::Void))))
    }

    pub fn codegen_make_panic(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (msg_var, _) = self.codegen_expr(expr, body) .check_error();
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

    pub fn codegen_plan(&mut self, format_str: &str, args: &[Expr], body: &mut String) -> Result<(String, Type), ()> {
        let mut arg_vars = Vec::new();
        let mut arg_types = Vec::new();
        
        for arg in args {
            let (var, ty) = self.codegen_expr(arg, body)?;
            arg_vars.push(var);
            arg_types.push(ty);
        }
        
         
        let placeholder_count = format_str.matches("{}").count();
        
        if placeholder_count != args.len() {
            self.diagnostics.error(
                "PlanArgumentMismatch",
                &format!("plan() expects {} arguments but got {}", placeholder_count, args.len()),
                ErrorContext {
                    primary_location: self.default_location(),
                    secondary_locations: vec![],
                    help_message: Some("Number of '{}' placeholders must match number of arguments.".to_string()),
                    suggestions: vec![],
                }
            );
            return Err(());
        }
        
         
        let result_var = self.fresh_var();
        let parts: Vec<&str> = format_str.split("{}").collect();
        let temp_buf = self.fresh_var();

        body.push_str(&format!("char* {} = (char*)malloc(4096);\n", result_var));
        body.push_str(&format!("{}[0] = '\\0';\n", result_var));
        body.push_str(&format!("char {}[512];\n", temp_buf));
        
        for (i, part) in parts.iter().enumerate() {  
            if !part.is_empty() {
                body.push_str(&format!("strcat({}, \"{}\");\n", result_var, part.replace("\\", "\\\\").replace("\"", "\\\"")));
            }
            
             
            if i < arg_vars.len() {
                let arg_var = &arg_vars[i];
                let arg_type = &arg_types[i];
                
                match arg_type {
                    Type::Int { bits, signed } => {
                        let fmt = if *signed {
                            match bits {
                                64 => "%lld",
                                _ => "%d",
                            }
                        } else {
                            match bits {
                                64 => "%llu",
                                _ => "%u",
                            }
                        };
                        body.push_str(&format!("sprintf({}, \"{}\", {});\n", temp_buf, fmt, arg_var));
                        body.push_str(&format!("strcat({}, {});\n", result_var, temp_buf));
                    }
                    Type::Float { bits } => {
                        let fmt = if *bits == 32 { "%f" } else { "%lf" };
                        body.push_str(&format!("sprintf({}, \"{}\", {});\n", temp_buf, fmt, arg_var));
                        body.push_str(&format!("strcat({}, {});\n", result_var, temp_buf));
                    }
                    Type::Bool => {
                        body.push_str(&format!("strcat({}, {} ? \"true\" : \"false\");\n", result_var, arg_var));
                    }
                    Type::Str { .. } => {
                        body.push_str(&format!("strcat({}, {}.ptr);\n", result_var, arg_var));
                    }
                    Type::ConstStr => {
                        body.push_str(&format!("strcat({}, {});\n", result_var, arg_var));
                    }
                    Type::Char { .. } => {
                        body.push_str(&format!("sprintf({}, \"%c\", {});\n", temp_buf, arg_var));
                        body.push_str(&format!("strcat({}, {});\n", result_var, temp_buf));
                    }
                    _ => {
                        body.push_str(&format!("sprintf({}, \"%p\", {});\n", temp_buf, arg_var));
                        body.push_str(&format!("strcat({}, {});\n", result_var, temp_buf));
                    }
                }
            }
        }
        
         
        let string_var = self.fresh_var();
        body.push_str(&format!("String {} = {{ .ptr = {}, .len = strlen({}) }};\n", 
            string_var, result_var, result_var));
        
        Ok((string_var, Type::Str { len_type: Box::new(Type::i64()) }))
    }

    pub fn codegen_hashmap_get(&mut self, map: &Expr, key: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (map_var, map_ty) = self.codegen_expr(map, body)?;
        let (key_var, _) = self.codegen_expr(key, body)?;      
        let (actual_map_ty, is_ref) = match &map_ty {
            Type::HashMap { .. } => (map_ty.clone(), false),
            Type::Ref(inner) | Type::MutRef(inner) => (*inner.clone(), true),
            _ => return Err(()),
        };
        let (key_ty, val_ty) = match actual_map_ty {
            Type::HashMap { key, value } => (*key, *value),
            _ => return Err(()),
        };
        let hashmap_nome = format!("HashMap_{}_{}", key_ty.name().replace(" ", "_"), val_ty.name().replace(" ", "_"));
        let tmp = self.fresh_var();
        let val_c_type = val_ty.to_c_type(&self.arch, &mut self.type_registry);
        let map_arg = if is_ref {
             map_var
        } else {
             format!("&{}", map_var)
        };

        body.push_str(&format!("{}* {}_ptr = get_{}({}, {});\n", val_c_type, tmp, hashmap_nome, map_arg, key_var));
        body.push_str(&format!("{} {} = {}_ptr ? *{}_ptr : ({}){{0}};\n", val_c_type, tmp, tmp, tmp, val_c_type));
        
        Ok((tmp, val_ty))
    }

    pub fn codegen_result_ok(&mut self, value: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (val_var, val_ty) = self.codegen_expr(value, body).check_error();
        let tmp = self.fresh_var(); 
        
        let result_type = if let Some(rt @ Type::Result { .. }) = &self.current_return_type {
            rt.clone()
                } else {
            Type::Result { 
                ok: Box::new(val_ty.clone()), 
                err: Box::new(Type::Str { len_type: Box::new(Type::i64()) }) 
            }
        };

         if let Some(def) = self.type_registry.generate_type_definition(&result_type, &self.arch)
            && !self.ir.forward_decls.contains(&def) {
                self.ir.forward_decls.push_str(&def);
                self.ir.forward_decls.push('\n');
            }
        

        if let Some(def) = self.type_registry.generate_type_definition(&result_type, &self.arch) {
            self.ir.add_type_definition(def);
        }

        let c_type = result_type.to_c_type(&self.arch, &mut self.type_registry);

        if let Type::Result { ok, .. } = &result_type
            && !self.types_compatible(ok, &val_ty) {
                self.diagnostics.error(
                    "TypeMismatch",
                    &format!("Cannot return {} in Result with OK type {}", val_ty.name(), ok.name()),
                    type_mismatch_error(&ok.name(), &val_ty.name(), value.location(), value.location())
                );
            }

        body.push_str(&format!("{} {} = {{ .tag = 0, .data = {{ .ok = {} }} }};\n", 
            c_type, tmp, val_var));
        
        Ok((tmp, result_type))
    }

    pub fn codegen_result_err(&mut self, value: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (val_var, val_ty) = self.codegen_expr(value, body).check_error();
        let tmp = self.fresh_var();
        
        let result_type = if let Some(rt @ Type::Result { .. }) = &self.current_return_type {
            rt.clone()
        } else {
            Type::Result { 
                ok: Box::new(Type::Void), 
                err: Box::new(val_ty.clone()) 
            }
        };

        if let Some(def) = self.type_registry.generate_type_definition(&result_type, &self.arch) {
            self.ir.add_type_definition(def);
        }
        
        let c_type = result_type.to_c_type(&self.arch, &mut self.type_registry);
        
        if let Type::Result { err, .. } = &result_type
            && !self.types_compatible(err, &val_ty) {
                self.diagnostics.error(
                    "TypeMismatch",
                    &format!("Cannot return {} in Result with ERR type {}", val_ty.name(), err.name()),
                    type_mismatch_error(&err.name(), &val_ty.name(), value.location(), value.location())
                );
            }

        body.push_str(&format!("{} {} = {{ .tag = 1, .data = {{ .err = {} }} }};\n", 
            c_type, tmp, val_var));
        
        Ok((tmp, result_type))
    }

    pub fn codegen_some(&mut self, inner: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (val_var, val_ty) = self.codegen_expr(inner, body).check_error();
        let tmp = self.fresh_var();
        
        let opt_type = Type::Option { inner: Box::new(val_ty.clone()) };
        
        if let Some(def) = self.type_registry.generate_type_definition(&opt_type, &self.arch) {
            self.ir.add_type_definition(def);
        }
        
        let c_type = opt_type.to_c_type(&self.arch, &mut self.type_registry);
        
        body.push_str(&format!("{} {} = {{ .tag = 1, .value = {} }};\n", 
            c_type, tmp, val_var));
        
        Ok((tmp, opt_type))
    }

    pub fn codegen_none(&mut self, expected_type: Option<&Type>, body: &mut String) -> Result<(String, Type), ()> {
        let tmp = self.fresh_var();
        
        let inner_type = expected_type
            .and_then(|t| match t {
                Type::Option { inner } => Some(inner.as_ref().clone()),
                _ => None,
            }).unwrap_or(Type::Void);

        let opt_type = Type::Option { inner: Box::new(inner_type.clone()) };
        
        if let Some(def) = self.type_registry.generate_type_definition(&opt_type, &self.arch) {
            self.ir.add_type_definition(def);
        }
        
        let c_type = opt_type.to_c_type(&self.arch, &mut self.type_registry);
        
        body.push_str(&format!("{} {} = {{ .tag = 0 }};\n", c_type, tmp));
        
        Ok((tmp, opt_type))
    }
}

