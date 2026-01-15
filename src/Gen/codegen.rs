use crate::import::*;

pub trait ErrorCheck<T> {
    fn check_error(self) -> T;
}

impl<T> ErrorCheck<T> for Result<T, anyhow::Error> 
where 
    T: Default,
{
    fn check_error(self) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                eprintln!("Error: {}", e);
                T::default()
            }
        }
    }
}

impl Codegen {
    pub fn error_value() -> (String, Type) {
        (String::from("_error_"), Type::Void)
    }
    
    pub fn error_var(&mut self) -> String {
        let tmp = self.fresh_var();
        format!("{}_error", tmp)
    }
}

pub struct CodegenResult(pub String, pub Type);

impl Default for CodegenResult {
    fn default() -> Self {
        CodegenResult(String::from("_error_"), Type::Void)
    }
}


impl From<CodegenResult> for (String, Type) {
    fn from(cr: CodegenResult) -> Self {
        (cr.0, cr.1)
    }
}

impl<T> ErrorCheck<T> for Result<T, ()> 
where
    T: From<CodegenResult>
{
    fn check_error(self) -> T {
        match self {
            Ok(value) => value,
            Err(_) => T::from(CodegenResult::default())
        }
    }
}
impl Codegen {
    pub fn new(arch: ArchConfig, source_code: String, filename: String) -> Self {
        let diagnostics = DiagnosticHandler::new(source_code.clone());
        
        Codegen {
            config: CodegenConfig { 
                arch: arch.clone(),
                optimization_level: OptimizationLevel::default(),
                debug_info: false,
            },
            type_registry: TypeRegistry::new(),
            impl_methods: HashMap::new(),
            c_code: String::new(),
            globals: String::new(),
            var_count: 0,
            label_count: 0,
            vars: HashMap::new(),
            owned_vars: HashSet::new(),
            extern_functions: HashMap::new(),
            extern_block: HashMap::new(),
            structs: HashMap::new(),
            module_vars: HashMap::new(),
            module_functions: HashMap::new(),
            compilation_mode: CompilationMode::default(),
            exported_functions: Vec::new(),
            externs_with_bodies: HashSet::new(),
            unsafe_depth: 0,
            scope_depth: 0,
            user_functions: HashMap::new(),
            linked_libraries: Vec::new(),
            ir: IR::new(),
            arch,
            diagnostics,
            source_code,
            current_file: filename,
            current_return_type: None,
        }
    }

    pub fn ensure_type_defined(&mut self, ty: &Type) {
        match ty {
            Type::Result { ok, err } => {
                self.ensure_type_defined(ok);
                self.ensure_type_defined(err);
            }
            Type::Option { inner } => {
                self.ensure_type_defined(inner);
            }
            Type::Tuple { fields } => {
                for field in fields {
                    self.ensure_type_defined(field);
                }
            }
            Type::Union { variants } => {
                for variant in variants {
                    self.ensure_type_defined(variant);
                }
            }
            Type::Array { element, .. } => {
                self.ensure_type_defined(element);
            }
            Type::Ptr(inner) | Type::RawPtr(inner) | Type::Owned(inner) | Type::Ref(inner) | Type::MutRef(inner) | Type::Const(inner) => {
                self.ensure_type_defined(inner);
            }
            _ => {}
        }

        if let Some(def) = self.type_registry.generate_type_definition(ty, &self.arch) {
            if !self.ir.forward_decls.contains(&def) {
                self.ir.forward_decls.push_str(&def);
                self.ir.forward_decls.push_str("\n");
            }
        }
    }

    fn make_location(&self, span: &SourceSpan) -> SourceLocation {
        let offset = span.offset();
        let len = span.len();
        
        let line = self.source_code[..offset]
            .chars()
            .filter(|&c| c == '\n')
            .count() + 1;
        
        let line_start = self.source_code[..offset]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        
        let column = offset - line_start + 1;
        let length = len.max(1);
        
        SourceLocation {
            file: self.current_file.clone(),
            line,
            column,
            length,
        }
    }
    pub fn default_location(&self) -> SourceLocation {
        SourceLocation {
            file: self.current_file.clone(),
            line: 0,
            column: 0,
            length: 1,
        }
    }

    pub fn fresh_label(&mut self) -> String {
        let label = format!("label_{}", self.label_count);
        self.label_count += 1;
        label
    }

    pub fn codegen_stmt(&mut self, stmt: &Stmt, body: &mut String) -> Result<(), ()> {
        let loc = self.default_location();
        
        match stmt {
            Stmt::TypedDeclaration { name, ty, value, is_mutable, .. } => {
                self.codegen_typed_declaration_impl(name, ty, value, body, loc, *is_mutable)
            }
            Stmt::Assign(name, value) => self.codegen_assign(name, value, body, loc),
            Stmt::TupleUnpack { names, value } => self.codegen_tuple_unpack(names, value, body, loc),
            Stmt::Match(expr, cases, default) => {self.codegen_match(expr, cases, default, body);  Ok(()) }
            Stmt::CompoundAssign(name, op, value) => self.codegen_compound_assign(name, op, value, body, loc),
            Stmt::IndexAssign(arr, indices, value) => self.codegen_index_assign(arr, indices, value, body, loc),
            Stmt::MemberAssign(obj, field, value) => self.codegen_member_assign(obj, field, value, body, loc),
            Stmt::If(cond, then_body, else_body) => self.codegen_if(cond, then_body, else_body, body),
            Stmt::While(cond, loop_body) => self.codegen_while(cond, loop_body, body, loc),
            Stmt::For(var, iter, loop_body) => self.codegen_for(var, iter, loop_body, body, loc),
            Stmt::Return(expr) => self.codegen_return(expr, body).map_err(|_| ()),
            Stmt::Call(func, args) => self.codegen_call_stmt(func, args, body, loc),
            Stmt::Break => self.codegen_break(body).map_err(|_| ()),
            Stmt::Continue => self.codegen_continue(body).map_err(|_| ()),
            Stmt::Scope(stmts) => self.codegen_scope(stmts, body),
            Stmt::StructDef(s) => self.codegen_struct_definition(s),
            Stmt::EnumDef(e) => self.codegen_enum_definition(e),
            Stmt::MemberCompoundAssign(obj, field, op, value) => {self.codegen_member_compound_assign(obj, field, op, value, body, loc)}
            Stmt::Expr(expr) => {
                self.codegen_expr(expr, body).check_error();
                Ok(())
            }
            _ => {
                self.diagnostics.warning(
                    "UnsupportedStatement",
                    &format!("Skipping unsupported statement type: {:?}", stmt),
                    ErrorContext {
                        primary_location: loc,
                        secondary_locations: vec![],
                        help_message: Some("This statement type is not yet implemented.".to_string()),
                        suggestions: vec![],
                    }
                );
                Ok(())
            }
        }
    }

pub fn codegen_expr(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()> {
    let loc = self.default_location();
    match expr {
        Expr::Number(n) => Ok(self.codegen_number(*n, body)),
        Expr::Float(f) => Ok(self.codegen_float(*f, body)),
        Expr::Bool(b) => Ok(self.codegen_bool(*b, body)),
        Expr::Char(c) => Ok(self.codegen_char(*c, body)),
        Expr::HexNumber(n) => Ok(self.codegen_hex_number(*n, body)),
        Expr::BinaryNumber(n) => Ok(self.codegen_binary_number(*n, body)),
        Expr::OctalNumber(n) => Ok(self.codegen_octal_number(*n, body)),
        Expr::String(s) => Ok(self.codegen_string(s, body)),
        Expr::Var(name) => self.codegen_var(name, loc),
        Expr::BinOp(op, left, right) => self.codegen_binop(op, left, right, body, loc),
        Expr::UnOp(op, operand) => self.codegen_unop(op, operand, body, loc),
        Expr::Call(name, args) if self.structs.contains_key(name) => {
            let constructor_name = format!("{}_new", name);
            
            let mut arg_vars = Vec::new();
            for arg in args {
                let (var, _ty) = self.codegen_expr(arg, body).check_error();
                arg_vars.push(var);
            }
            
            let tmp = self.fresh_var();
            let args_str = arg_vars.join(", ");
            
            body.push_str(&format!("{} {} = {}({});\n", name, tmp, constructor_name, args_str));
            return Ok((tmp, Type::Struct { name: name.clone() }));
        }
        
        Expr::Call(func, args) => self.codegen_call_expr(func, args, body, loc),
        Expr::Array(elements) => self.codegen_array(elements, body),
        Expr::Index(arr, indices) => self.codegen_index(arr, indices, body),
        Expr::MemberAccess(obj, field) => self.codegen_member_access(obj, field, body, loc),
        Expr::ResultOk(inner) => {self.codegen_result_ok(inner, body)}
        Expr::ResultErr(inner) => {self.codegen_result_err(inner, body)}
        Expr::Not(expr) => self.codegen_not(expr, body).map_err(|_| ()),
        Expr::Tuple(elements) => self.codegen_tuple(elements, body),
        Expr::MethodCall(obj, method, args) => self.codegen_method_call(obj, method, args, body, loc),
        Expr::ModuleCall(module, func, args) => self.codegen_module_call(module, func, args, body, loc),
        Expr::Cast(expr, target) => self.codegen_cast_target(expr, target, body, loc),
        Expr::StaticMethodCall(type_name, method, args) => {
            self.codegen_static_method(type_name, method, args, body, loc)
        }
        Expr::CallNamed(name, named_args) => {
            if self.structs.contains_key(name) {
                let constructor_name = format!("{}_new", name);
                
                let mut arg_vars = Vec::new();
                for (_arg_name, arg_expr) in named_args {
                    let (var, _ty) = self.codegen_expr(arg_expr, body).check_error();
                    arg_vars.push(var);
                }
                
                let tmp = self.fresh_var();
                let args_str = arg_vars.join(", ");
                
                body.push_str(&format!("{} {} = {}({});\n", name, tmp, constructor_name, args_str));
                return Ok((tmp, Type::Struct { name: name.clone() }));
            }
            
             
            let mut arg_vars = Vec::new();
            for (_arg_name, arg_expr) in named_args {
                let (var, _ty) = self.codegen_expr(arg_expr, body).check_error();
                arg_vars.push(var);
            }
            
            let tmp = self.fresh_var();
            let args_str = arg_vars.join(", ");
            body.push_str(&format!("int32_t {} = {}({});\n", tmp, name, args_str));
            Ok((tmp, Type::i32()))
        }

        Expr::SizeOf(ty) => {
            let tmp = self.fresh_var();
            let c_type = ty.to_c_type(&self.arch);
            body.push_str(&format!("size_t {} = sizeof({});\n", tmp, c_type));
            Ok((tmp, Type::i64()))
        }
        Expr::AlignOf(ty) => {
            let tmp = self.fresh_var();
            let c_type = ty.to_c_type(&self.arch);
            body.push_str(&format!("size_t {} = _Alignof({});\n", tmp, c_type));
            Ok((tmp, Type::i64()))
        }
        Expr::TypeOf(expr) => {
            let (var, ty) = self.codegen_expr(expr, body)?;
            Ok((var, ty))
        }

        Expr::None => {
            let tmp = self.fresh_var();
            body.push_str(&format!("void* {} = NULL;\n", tmp));
            Ok((tmp, Type::Ptr(Box::new(Type::Void))))
        }
        Expr::Some(inner) => {
            self.codegen_some(inner, body)
        }

        Expr::Pipe(left, right) => {
            let _ = self.codegen_expr(left, body)?;
            match right.as_ref() {
                Expr::Call(func, args) => {
                    let mut new_args = vec![*left.clone()];
                    new_args.extend(args.clone());
                    self.codegen_call_expr(func, &new_args, body, loc)
                }
                _ => self.codegen_expr(right, body),
            }
        }

        _ => {
            self.diagnostics.error(
                "UnsupportedExpression",
                &format!("Unsupported expression type: {:?}", expr),
                ErrorContext {
                    primary_location: loc,
                    secondary_locations: vec![],
                    help_message: Some("This expression type is not yet implemented.".to_string()),
                    suggestions: vec!["Check if there's a typo or use a supported expression".to_string()],
                }
            );
            Err(())
        }
    }
}


    

    pub fn codegen_program_full(
        &mut self, 
        program: &Program,
        structs: &[StructDef],
        enums: &[EnumDef],
        impls: &[ImplBlock],
        externs: &[ExternDecl],
        library_includes: &[String],  
        library_functions: &[FunctionSignature],  
    ) -> Result<String, String> {
         
        for include in library_includes {
            self.ir.headers.push_str(include);
            self.ir.headers.push('\n');
        }
        
         
        for func_sig in library_functions {
            let c_decl = format!("{} {}({});", 
                func_sig.return_type, 
                func_sig.name, 
                if func_sig.parameters.is_empty() {
                    continue;
                } else {
                    func_sig.parameters.iter().map(|(name, ty)| format!("{} {}", ty, name)).collect::<Vec<_>>().join(", ")
                }
            );
            self.ir.forward_decls.push_str(&c_decl);
            self.ir.forward_decls.push_str("\n");
        }
        
        println!("   {} Generating struct definitions...", "processing:".bright_black());
        
        for struct_def in structs {
            self.codegen_struct_definition(struct_def);
        }

        for enum_def in enums {
            self.codegen_enum_definition(enum_def);
        }

        println!("   {} Registering function/method signatures...", "processing:".bright_black());
        for func in &program.functions {
            self.codegen_function(func, true);
        }
        for impl_block in impls {
            let _ = self.codegen_impl_block(impl_block, true);
        }

        if let Err(_) = self.codegen_externs(externs) {
           
        }

        println!("   {} Generating function code...", "processing:".bright_black());
        for func in &program.functions {
            self.codegen_function(func, false);
        }

        println!("   {} Generating impl block code...", "processing:".bright_black());
        for impl_block in impls {
            if let Err(_) = self.codegen_impl_block(impl_block, false) {
               
            }
        }
         
        if self.diagnostics.has_errors() {
            println!();
            self.diagnostics.print_summary();
            println!("Code generation failed due to errors");
        }
        
        if self.diagnostics.warning_count > 0 {
            println!("   {} {} warning(s) generated", "Warning:".yellow(), self.diagnostics.warning_count);
        }

         
        self.ir.functions.push_str("\nint main() {\n    vix_main();\n    return 0;\n}\n");
        
        Ok(self.ir.clone().finalize())
    }
}
