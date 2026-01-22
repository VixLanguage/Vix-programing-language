use crate::import::*;

impl Codegen {
    pub fn codegen_binop(
        &mut self,
        op: &str, 
        left: &Expr,
        right: &Expr,
        body: &mut String,
        loc: SourceLocation,
    ) -> Result<(String, Type), ()> {
        let (l_var, l_ty) = self.codegen_expr(left, body) .check_error();
        let (r_var, r_ty) = self.codegen_expr(right, body) .check_error();

         
        let left_var = l_var.clone();
        let right_var = r_var.clone();
        let left_ty = l_ty.clone();
        let right_ty = r_ty.clone();
        
        if op == "+" && matches!(left_ty, Type::Str { .. }) && matches!(right_ty, Type::Char { .. }) {
            let char_tmp = self.fresh_var();
            body.push_str(&format!("char {}[2] = {{ {}, '\\0' }};\n", char_tmp, right_var));
            body.push_str(&format!("Slice_char {}_str = {{ .ptr = {}, .len = 1 }};\n", char_tmp, char_tmp));
            
             
            let tmp = self.fresh_var();
            body.push_str(&format!("size_t {}_len = {}.len + 2;\n", tmp, left_var));
            body.push_str(&format!("char* {}_new = malloc({}_len);\n", tmp, tmp));
            body.push_str(&format!("memcpy({}_new, {}.ptr, {}.len);\n", tmp, left_var, left_var));
            body.push_str(&format!("{}_new[{}.len] = {};\n", tmp, left_var, right_var));
            body.push_str(&format!("{}_new[{}.len + 1] = '\\0';\n", tmp, left_var));
            
            let result = self.fresh_var();
            body.push_str(&format!("Slice_char {} = {{ .ptr = {}_new, .len = {}.len + 1 }};\n", 
                result, tmp, left_var));
            
            return Ok((result, Type::Str { len_type: Box::new(Type::i64()) }));
        }


        if (matches!(l_ty, Type::Str { .. }) || matches!(l_ty, Type::ConstStr)) && op == "+" {
            let mut l_ptr = l_var.clone();
            let mut r_ptr = r_var.clone();

            self.ensure_string_typedef();
            self.ensure_string_concat();

            if matches!(l_ty, Type::ConstStr) {
                let tmp_l = self.fresh_var();
                body.push_str(&format!("String {} = vix_string_from_const({});\n", tmp_l, l_ptr));
                l_ptr = tmp_l;
            }
            
            if matches!(r_ty, Type::ConstStr) {
                let tmp_r = self.fresh_var();
                body.push_str(&format!("String {} = vix_string_from_const({});\n", tmp_r, r_ptr));
                r_ptr = tmp_r;
            } else if matches!(r_ty, Type::Str { .. }) {
                 
            } else {
                 
                let tmp_r = self.fresh_var();
                match r_ty {
                    Type::Int { .. } => {
                        body.push_str(&format!("String {} = vix_int_to_str({});\n", tmp_r, r_ptr));
                    }
                    _ => {
                        body.push_str(&format!("String {} = vix_string_from_const(\"{}\");\n", tmp_r, r_ptr));
                    }
                }
                r_ptr = tmp_r;
            }
            
            let res_tmp = self.fresh_var();
            body.push_str(&format!("String {} = vix_string_concat({}, {});\n", res_tmp, l_ptr, r_ptr));
            return Ok((res_tmp, Type::Str { len_type: Box::new(Type::i64()) }));
        }

        if (matches!(l_ty, Type::Str { .. }) || matches!(l_ty, Type::ConstStr)) &&
        (matches!(r_ty, Type::Str { .. }) || matches!(r_ty, Type::ConstStr)) &&
        matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=") {
            
            let l_ptr = if matches!(l_ty, Type::Str { .. }) {
                format!("{}.ptr", l_var)
            } else {
                l_var.clone()
            };
            
            let r_ptr = if matches!(r_ty, Type::Str { .. }) {
                format!("{}.ptr", r_var)
            } else {
                r_var.clone()
            };
            
            let result = self.codegen_string_compare(&l_ptr, &r_ptr, op, body);
            return Ok((result, Type::Bool));
        }

        if matches!(l_ty, Type::Void) {
            self.diagnostics.error(
                "VoidOperand",
                &format!("Left operand of '{}' cannot be void", op),
                ErrorContext {
                    primary_location: loc.clone(),
                    secondary_locations: vec![],
                    help_message: Some("Void cannot be used in binary operations.".to_string()),
                    suggestions: vec!["Remove the void expression".to_string()],
                }
            );
            return Err(());
        }

        if matches!(r_ty, Type::Void) {
            self.diagnostics.error(
                "VoidOperand",
                &format!("Right operand of '{}' cannot be void", op),
                ErrorContext {
                    primary_location: loc.clone(),
                    secondary_locations: vec![],
                    help_message: Some("Void cannot be used in binary operations.".to_string()),
                    suggestions: vec!["Remove the void expression".to_string()],
                }
            );
            return Err(());
        }
 
        if !self.binop_types_compatible_str(&l_ty, &r_ty, op) {
            let left_loc = left.location();
            let right_loc = right.location();

            self.diagnostics.error(
                "IncompatibleTypes",
                &format!("Cannot apply '{}' to types {} and {}", op, l_ty.name(), r_ty.name()),
                ErrorContext {
                    primary_location: loc,
                    secondary_locations: vec![
                        (left_loc, format!("type: {}", l_ty.name())),
                        (right_loc, format!("type: {}", r_ty.name())),
                    ],
                    help_message: Some(format!(
                        "Operator '{}' requires compatible types.\n    Left: {}, Right: {}",
                        op, l_ty.name(), r_ty.name()
                    )),
                    suggestions: vec![
                        "Cast one operand to match the other type".to_string(),
                        "Use type conversion functions".to_string(),
                    ],
                }
            );
            return Err(());
        }

        if (matches!(l_ty, Type::Str { .. }) || matches!(l_ty, Type::ConstStr)) && op == "+" {
            let mut l_v = l_var;
            let mut r_v = r_var;
            if matches!(l_ty, Type::ConstStr) {
                let tmp_l = self.fresh_var();
                body.push_str(&format!("String {} = vix_string_from_const({});\n", tmp_l, l_v));
                l_v = tmp_l;
            }
            if matches!(r_ty, Type::ConstStr) {
                let tmp_r = self.fresh_var();
                body.push_str(&format!("String {} = vix_string_from_const({});\n", tmp_r, r_v));
                r_v = tmp_r;
            }
            let res_tmp = self.fresh_var();
            body.push_str(&format!("String {} = vix_string_concat({}, {});\n", res_tmp, l_v, r_v));
            return Ok((res_tmp, Type::Str { len_type: Box::new(Type::i64()) }));
        }

       let (c_op, result_ty) = match op {
            "+" => (op, l_ty.clone()),
            "-" => (op, l_ty.clone()),
            "*" => (op, l_ty.clone()),
            "/" => (op, l_ty.clone()),
            "%" => (op, l_ty.clone()),
            "==" => ("==", Type::Bool),
            "!=" => ("!=", Type::Bool),
            "<" => ("<", Type::Bool),
            "<=" => ("<=", Type::Bool),
            ">" => (">", Type::Bool),
            ">=" => (">=", Type::Bool),
            "&&" => ("&&", Type::Bool),
            "||" => ("||", Type::Bool),
            "&" => (op, l_ty.clone()),
            "|" => (op, l_ty.clone()),
            "^" => (op, l_ty.clone()),
            "<<" => (op, l_ty.clone()),
            ">>" => (op, l_ty.clone()),
            _ => {
                self.diagnostics.error(
                    "UnsupportedBinOp",
                    &format!("Binary operator '{}' is not supported", op),
                    ErrorContext {
                        primary_location: loc,
                        secondary_locations: vec![],
                        help_message: Some("This operator is not implemented in the code generator.".to_string()),
                        suggestions: vec!["Use a supported operator".to_string()],
                    }
                );
                return Err(());
            }
        };

        let c_type = result_ty.to_c_type(&self.arch, &mut self.type_registry);
        let tmp = self.fresh_var();
        body.push_str(&format!("{} {} = {} {} {};\n", c_type, tmp, l_var, c_op, r_var));
        Ok((tmp, result_ty))
    }


    pub fn codegen_unop(&mut self, op: &str, operand: &Expr, body: &mut String, loc: SourceLocation) -> Result<(String, Type), ()> {
        let (var, ty) = self.codegen_expr(operand, body) .check_error();
        let tmp = self.fresh_var();
        
        match op {
            "&" => {
                if matches!(ty, Type::Void) {
                    self.diagnostics.error(
                        "VoidAddressOf",
                        "Cannot take address of void expression",
                        void_operation_error("address-of (&)", loc)
                    );
                    return Err(());
                }
                
                let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("{}* {} = &{};\n", c_type, tmp, var));
                Ok((tmp, Type::Ref(Box::new(ty))))
            }
            
            "&mut" => {
                if matches!(ty, Type::Void) {
                    self.diagnostics.error(
                        "VoidAddressOf",
                        "Cannot take mutable address of void expression",
                        void_operation_error("mutable address-of (&mut)", loc)
                    );
                    return Err(());
                }
                
                let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("{}* {} = &{};\n", c_type, tmp, var));
                Ok((tmp, Type::MutRef(Box::new(ty))))
            }
            
            "*" => {
                if let Type::Ptr(inner) = ty {
                    if matches!(*inner, Type::Void) {
                        self.diagnostics.error(
                            "VoidDereference",
                            "Cannot dereference void pointer without cast",
                            dereference_void_error(loc)
                        );
                        return Err(());
                    }
                    
                    let c_type = inner.to_c_type(&self.arch, &mut self.type_registry);
                    body.push_str(&format!("{} {} = *{};\n", c_type, tmp, var));
                    Ok((tmp, *inner))
                } else {
                    let operand_loc = operand.location();
                    
                    self.diagnostics.error(
                        "InvalidDereference",
                        "Cannot dereference non-pointer type",
                        ErrorContext {
                            primary_location: loc,
                            secondary_locations: vec![
                                (operand_loc, format!("type: {}", ty.name())),
                            ],
                            help_message: Some(format!(
                                "Dereference operator (*) can only be applied to pointer types.\n    Got: {}",
                                ty.name()
                            )),
                            suggestions: vec![
                                "Ensure the expression is a pointer".to_string(),
                                "Use address-of (&) to create a pointer first".to_string(),
                            ],
                        }
                    );
                    Err(())
                }
            }
            
            _ => {
                if matches!(ty, Type::Void) {
                    self.diagnostics.error(
                        "VoidUnaryOp",
                        &format!("Cannot apply '{}' to void type", op),
                        void_operation_error(op, loc)
                    );
                    return Err(());
                }
                
                let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("{} {} = {}{};\n", c_type, tmp, op, var));
                Ok((tmp, ty))
            }
        }
    }

    pub fn codegen_typed_declaration(&mut self, name: &str, ty: &Type, value: &Expr, body: &mut String, loc: SourceLocation) -> Result<(), ()> {
        let (val_var, val_ty) = self.codegen_expr(value, body) .check_error();
        let c_name = format!("var_{}", name);
        
 
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
                body.push_str(&format!("struct {{ {}* ptr; size_t len; }} {} = {};\n", elem_c_type, c_name, val_var));
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
                
                let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("{} {} = {};\n", c_type, c_name, val_var));
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
                
                let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("{} {} = {};\n", c_type, c_name, val_var));
            }
            
            Type::Ptr(inner) if matches!(**inner, Type::Void) => {
 
                let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("{} {} = {};\n", c_type, c_name, val_var));
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
                
                let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                match (ty, &val_ty) {
                    (Type::ConstStr, Type::Str { .. }) => {
                        body.push_str(&format!("{} {} = {}.ptr;\n", c_type, c_name, val_var));
                    }
                    _ => {
                        body.push_str(&format!("{} {} = {};\n", c_type, c_name, val_var));
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



    fn binop_types_compatible(&self, left: &Type, right: &Type, op: &str) -> bool {
        match op {
            "+" | "-" | "*" | "/" | "%" => {
                matches!((left, right), 
                    (Type::Int { .. }, Type::Int { .. }) |
                    (Type::Float { .. }, Type::Float { .. })
                )
            }
            
            "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                matches!((left, right),
                    (Type::Int { .. }, Type::Int { .. }) |
                    (Type::Float { .. }, Type::Float { .. }) |
                    (Type::Bool, Type::Bool) |
                    (Type::Ptr(_), Type::Ptr(_))
                )
            }
            
            "&&" | "||" => {
                matches!((left, right),
                    (Type::Bool, Type::Bool) |
                    (Type::Int { .. }, Type::Int { .. })
                )
            }
            
            "&" | "|" | "^" | "<<" | ">>" => {
                matches!((left, right),
                    (Type::Int { .. }, Type::Int { .. })
                )
            }
            
            _ => false,
        }
    }

    fn binop_types_compatible_str(&self, left: &Type, right: &Type, op: &str) -> bool {
        match op {
            "+" | "-" | "*" | "/" | "%" => {
                if matches!((left, right), (Type::Int { .. }, Type::Int { .. }) | (Type::Float { .. }, Type::Float { .. })) {
                    true
                } else { op == "+" && (matches!(left, Type::Str { .. } | Type::ConstStr) || matches!(right, Type::Str { .. } | Type::ConstStr)) }
            }

 
            "&" | "|" | "^" | "<<" | ">>" => {
                matches!((left, right),
                    (Type::Int { .. }, Type::Int { .. })
                )
            }

 
            "==" | "!=" | "<" | "<=" | ">" | ">=" => {
                matches!((left, right),
                    (Type::Int { .. }, Type::Int { .. }) |
                    (Type::Float { .. }, Type::Float { .. }) |
                    (Type::Bool, Type::Bool) |
                    (Type::Char { bits: _, signed: _ }, Type::Char { bits: _, signed: _ }) |
                    (Type::Ptr(_), Type::Ptr(_))
                )
            }

            "&&" | "||" => {
                matches!((left, right),
                    (Type::Bool, Type::Bool) |
                    (Type::Int { .. }, Type::Int { .. })
                )
            }

            _ => false,
        }
    }

    pub fn codegen_union_value(&mut self, value: &Expr, expected_ty: &Type, body: &mut String) -> Result<(String, Type), ()> {
        let (val_var, val_ty) = self.codegen_expr(value, body)?;
        
        if let Type::Union { variants } = expected_ty {
            for (idx, variant) in variants.iter().enumerate() {
                if self.types_compatible(variant, &val_ty) {
                    let tmp = self.fresh_var();
                    let union_type_name = self.type_registry.get_union_type_name(variants, &self.arch);
                    
                    body.push_str(&format!("{} {};\n", union_type_name, tmp));
                    body.push_str(&format!("{}.tag = {};\n", tmp, idx));
                    
                    let field_name = format!("variant_{}", idx);
                    body.push_str(&format!("{}.data.{} = {};\n", tmp, field_name, val_var));
                    
                    return Ok((tmp, expected_ty.clone()));
                }
            }
        }
        
        Ok((val_var, val_ty))
    }
}