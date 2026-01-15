use crate::import::*;

impl Codegen {
    pub fn fresh_var(&mut self) -> String {
        let varaable = format!("t{}", self.var_count);
        self.var_count += 1;

        varaable
    }

    pub fn codegen_var(&mut self, name: &str, loc: SourceLocation) -> Result<(String, Type), ()> {
        if let Some((c_name, ty)) = self.vars.get(name) {
            return Ok((c_name.clone(), ty.clone()));
        }
        
        for ((_, var_name), (c_name, ty, _)) in &self.module_vars {
            if var_name == name {
                return Ok((c_name.clone(), ty.clone()));
            }
        }
        
        self.diagnostics.error(
            "UndefinedVariable",
            &format!("Variable '{}' has not been declared in this scope.", name),
            ErrorContext {
                primary_location: loc.clone(),
                secondary_locations: vec![],
                help_message: Some(format!(
                    "Cannot find variable '{}' in the current scope.", 
                    name
                )),
                suggestions: vec![
                    format!("Declare '{}' before using it", name),
                    format!("Use 'let {} = ...' to declare and initialize", name),
                    "Check for typos in the variable name".to_string(),
                ],
            }
        );
        
        Err(())
    }

    
    pub fn codegen_break(&self, body: &mut String) -> Result<(), ()> {
        body.push_str("break;\n");
        Ok(())
    }

    pub fn codegen_continue(&self, body: &mut String) -> Result<(), ()> {
        body.push_str("continue;\n");
        Ok(())
    }

    pub fn codegen_match(&mut self, expr: &Expr, cases: &[MatchCase], default: &Option<Vec<Stmt>>, body: &mut String){
        let (match_var, _match_ty) = self.codegen_expr(expr, body).check_error();

        let end_label = self.fresh_label();

        for case in cases {
            let (case_val, _) = self.codegen_expr(&case.value, body).check_error();
            
            let _case_label = self.fresh_label();
            body.push_str(&format!("if ({} == {}) {{\n", match_var, case_val));

            for stmt in &case.body {
                self.codegen_stmt(stmt, body) ;
            }

            body.push_str(&format!("goto {};\n}}\n", end_label));
        }

        if let Some(default_body) = default {
            for stmt in default_body {
                self.codegen_stmt(stmt, body);
            }
        }

        body.push_str(&format!("{}:\n", end_label));
    }


    pub fn codegen_not(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()> {
        let (var, ty) = self.codegen_expr(expr, body).check_error();
        let tmp = self.fresh_var();
        
        match ty {
            Type::Bool | Type::Int { .. } => {
                body.push_str(&format!("bool {} = !{};\n", tmp, var));
                Ok((tmp, Type::Bool))
            }
            _ => {
                self.diagnostics.error(
                    "InvalidNot",
                    &format!("Cannot apply NOT operator to type {}", ty.name()),
                    ErrorContext {
                        primary_location: self.default_location(),
                        secondary_locations: vec![],
                        help_message: Some("NOT operator can only be applied to boolean or integer types.".to_string()),
                        suggestions: vec![
                            "Convert the expression to a boolean first".to_string(),
                        ],
                    }
                );
                Err(())
            }
        }
    }



 
    pub fn codegen_return(&mut self, expr: &Option<Expr>, body: &mut String) -> Result<(), ()> {
        if let Some(e) = expr {
            let (var, ty) = self.codegen_expr(e, body).check_error();
            
            if !matches!(ty, Type::Void) {
                body.push_str(&format!("return {};\n", var));
                    } else {
                body.push_str("return;\n");
            }
        } else {
            body.push_str("return;\n");
        }
        Ok(())
    }
}
