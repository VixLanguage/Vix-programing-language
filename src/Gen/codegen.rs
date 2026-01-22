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
            externs_bodies: HashSet::new(),
            unsafe_depth: 0,
            scope_depth: 0,
            user_functions: HashMap::new(),
            linked_libraries: Vec::new(),
            ir: IR::new(),
            arch,
            diagnostics,
            source_code,
            current_file: filename,
            module_function_signatures: HashMap::new(),
            module_init_functions: Vec::new(),
            current_return_type: None,
        }
    }

    pub fn ensure_type_defined(&mut self, ty: &Type) {
    match ty {
        Type::Str { .. } | Type::ConstStr => {
            self.ensure_string_typedef();
        }
        Type::Array { element, .. } => {
            self.ensure_type_defined(element);
            
            let elem_c_type = element.to_c_type(&self.arch, &mut self.type_registry);
            let slice_type_name = self.type_registry.generate_slices(element, &self.arch);
            let typedef = format!(
                "typedef struct {{\n    {}* ptr;\n    size_t len;\n}} {};\n", elem_c_type, slice_type_name);

            if !self.ir.type_definitions.contains(&typedef) {
                self.ir.type_definitions.push_str(&typedef);
                self.ir.type_definitions.push('\n');
            }
        }

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
        
        Type::Ptr(inner) | Type::RawPtr(inner) | Type::Owned(inner) | Type::Ref(inner) | Type::MutRef(inner) | Type::Const(inner) => {
            self.ensure_type_defined(inner);
        }
        _ => {}
    }

    if let Some(def) = self.type_registry.generate_type_definition(ty, &self.arch)
        && !self.ir.type_definitions.contains(&def) {
            self.ir.type_definitions.push_str(&def);
            self.ir.type_definitions.push('\n');
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
            Stmt::TupleUnpack { names, value } => self.codegen_tuple_unpack(names, value, body, loc).map_err(|_| ()),
            Stmt::Match(expr, cases, default) => {self.codegen_match(expr, cases, default, body);  Ok(()) }
            Stmt::CompoundAssign(name, op, value) => self.codegen_compound_assign(name, op, value, body, loc),
            Stmt::IndexAssign(arr, indices, value) => self.codegen_index_assign(arr, indices, value, body, loc),
            Stmt::MemberAssign(obj, field, value) => self.codegen_member_assign(obj, field, value, body, loc),
            Stmt::If(cond, then_body, else_body) => self.codegen_if(cond, then_body, else_body, body),
            Stmt::IfLet { pattern, value, then_block, else_block } => self.codegen_if_let(pattern, value, then_block, else_block, body).map_err(|_| ()),
            Stmt::While(cond, loop_body) => self.codegen_while(cond, loop_body, body, loc),
            Stmt::For(var, iter, loop_body) => self.codegen_for(var, iter, loop_body, body, loc),
            Stmt::Return(expr) => self.codegen_return(expr, body).map_err(|_| ()),
            Stmt::Call(func, args) => self.codegen_call_stmt(func, args, body, loc),
            Stmt::Break => self.codegen_break(body).map_err(|_| ()),
            Stmt::Continue => self.codegen_continue(body).map_err(|_| ()),
            Stmt::Scope(stmts) => self.codegen_scope(stmts, body),
            Stmt::StructDef(s) => self.codegen_struct_definition(s),
            Stmt::EnumDef(e) => self.codegen_enum_definition(e),
            Stmt::Function(f) => {
                self.codegen_function(f, true);
                self.codegen_function(f, false);
                Ok(())
            }
            Stmt::ModuleDef { .. } => {
                self.diagnostics.error(
                    "UnsupportedNestedModule",
                    "Modules cannot be defined inside functions.",
                    ErrorContext {
                        primary_location: loc,
                        secondary_locations: vec![],
                        help_message: Some("Move the module to the top level.".to_string()),
                        suggestions: vec![],
                    }
                );
                Ok(())
            }
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
                Ok((tmp, Type::Struct { name: name.clone() }))
            }
            Expr::Call(func, args) => {
                eprintln!("[DEBUG] codegen.rs Expr::Call: {}", func);
                self.codegen_call_expr(func, args, body, loc)
            },
            Expr::Array(elements) => self.codegen_array(elements, body),
            Expr::MemberAccess(obj, field) => self.codegen_member_access(obj, field, body, loc),
            Expr::ResultOk(inner) => {self.codegen_result_ok(inner, body)}
            Expr::ResultErr(inner) => {self.codegen_result_err(inner, body)}
            Expr::Not(expr) => self.codegen_not(expr, body).map_err(|_| ()),
            Expr::Tuple(elements) => self.codegen_tuple(elements, body),
            Expr::MethodCall(obj, method, args) => self.codegen_method_call(obj, method, args, body, loc),
            Expr::ModuleCall(module, func, args) => self.codegen_module_call(module, func, args, body, loc),
            Expr::Cast(expr, target) => self.codegen_cast_target(expr, target, body, loc),
            Expr::StaticMethodCall(type_name, method, args) => {self.codegen_static_method(type_name, method, args, body, loc)}
            Expr::MethodCall(obj, method, args) => {
                let (obj_var, obj_ty) = self.codegen_expr(obj, body)?;
                let struct_name = match &obj_ty {
                    Type::Struct { name } => name.clone(),
                    _ => return Err(()),
                };
                
                let method_name = format!("{}_{}", struct_name, method);
                let mut arg_vars = vec![format!("&{}", obj_var)];
                
                for arg in args {
                    let (var, _) = self.codegen_expr(arg, body)?;
                    arg_vars.push(var);
                }
                
                let return_type = self.impl_methods
                    .get(&(struct_name, method.to_string()))
                    .map(|(_, ret_ty, _)| ret_ty.clone())
                    .unwrap_or(Type::i32());
                
                let tmp = self.fresh_var();
                let c_type = return_type.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("{} {} = {}({});\n", 
                    c_type, tmp, method_name, arg_vars.join(", ")));
                
                Ok((tmp, return_type))
            }

            Expr::Call(name, args) if self.structs.contains_key(name) && args.is_empty() => {
                 
                let constructor_name = format!("{}_new", name);
                let struct_info = self.structs.get(name).unwrap();
                
                let mut default_args = Vec::new();
                for (_, field_ty, _) in &struct_info.fields {
                    let default_val = match field_ty {
                        Type::Int { .. } => "0",
                        Type::Str { .. } => "(Slice_char){ .ptr = \"\", .len = 0 }",
                        _ => "{ 0 }",
                    };
                    default_args.push(default_val.to_string());
                }
                
                let tmp = self.fresh_var();
                body.push_str(&format!("{} {} = {}({});\n", 
                    name, tmp, constructor_name, default_args.join(", ")));
                
                Ok((tmp, Type::Struct { name: name.clone() }))
            }
            Expr::Index(arr, indices) => {
                let (arr_var, arr_ty) = self.codegen_expr(arr, body)?;
                
                let is_hashmap = match &arr_ty {
                    Type::HashMap { .. } => true,
                    Type::Ref(inner) | Type::MutRef(inner) => matches!(**inner, Type::HashMap { .. }),
                    _ => false,
                };

                if is_hashmap
                    && indices.len() == 1 {
                        return self.codegen_hashmap_get(arr, &indices[0], body);
                    }
    
                self.codegen_index(arr, indices, body)
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

            Expr::Slice(arr, start, end) => {
                let (arr_var, arr_ty) = self.codegen_expr(arr, body)?;
                let (start_var, _) = self.codegen_expr(start, body)?;
                let (end_var, _) = self.codegen_expr(end, body)?;
                
                let elem_ty = match &arr_ty {
                    Type::Array { element, .. } => *element.clone(),
                    _ => Type::Void,
                };
                
                let tmp = self.fresh_var();
                let slice_type = self.type_registry.generate_slices(&elem_ty, &self.arch);
                
                body.push_str(&format!("{} {} = {{ .ptr = &{}.ptr[{}], .len = {} - {} }};\n", 
                    slice_type, tmp, arr_var, start_var, end_var, start_var));
                
                Ok((tmp, Type::Array { element: Box::new(elem_ty), size: None }))
            }

            Expr::HashMap(entries) => {
                if entries.is_empty() {
                    return Err(());
                }
                
                 
                let (first_key, first_val) = &entries[0];
                let (_, key_ty) = self.codegen_expr(first_key, body)?;
                let (_, val_ty) = self.codegen_expr(first_val, body)?;
                
                 
                let hashmap_ty = Type::HashMap { 
                    key: Box::new(key_ty.clone()), 
                    value: Box::new(val_ty.clone()) 
                };
                
                if let Some(def) = self.type_registry.generate_hashmap_definition(&key_ty, &val_ty, &self.arch)
                    && !self.ir.forward_decls.contains(&def) {
                        self.ir.forward_decls.push_str(&def);
                        self.ir.forward_decls.push('\n');
                    }
                
                let hashmap_name = format!("HashMap_{}_{}", 
                    key_ty.name().replace(" ", "_"), 
                    val_ty.name().replace(" ", "_"));
                
                let tmp = self.fresh_var();
                
                 
                body.push_str(&format!("{} {} = {{0}};\n", hashmap_name, tmp));
                body.push_str(&format!("{}.size = 0;\n", tmp));
                body.push_str(&format!("for (int i = 0; i < 256; i++) {{ {}.entries[i].occupied = false; }}\n", tmp));
                
                 
                for (key_expr, val_expr) in entries {
                    let (key_var, _) = self.codegen_expr(key_expr, body)?;
                    let (val_var, _) = self.codegen_expr(val_expr, body)?;
                    
                    body.push_str(&format!("insert_{}(&{}, {}, {});\n", 
                        hashmap_name, tmp, key_var, val_var));
                }
                
                Ok((tmp, hashmap_ty))
            }

            Expr::SizeOf(ty) => {
                let tmp = self.fresh_var();
                let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
                body.push_str(&format!("size_t {} = sizeof({});\n", tmp, c_type));
                Ok((tmp, Type::i64()))
            }
            Expr::AlignOf(ty) => {
                let tmp = self.fresh_var();
                let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
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



 

pub fn codegen_library(
    &mut self, 
    program: &Program,
    structs: &[StructDef],
    enums: &[EnumDef],
    impls: &[ImplBlock],
    externs: &[ExternDecl],
    library_includes: &[String],
) -> Result<String, String> {
    println!("   {} Starting library codegen...", "→".bright_cyan());
    
     
    for include in library_includes {
        self.ir.headers.push_str(include);
        self.ir.headers.push('\n');
    }
    
    println!("   {} Generating struct definitions...", "→".bright_black());
    
     
    for struct_def in structs {
        if self.codegen_struct_definition(struct_def).is_err() {
            eprintln!("   {} Failed to generate struct: {}", "Error:".red(), struct_def.name);
        }
    }

    println!("   {} Generating enum definitions...", "→".bright_black());
    for enum_def in enums {
        if self.codegen_enum_definition(enum_def).is_err() {
            eprintln!("   {} Failed to generate enum: {}", "Error:".red(), enum_def.name);
        }
    }

    println!("   {} Processing {} modules...", "→".bright_black(), program.modules.len());
     
    for module in &program.modules {
        if let Stmt::ModuleDef { name, .. } = module {
            println!("      {} Processing module: {}", "→".bright_black(), name);
        }
        if self.codegen_module(module).is_err() {
            eprintln!("   {} Failed to generate module", "Error:".red());
        }
    }

    println!("   {} Ensuring type definitions...", "→".bright_black());
     
    for func in &program.functions {
        self.ensure_type_defined(&func.return_type);
        for (_, param_ty, _) in &func.params {
            self.ensure_type_defined(param_ty);
        }
    }

    for impl_block in impls {
        for method in &impl_block.methods {
            self.ensure_type_defined(&method.return_type);
            for (_, param_ty, _) in &method.params {
                self.ensure_type_defined(param_ty);
            }
        }
    }

    println!("   {} Generating {} function declarations...", "→".bright_black(), program.functions.len());
     
    for func in &program.functions {
        self.codegen_function(func, true);
    }
    for impl_block in impls {
        let _ = self.codegen_impl_block(impl_block, true);
    }

    if self.codegen_externs(externs).is_err() {
        eprintln!("   {} Failed to generate externs", "Error:".red());
    }

    println!("   {} Generating {} function implementations...", "→".bright_black(), program.functions.len());
     
    for func in &program.functions {
        println!("      {} Generating: {}", "→".bright_black(), func.name);
        self.codegen_function(func, false);
    }

    for impl_block in impls {
        if let Err(_) = self.codegen_impl_block(impl_block, false) {
            eprintln!("   {} Failed to generate impl block", "Error:".red());
        }
    }
    
    if self.diagnostics.has_errors() {
        println!("   {} Diagnostics has errors, printing summary...", "Error:".red());
        self.diagnostics.print_summary();
        return Err("Code generation failed due to errors".to_string());
    }
    
    if self.diagnostics.warning_count > 0 {
        println!("   {} {} warning(s) generated", "Warning:".yellow(), self.diagnostics.warning_count);
    }
    
    println!("   {} Finalizing library code...", "→".bright_black());
    
    for def in &self.type_registry.ordered_definitions {
        if !self.ir.forward_decls.contains(def) {
            self.ir.forward_decls.push_str(def);
        }
    }

    println!("   {} Library codegen complete!", "✓".green());
    Ok(self.ir.clone().finalize())
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
        
        println!("   {} Processing {} library function signatures...", "→".bright_cyan(), library_functions.len());
        
        for func_sig in library_functions {
            println!("      {} Registering library function: {}", "→".bright_black(), func_sig.name);
            
             
            let param_str = if func_sig.parameters.is_empty() {
                "void".to_string()
            } else {
                func_sig.parameters.iter()
                    .map(|(name, ty)| format!("{} {}", ty, name))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            
            let c_decl = format!("{} {}({});", 
                func_sig.return_type, 
                func_sig.name,
                param_str
            );
            self.ir.forward_decls.push_str(&c_decl);
            self.ir.forward_decls.push('\n');
            
             
             
            let return_type = self.parse_c_type_to_vix_type(&func_sig.return_type);
            
             
            let params: Vec<(String, Type)> = func_sig.parameters.iter()
                .map(|(name, ty_str)| {
                    let ty = self.parse_c_type_to_vix_type(ty_str);
                    (name.clone(), ty)
                })
                .collect();
            
             
            self.user_functions.insert(
                func_sig.name.clone(),
                (params, return_type)
            );
        }
        
            self.ensure_string_typedef();
        self.type_registry.generate_slices(&Type::char8(), &self.arch);
        
        println!("   {} Generating struct definitions...", "processing:".bright_black());
        
        for struct_def in structs {
            self.codegen_struct_definition(struct_def).ok();
        }

        for enum_def in enums {
            self.codegen_enum_definition(enum_def).ok();
        }

        println!("   {} Processing modules...", "processing:".bright_black());
        for module in &program.modules {
            if self.codegen_module(module).is_err() {
            }
        }

         
        println!("   {} Pre-processing type definitions...", "processing:".bright_black());
        for func in &program.functions {
            self.ensure_type_defined(&func.return_type);
            for (_, param_ty, _) in &func.params {
                self.ensure_type_defined(param_ty);
            }
        }

        for impl_block in impls {
            for method in &impl_block.methods {
                self.ensure_type_defined(&method.return_type);
                for (_, param_ty, _) in &method.params {
                    self.ensure_type_defined(param_ty);
                }
            }
        }

        for func in &program.functions {
            self.codegen_function(func, true);
        }
        for impl_block in impls {
            let _ = self.codegen_impl_block(impl_block, true);
        }

        self.codegen_externs(externs).is_err();

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

        for init_func in &self.module_init_functions {
             self.ir.functions.push_str(&format!("    {}();\n", init_func));
        }

        Ok(self.ir.clone().finalize())
    }

        

    pub fn parse_c_type_to_vix_type(&self, c_type: &str) -> Type {
        match c_type.trim() {
            "void" => Type::Void,
            "bool" => Type::Bool,
            "int8_t" => Type::i8(),
            "int16_t" => Type::i16(),
            "int32_t" | "int" => Type::i32(),
            "int64_t" => Type::i64(),
            "uint8_t" => Type::u8(),
            "uint16_t" => Type::u16(),
            "uint32_t" => Type::u32(),
            "uint64_t" => Type::u64(),
            "float" => Type::f32(),
            "double" => Type::f64(),
            "char*" | "const char*" => Type::ConstStr,
            s if s.starts_with("const ") && s.ends_with("*") => {
                Type::Ptr(Box::new(Type::Const(Box::new(Type::Void))))
            }
            s if s.ends_with("*") => {
                Type::Ptr(Box::new(Type::Void))
            }
            _ => Type::i32(),  
        }
    }
}
