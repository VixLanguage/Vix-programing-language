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
        let (arr_var, arr_ty) = self.codegen_expr(arr, body) .check_error();
        
        let elem_ty = match arr_ty {
            Type::Array { element, .. } => *element,
            Type::Ptr(inner) => *inner,
            _ => {
                return Err(());
            }
        };
        
        if matches!(elem_ty, Type::Void) {
            return Err(());
        }
        
        let c_type = elem_ty.to_c_type(&self.arch);
        
        let mut index_str = arr_var.clone();
        for idx in indices {
            let (idx_var, _) = self.codegen_expr(idx, body) .check_error();
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
            let (var, ty) = self.codegen_expr(elem, body) .check_error();
            if matches!(ty, Type::Void) {
                return Err(());
            }
            if elem_type.is_none() {
                elem_type = Some(ty);
            }
            elem_vars.push(var);
        }
        
        let elem_type = elem_type.unwrap();
        let c_type = elem_type.to_c_type(&self.arch);
        
        let tmp = self.fresh_var();
        let elems_str = elem_vars.join(", ");
        
        body.push_str(&format!("{} {}[] = {{{}}};\n", c_type, tmp, elems_str));
        
        Ok((tmp, Type::Array { 
            element: Box::new(elem_type), 
            size: Some(elements.len()) 
        }))
    }

    pub fn codegen_some(&mut self, inner: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (val_var, val_ty) = self.codegen_expr(inner, body) .check_error();
        let tmp = self.fresh_var();
        
 
        let opt_type = Type::Option { inner: Box::new(val_ty.clone()) };
        if let Some(def) = self.type_registry.generate_type_definition(&opt_type, &self.arch) {
            self.ir.add_function(def); 
        }
        
        let c_type = opt_type.to_c_type(&self.arch);
        body.push_str(&format!("{} {} = {{ .tag = 1, .value = {} }};\n", c_type, tmp, val_var));
        
        Ok((tmp, opt_type))
    }
    
    pub fn codegen_none(&mut self, expected_type: Option<&Type>, body: &mut String) -> Result<(String, Type), ()> {
        let tmp = self.fresh_var();
        
 
        let inner_type = expected_type
            .and_then(|t| match t {
                Type::Option { inner } => Some(inner.as_ref().clone()),
                _ => None,
            })
            .unwrap_or(Type::Void);
        
        let opt_type = Type::Option { inner: Box::new(inner_type.clone()) };
        if let Some(def) = self.type_registry.generate_type_definition(&opt_type, &self.arch) {
            self.ir.add_function(def); 
        }
        
        let c_type = opt_type.to_c_type(&self.arch);
        body.push_str(&format!("{} {} = {{ .tag = 0 }};\n", c_type, tmp));
        
        Ok((tmp, opt_type))
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
        
         
        if let Some(def) = self.type_registry.generate_type_definition(&result_type, &self.arch) {
            self.ir.add_function(def);
        }
        
        let c_type = result_type.to_c_type(&self.arch);
        
         
        if let Type::Result { ok, .. } = &result_type {
            if !self.types_compatible(ok, &val_ty) {
                self.diagnostics.error(
                    "TypeMismatch",
                    &format!("Cannot return {} in Result with OK type {}", val_ty.name(), ok.name()),
                    type_mismatch_error(&ok.name(), &val_ty.name(), value.location(), value.location())
                );
            }
        }

         
        body.push_str(&format!("{} {} = {{ .tag = 0, .data.ok = {} }};\n", c_type, tmp, val_var));
        
        Ok((tmp, result_type))
    }


    
    pub fn codegen_result_err(&mut self, value: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (val_var, val_ty) = self.codegen_expr(value, body) .check_error();
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
            self.ir.add_function(def); 
        }
        
        let c_type = result_type.to_c_type(&self.arch);
        
         
        if let Type::Result { err, .. } = &result_type {
            if !self.types_compatible(err, &val_ty) {
                self.diagnostics.error(
                    "TypeMismatch",
                    &format!("Cannot return {} in Result with ERR type {}", val_ty.name(), err.name()),
                    type_mismatch_error(&err.name(), &val_ty.name(), value.location(), value.location())
                );
            }
        }

        body.push_str(&format!("{} {} = {{ .tag = 1, .data.err = {} }};\n", c_type, tmp, val_var));
        Ok((tmp, result_type))
    }
    
    pub fn codegen_unwrap(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (var, ty) = self.codegen_expr(expr, body) .check_error();
        let tmp = self.fresh_var();
        
        match &ty {
            Type::Option { inner } => {
                let c_type = inner.to_c_type(&self.arch);
 
                body.push_str(&format!("if ({}.tag == 0) {{\n", var));
                body.push_str("    fprintf(stderr, \"unwrap called on None\\n\");\n");
                body.push_str("    exit(1);\n");
                body.push_str("}\n");
                body.push_str(&format!("{} {} = {}.value;\n", c_type, tmp, var));
                Ok((tmp, *inner.clone()))
            }
            Type::Result { ok, .. } => {
                let c_type = ok.to_c_type(&self.arch);
 
                body.push_str(&format!("if ({}.tag != 0) {{\n", var));
                body.push_str("    fprintf(stderr, \"unwrap called on Err\\n\");\n");
                body.push_str("    exit(1);\n");
                body.push_str("}\n");
                body.push_str(&format!("{} {} = {}.data.ok;\n", c_type, tmp, var));
                Ok((tmp, *ok.clone()))
            }
            Type::Ptr(inner) => {
                let c_type = inner.to_c_type(&self.arch);
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
                let c_type = inner.to_c_type(&self.arch);
                body.push_str(&format!("{} {} = ({}.tag == 1) ? {}.value : {};\n", 
                    c_type, tmp, var, var, default_var));
                Ok((tmp, *inner.clone()))
            }
            Type::Result { ok, .. } => {
                let c_type = ok.to_c_type(&self.arch);
                body.push_str(&format!("{} {} = ({}.tag == 0) ? {}.data.ok : {};\n", 
                    c_type, tmp, var, var, default_var));
                Ok((tmp, *ok.clone()))
            }
            Type::Ptr(_) => {
                body.push_str(&format!("auto {} = ({} != NULL) ? *{} : {};\n", 
                    tmp, var, var, default_var));
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
        body.push_str(&format!("{}* {} = &{};\n", ty.to_c_type(&self.arch), tmp, var));
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
        
         
        if let Some(tuple_def) = self.type_registry.generate_tuple_definition(&element_types, &self.config.arch) {
            if !self.ir.forward_decls.contains(&tuple_def) {
                self.ir.forward_decls.push_str(&tuple_def);
                self.ir.forward_decls.push_str("\n");
            }
        }
        
        let tuple_type = Type::Tuple { fields: element_types.clone() };
        let c_type = tuple_type.to_c_type(&self.arch);
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
            Type::Struct { name } => name.clone(),
            Type::Ref(inner) | Type::MutRef(inner) => {
                if let Type::Struct { name } = inner.as_ref() {
                    name.clone()
                } else {
                    return Err(());
                }
            }
            _ => return Err(()),
        };

        let method_full_name = format!("{}_{}", struct_name, method);

        let (return_type, is_instance) = if let Some((_, ret_ty, is_inst)) = self.impl_methods.get(&(struct_name.clone(), method.to_string())) {
            (ret_ty.clone(), *is_inst)
        } else {
             
            let prefixed_name = format!("{}_{}", struct_name, method);
            if let Some((_, ret_ty)) = self.user_functions.get(&prefixed_name) {
                (ret_ty.clone(), false)
            } else {
                (Type::Void, false)
            }
        };

        let mut arg_vars = Vec::new();
        if is_instance {
            if matches!(obj_ty, Type::Ref(_) | Type::MutRef(_)) || obj_ty.is_ptr() {
                arg_vars.push(obj_var);
            } else {
                arg_vars.push(format!("&{}", obj_var));
            }
        }

        for arg in args {
            let (var, _ty) = self.codegen_expr(arg, body).check_error();
            arg_vars.push(var);
        }

        let tmp = self.fresh_var();
        let args_str = arg_vars.join(", ");
        let c_type = return_type.to_c_type(&self.arch);
        
        if matches!(return_type, Type::Void) {
            body.push_str(&format!("{}({});\n", method_full_name, args_str));
            Ok(("".to_string(), Type::Void))
        } else {
            body.push_str(&format!("{} {} = {}({});\n", c_type, tmp, method_full_name, args_str));
            Ok((tmp, return_type))
        }
    }



    pub fn codegen_module_call(&mut self, module: &str, func: &str, args: &[Expr], body: &mut String, _loc: SourceLocation) -> Result<(String, Type), ()> {
        let mut arg_vars = Vec::new();
        for arg in args {
            let (var, _) = self.codegen_expr(arg, body) .check_error();
            arg_vars.push(var);
        }
        
        let tmp = self.fresh_var();
        let args_str = arg_vars.join(", ");
        
 
        let method_key = (module.to_string(), func.to_string());
        if let Some((_, ret_ty, _)) = self.module_functions.get(&method_key).cloned() {
            let c_type = ret_ty.to_c_type(&self.arch);
            body.push_str(&format!("{} {} = {}_{}({});\n", c_type, tmp, module, func, args_str));
            return Ok((tmp, ret_ty));
        }
        
 
        body.push_str(&format!("int32_t {} = {}_{}({});\n", tmp, module, func, args_str));
        Ok((tmp, Type::i32()))
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

    pub fn codegen_panic(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (msg_var, _) = self.codegen_expr(expr, body) .check_error();
        let tmp = self.fresh_var();
        
        body.push_str(&format!("fprintf(stderr, \"panic: %s\\n\", {});\n", msg_var));
        body.push_str("exit(1);\n");
        body.push_str(&format!("int {} = 0;\n", tmp));
        
        Ok((tmp, Type::Void))
    }
}