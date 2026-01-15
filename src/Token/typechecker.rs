use crate::import::*;
use std::collections::HashMap;

pub struct TypeChecker {
    handler: DiagnosticHandler,
    functions: HashMap<String, FunctionSignature>,
    builtin_functions: HashMap<String, FunctionSignature>,
    imported_functions: HashMap<String, FunctionSignature>,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    variables: Vec<HashMap<String, Type>>,
    current_function_return_type: Option<Type>,
    borrow_tracker: BorrowTracker,
    current_line: usize,
}

#[derive(Debug, Clone)]
struct FunctionSignature {
    params: Vec<(String, Type, ParamModifier)>,
    return_type: Type,
    location: SourceLocation,
    is_builtin: bool,
}

#[derive(Debug, Clone)]
struct StructInfo {
    fields: HashMap<String, (Type, bool, bool)>,
    location: SourceLocation,
}

#[derive(Debug, Clone)]
struct EnumInfo {
    variants: HashMap<String, EnumVariantInfo>,
    location: SourceLocation,
}

#[derive(Debug, Clone)]
enum EnumVariantInfo {
    Simple,
    Tuple(Vec<Type>),
    Struct(HashMap<String, Type>),
}

struct BorrowTracker {
    mutable_borrows: HashMap<String, SourceLocation>,
    immutable_borrows: HashMap<String, Vec<SourceLocation>>,
}

impl BorrowTracker {
    fn new() -> Self {
        Self {
            mutable_borrows: HashMap::new(),
            immutable_borrows: HashMap::new(),
        }
    }

    fn add_mutable_borrow(&mut self, var: String, loc: SourceLocation) -> Option<SourceLocation> {
        if let Some(existing) = self.mutable_borrows.get(&var) {
            return Some(existing.clone());
        }
        if let Some(borrows) = self.immutable_borrows.get(&var) {
            if !borrows.is_empty() {
                return Some(borrows[0].clone());
            }
        }
        self.mutable_borrows.insert(var, loc);
        None
    }

    fn add_immutable_borrow(&mut self, var: String, loc: SourceLocation) -> Option<SourceLocation> {
        if let Some(existing) = self.mutable_borrows.get(&var) {
            return Some(existing.clone());
        }
        self.immutable_borrows.entry(var).or_insert_with(Vec::new).push(loc);
        None
    }

    fn clear(&mut self) {
        self.mutable_borrows.clear();
        self.immutable_borrows.clear();
    }
}

impl TypeChecker {
    pub fn new(source_code: String) -> Self {
        let mut checker = Self {
            handler: DiagnosticHandler::new(source_code),
            functions: HashMap::new(),
            builtin_functions: HashMap::new(),
            imported_functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            variables: vec![HashMap::new()],
            current_function_return_type: None,
            borrow_tracker: BorrowTracker::new(),
            current_line: 1,
        };
        
        checker.register_builtin_functions();
        checker
    }

    fn register_builtin_functions(&mut self) {
        let builtin_loc = self.make_location(0, 0, 0);
        
         
        let builtins = vec![
            ("print", vec![], Type::Void),
            ("println", vec![], Type::Void),
            ("panic", vec![], Type::Void),
            ("array", vec![], Type::Void),
            ("slots", vec![], Type::Void),
            ("lists", vec![], Type::Void),
            ("char", vec![], Type::char32()),
            ("len", vec![("arr".to_string(), Type::Any, ParamModifier::Immutable)], Type::i32()),
            ("push", vec![("arr".to_string(), Type::Any, ParamModifier::Immutable), ("item".to_string(), Type::Any, ParamModifier::Immutable)], Type::Void),
            ("pop", vec![("arr".to_string(), Type::Any, ParamModifier::Immutable)], Type::Any),
            ("assert", vec![("condition".to_string(), Type::Bool, ParamModifier::Immutable)], Type::Void),
        ];
        
        for (name, params, ret_type) in builtins {
            self.builtin_functions.insert(
                name.to_string(),
                FunctionSignature {
                    params,
                    return_type: ret_type,
                    location: builtin_loc.clone(),
                    is_builtin: true,
                },
            );
        }
    }

    fn enter_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.variables.pop();
        self.borrow_tracker.clear();
    }

    fn add_variable(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name, ty);
        }
    }

    fn get_variable_type(&self, name: &str) -> Option<Type> {
        for scope in self.variables.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    fn make_location(&self, line: usize, col: usize, len: usize) -> SourceLocation {
        SourceLocation {
            file: "input".to_string(),
            line,
            column: col,
            length: len,
        }
    }
    
    fn set_line(&mut self, line: usize) {
        self.current_line = line;
    }

    pub fn check_program(
        &mut self,
        program: &Program,
        structs: &[StructDef],
        enums: &[EnumDef],
        externs: &[ExternDecl],
        impls: &[ImplBlock],
        imports: &[ImportDecl],
    ) -> bool {
        self.register_structs(structs);
        self.register_enums(enums);
        self.register_externs(externs);
        self.register_imports(imports);
        self.register_functions(&program.functions);
        self.register_impl_methods(impls);

        for func in &program.functions {
            self.check_function(func);
        }

        for impl_block in impls {
            self.check_impl_block(impl_block);
        }

        self.handler.print_summary();
        !self.handler.has_errors()
    }

    fn register_imports(&mut self, imports: &[ImportDecl]) {
        for import in imports {
            match import {
                ImportDecl::LibraryImport { name } => {
                     
                    let loc = self.make_location(1, 1, name.len());
                    match name.as_str() {
                        "hashMap" => {
                            self.imported_functions.insert(
                                "hashMap".to_string(),
                                FunctionSignature {
                                    params: vec![],
                                    return_type: Type::Any,
                                    location: loc,
                                    is_builtin: false,
                                },
                            );
                        }
                        _ => {
                             
                            self.imported_functions.insert(
                                name.clone(),
                                FunctionSignature {
                                    params: vec![],
                                    return_type: Type::Any,
                                    location: loc,
                                    is_builtin: false,
                                },
                            );
                        }
                    }
                }
                ImportDecl::FileImport { name, .. } => {
                    let loc = self.make_location(1, 1, name.len());
                    self.imported_functions.insert(
                        name.clone(),
                        FunctionSignature {
                            params: vec![],
                            return_type: Type::Any,
                            location: loc,
                            is_builtin: false,
                        },
                    );
                }
            }
        }
    }

    fn register_structs(&mut self, structs: &[StructDef]) {
        for s in structs {
            let mut fields = HashMap::new();
            for field in &s.fields {
                fields.insert(
                    field.name.clone(),
                    (field.ty.clone(), field.is_public, field.is_mutable),
                );
            }
            self.structs.insert(
                s.name.clone(),
                StructInfo {
                    fields,
                    location: self.make_location(1, 1, s.name.len()),
                },
            );
        }
    }

    fn register_enums(&mut self, enums: &[EnumDef]) {
        for e in enums {
            let mut variants = HashMap::new();
            for variant in &e.variants {
                let info = match variant {
                    EnumVariant::Simple(name) => (name.clone(), EnumVariantInfo::Simple),
                    EnumVariant::Tuple(name, types) => {
                        (name.clone(), EnumVariantInfo::Tuple(types.clone()))
                    }
                    EnumVariant::Struct(name, fields) => {
                        let mut field_map = HashMap::new();
                        for field in fields {
                            field_map.insert(field.name.clone(), field.ty.clone());
                        }
                        (name.clone(), EnumVariantInfo::Struct(field_map))
                    }
                };
                variants.insert(info.0, info.1);
            }
            self.enums.insert(
                e.name.clone(),
                EnumInfo {
                    variants,
                    location: self.make_location(1, 1, e.name.len()),
                },
            );
        }
    }

    fn register_externs(&mut self, externs: &[ExternDecl]) {
        for ext in externs {
            match ext {
                ExternDecl::Single { func, .. } => {
                    self.functions.insert(
                        func.name.clone(),
                        FunctionSignature {
                            params: func.params.iter().map(|(n, t)| (n.clone(), t.clone(), ParamModifier::Immutable)).collect(),
                            return_type: func.return_type.clone(),
                            location: self.make_location(1, 1, func.name.len()),
                            is_builtin: false,
                        },
                    );
                }
                ExternDecl::SingleWithBody { func, .. } => {
                    self.functions.insert(
                        func.name.clone(),
                        FunctionSignature {
                            params: func.params.iter().map(|(n, t)| (n.clone(), t.clone(), ParamModifier::Immutable)).collect(),
                            return_type: func.return_type.clone(),
                            location: self.make_location(1, 1, func.name.len()),
                            is_builtin: false,
                        },
                    );
                }
                ExternDecl::Block { functions, .. } => {
                    for func in functions {
                        self.functions.insert(
                            func.name.clone(),
                            FunctionSignature {
                                params: func.params.iter().map(|(n, t)| (n.clone(), t.clone(), ParamModifier::Immutable)).collect(),
                                return_type: func.return_type.clone(),
                                location: self.make_location(1, 1, func.name.len()),
                                is_builtin: false,
                            },
                        );
                    }
                }
            }
        }
    }

    fn register_functions(&mut self, functions: &[Function]) {
        for func in functions {
            let loc = self.make_location(1, 1, func.name.len());
            
             
            if self.builtin_functions.contains_key(&func.name) {
                self.handler.error(
                    "E0428",
                    &format!("Function '{}' conflicts with built-in function", func.name),
                    ErrorContext {
                        primary_location: loc.clone(),
                        secondary_locations: vec![],
                        help_message: Some(format!(
                            "The function name '{}' is reserved for a built-in function.\n    \
                            Built-in functions cannot be redefined.",
                            func.name
                        )),
                        suggestions: vec![
                            format!("Rename function '{}' to something else", func.name),
                            "Choose a different function name".to_string(),
                        ],
                    },
                );
            }
            
             
            if let Some(imported_sig) = self.imported_functions.get(&func.name) {
                self.handler.error(
                    "E0428",
                    &format!("Function '{}' conflicts with imported function", func.name),
                    ErrorContext {
                        primary_location: loc.clone(),
                        secondary_locations: vec![
                            (imported_sig.location.clone(), "imported here".to_string())
                        ],
                        help_message: Some(format!(
                            "The function name '{}' is already imported from a module.\n    \
                            Cannot define a function with the same name as an import.",
                            func.name
                        )),
                        suggestions: vec![
                            format!("Rename function '{}' to something else", func.name),
                            format!("Remove the import of '{}'", func.name),
                        ],
                    },
                );
            }
            
            self.functions.insert(
                func.name.clone(),
                FunctionSignature {
                    params: func.params.clone(),
                    return_type: func.return_type.clone(),
                    location: loc,
                    is_builtin: false,
                },
            );
        }
    }

    fn register_impl_methods(&mut self, impls: &[ImplBlock]) {
        for impl_block in impls {
            for method in &impl_block.methods {
                let full_name = format!("{}.{}", impl_block.struct_name, method.name);
                self.functions.insert(
                    full_name,
                    FunctionSignature {
                        params: method.params.clone(),
                        return_type: method.return_type.clone(),
                        location: self.make_location(1, 1, method.name.len()),
                        is_builtin: false,
                    },
                );
            }
        }
    }

    fn check_function(&mut self, func: &Function) {
        self.enter_scope();
        self.current_function_return_type = Some(func.return_type.clone());
        self.set_line(1);

        for (name, ty, modifier) in &func.params {
            if self.is_void_type(ty) {
                self.handler.error(
                    "E0001",
                    &format!("Parameter '{}' cannot have void type", name),
                    void_variable_error(name, self.make_location(1, 1, name.len())),
                );
            }
            
            match modifier {
                ParamModifier::MutableReference => {
                    if let Some(conflict) = self.borrow_tracker.add_mutable_borrow(name.clone(), self.make_location(1, 1, name.len())) {
                        self.handler.error(
                            "E0502",
                            &format!("Cannot borrow '{}' as mutable more than once", name),
                            borrow_conflict_error(name, self.make_location(1, 1, name.len()), conflict),
                        );
                    }
                }
                ParamModifier::Reference => {
                    if let Some(conflict) = self.borrow_tracker.add_immutable_borrow(name.clone(), self.make_location(1, 1, name.len())) {
                        self.handler.error(
                            "E0502",
                            &format!("Cannot borrow '{}' as immutable while mutable borrow exists", name),
                            borrow_conflict_error(name, self.make_location(1, 1, name.len()), conflict),
                        );
                    }
                }
                _ => {}
            }

            self.add_variable(name.clone(), ty.clone());
        }

        for stmt in &func.body {
            self.check_statement(stmt, &func.return_type);
        }

        self.current_function_return_type = None;
        self.exit_scope();
    }

    fn check_impl_block(&mut self, impl_block: &ImplBlock) {
         
        if !self.structs.contains_key(&impl_block.struct_name) {
            self.handler.error(
                "E0412",
                &format!("Cannot find struct '{}' for impl block", impl_block.struct_name),
                ErrorContext {
                    primary_location: self.make_location(1, 1, impl_block.struct_name.len()),
                    secondary_locations: vec![],
                    help_message: Some(format!(
                        "Struct '{}' must be defined before implementing methods for it.",
                        impl_block.struct_name
                    )),
                    suggestions: vec![
                        format!("Define struct '{}' before impl block", impl_block.struct_name),
                    ],
                },
            );
        }

        for method in &impl_block.methods {
             
            if method.name == impl_block.struct_name {
                 
                 
                continue;
            }

            self.enter_scope();
            self.current_function_return_type = Some(method.return_type.clone());

             
            if let Some(self_mod) = &method.self_modifier {
                let self_type = Type::Struct { name: impl_block.struct_name.clone() };
                self.add_variable("self".to_string(), self_type);

                match self_mod {
                    SelfModifier::Mutable => {
                        if let Some(conflict) = self.borrow_tracker.add_mutable_borrow("self".to_string(), self.make_location(1, 1, 4)) {
                            self.handler.error(
                                "E0502",
                                "Cannot borrow 'self' as mutable more than once",
                                borrow_conflict_error("self", self.make_location(1, 1, 4), conflict),
                            );
                        }
                    }
                    SelfModifier::Reference | SelfModifier::Borrow => {
                        if let Some(conflict) = self.borrow_tracker.add_immutable_borrow("self".to_string(), self.make_location(1, 1, 4)) {
                            self.handler.error(
                                "E0502",
                                "Cannot borrow 'self' as immutable while mutable borrow exists",
                                borrow_conflict_error("self", self.make_location(1, 1, 4), conflict),
                            );
                        }
                    }
                    _ => {}
                }
            }

            for (name, ty, modifier) in &method.params {
                if self.is_void_type(ty) {
                    self.handler.error(
                        "E0001",
                        &format!("Parameter '{}' cannot have void type", name),
                        void_variable_error(name, self.make_location(1, 1, name.len())),
                    );
                }

                match modifier {
                    ParamModifier::MutableReference => {
                        if let Some(conflict) = self.borrow_tracker.add_mutable_borrow(name.clone(), self.make_location(1, 1, name.len())) {
                            self.handler.error(
                                "E0502",
                                &format!("Cannot borrow '{}' as mutable more than once", name),
                                borrow_conflict_error(name, self.make_location(1, 1, name.len()), conflict),
                            );
                        }
                    }
                    ParamModifier::Reference => {
                        if let Some(conflict) = self.borrow_tracker.add_immutable_borrow(name.clone(), self.make_location(1, 1, name.len())) {
                            self.handler.error(
                                "E0502",
                                &format!("Cannot borrow '{}' as immutable while mutable borrow exists", name),
                                borrow_conflict_error(name, self.make_location(1, 1, name.len()), conflict),
                            );
                        }
                    }
                    _ => {}
                }

                self.add_variable(name.clone(), ty.clone());
            }

            for stmt in &method.body {
                self.check_statement(stmt, &method.return_type);
            }

            self.current_function_return_type = None;
            self.exit_scope();
        }
    }
    fn check_statement(&mut self, stmt: &Stmt, expected_return_type: &Type) {
         
        
        match stmt {
            Stmt::TypedDeclaration { name, ty, value, is_mutable } => {
                if self.is_void_type(ty) {
                    self.handler.error(
                        "E0001",
                        &format!("Variable '{}' cannot have void type", name),
                        void_variable_error(name, self.make_location(1, 1, name.len())),
                    );
                }

                let value_type = self.infer_expr_type(value);
                if !self.types_compatible(ty, &value_type) {
                    self.handler.error(
                        "E0308",
                        &format!("Type mismatch in variable declaration '{}'", name),
                        type_mismatch_error(
                            &self.type_to_string(ty),
                            &self.type_to_string(&value_type),
                            self.make_location(1, 5, name.len()),
                            self.make_location(1, 5, 1),
                        ),
                    );
                }

                if *is_mutable {
                    if let Some(conflict) = self.borrow_tracker.add_mutable_borrow(name.clone(), self.make_location(1, 1, name.len())) {
                        self.handler.error(
                            "E0502",
                            &format!("Cannot borrow '{}' as mutable more than once", name),
                            borrow_conflict_error(name, self.make_location(1, 1, name.len()), conflict),
                        );
                    }
                }

                self.add_variable(name.clone(), ty.clone());
            }

            Stmt::Assign(name, value) => {
                if let Some(var_type) = self.get_variable_type(name) {
                    let value_type = self.infer_expr_type(value);
                    if !self.types_compatible(&var_type, &value_type) {
                        self.handler.error(
                            "E0308",
                            &format!("Type mismatch in assignment to '{}'", name),
                            type_mismatch_error(
                                &self.type_to_string(&var_type),
                                &self.type_to_string(&value_type),
                                self.make_location(1, 1, name.len()),
                                self.make_location(1, 1, 1),
                            ),
                        );
                    }
                } else {
                    self.handler.error(
                        "E0425",
                        &format!("Cannot find variable '{}' in this scope", name),
                        undefined_variable_error(name, self.make_location(1, 1, name.len())),
                    );
                }
            }

            Stmt::CompoundAssign(name, op, value) => {
                if let Some(var_type) = self.get_variable_type(name) {
                    if self.is_void_type(&var_type) {
                        self.handler.error(
                            "E0277",
                            &format!("Cannot perform operation '{}' on void type", op),
                            void_operation_error(op, self.make_location(1, 1, name.len())),
                        );
                    }

                    let value_type = self.infer_expr_type(value);
                    if !self.types_compatible(&var_type, &value_type) {
                        self.handler.error(
                            "E0308",
                            &format!("Type mismatch in compound assignment to '{}'", name),
                            type_mismatch_error(
                                &self.type_to_string(&var_type),
                                &self.type_to_string(&value_type),
                                self.make_location(1, 1, name.len()),
                                self.make_location(1, 1, 1),
                            ),
                        );
                    }
                } else {
                    self.handler.error(
                        "E0425",
                        &format!("Cannot find variable '{}' in this scope", name),
                        undefined_variable_error(name, self.make_location(1, 1, name.len())),
                    );
                }
            }

            Stmt::Return(expr_opt) => {
                let return_type = if let Some(expr) = expr_opt {
                    self.infer_expr_type(expr)
                } else {
                    Type::Void
                };

                if !self.types_compatible(expected_return_type, &return_type) {
                    self.handler.error(
                        "E0308",
                        "Mismatched return type",
                        return_type_mismatch_error(
                            &self.type_to_string(expected_return_type),
                            &self.type_to_string(&return_type),
                            self.make_location(1, 1, 6),
                            self.make_location(1, 1, 1),
                        ),
                    );
                }
            }

            Stmt::If(cond, then_body, else_body) => {
                let cond_type = self.infer_expr_type(cond);
                if !matches!(cond_type, Type::Bool) && !matches!(cond_type, Type::Any) {
                    self.handler.error(
                        "E0308",
                        "Condition must be boolean",
                        ErrorContext {
                            primary_location: self.make_location(1, 1, 2),
                            secondary_locations: vec![],
                            help_message: Some(format!(
                                "Expected boolean type in condition, found '{}'",
                                self.type_to_string(&cond_type)
                            )),
                            suggestions: vec![
                                "Use a comparison operator (==, !=, <, >, <=, >=)".to_string(),
                                "Convert to boolean explicitly".to_string(),
                            ],
                        },
                    );
                }

                self.enter_scope();
                for stmt in then_body {
                    self.check_statement(stmt, expected_return_type);
                }
                self.exit_scope();

                if let Some(else_stmts) = else_body {
                    self.enter_scope();
                    for stmt in else_stmts {
                        self.check_statement(stmt, expected_return_type);
                    }
                    self.exit_scope();
                }
            }

            Stmt::While(cond, body) => {
                let cond_type = self.infer_expr_type(cond);
                if !matches!(cond_type, Type::Bool) && !matches!(cond_type, Type::Any) {
                    self.handler.error(
                        "E0308",
                        "Loop condition must be boolean",
                        ErrorContext {
                            primary_location: self.make_location(1, 1, 5),
                            secondary_locations: vec![],
                            help_message: Some(format!(
                                "Expected boolean type in loop condition, found '{}'",
                                self.type_to_string(&cond_type)
                            )),
                            suggestions: vec![
                                "Use a comparison operator".to_string(),
                            ],
                        },
                    );
                }

                self.enter_scope();
                for stmt in body {
                    self.check_statement(stmt, expected_return_type);
                }
                self.exit_scope();
            }

            Stmt::For(var_name, iterable, body) => {
                let iter_type = self.infer_expr_type(iterable);
                
                self.enter_scope();
                match &iter_type {
                    Type::Array { element, .. } => {
                        self.add_variable(var_name.clone(), (**element).clone());
                    }
                    _ => {
                        self.handler.error(
                            "E0277",
                            "For loop requires iterable type",
                            ErrorContext {
                                primary_location: self.make_location(1, 1, 3),
                                secondary_locations: vec![],
                                help_message: Some(format!(
                                    "Cannot iterate over type '{}'. Expected array or iterable type.",
                                    self.type_to_string(&iter_type)
                                )),
                                suggestions: vec![
                                    "Use an array type".to_string(),
                                    "Convert to iterable type".to_string(),
                                ],
                            },
                        );
                        self.add_variable(var_name.clone(), Type::Any);
                    }
                }

                for stmt in body {
                    self.check_statement(stmt, expected_return_type);
                }
                self.exit_scope();
            }

            Stmt::Match(expr, cases, default) => {
                let expr_type = self.infer_expr_type(expr);

                for case in cases {
                    let case_type = self.infer_expr_type(&case.value);
                    if !self.types_compatible(&expr_type, &case_type) {
                        self.handler.error(
                            "E0308",
                            "Match arm type mismatch",
                            type_mismatch_error(
                                &self.type_to_string(&expr_type),
                                &self.type_to_string(&case_type),
                                self.make_location(1, 1, 5),
                                self.make_location(1, 1, 1),
                            ),
                        );
                    }

                    self.enter_scope();
                    for stmt in &case.body {
                        self.check_statement(stmt, expected_return_type);
                    }
                    self.exit_scope();
                }

                if let Some(default_body) = default {
                    self.enter_scope();
                    for stmt in default_body {
                        self.check_statement(stmt, expected_return_type);
                    }
                    self.exit_scope();
                }
            }

            Stmt::Call(func_name, args) | Stmt::ModuleCall(_, func_name, args) => {
                 
                if self.builtin_functions.contains_key(func_name) {
                     
                    return;
                }
                
                 
                if self.imported_functions.contains_key(func_name) {
                    return;
                }
                
                 
                if let Some(sig) = self.functions.get(func_name) {
                    if args.len() != sig.params.len() {
                        self.handler.error(
                            "E0061",
                            &format!("Function '{}' expects {} arguments, got {}", func_name, sig.params.len(), args.len()),
                            ErrorContext {
                                primary_location: self.make_location(1, 1, func_name.len()),
                                secondary_locations: vec![
                                    (sig.location.clone(), format!("defined here with {} parameters", sig.params.len()))
                                ],
                                help_message: Some(format!(
                                    "This function requires {} argument(s), but {} were provided.",
                                    sig.params.len(), args.len()
                                )),
                                suggestions: vec![
                                    if args.len() < sig.params.len() {
                                        format!("Add {} more argument(s)", sig.params.len() - args.len())
                                    } else {
                                        format!("Remove {} argument(s)", args.len() - sig.params.len())
                                    }
                                ],
                            },
                        );
                    } else {
                        for (i, arg) in args.iter().enumerate() {
                            if let Some((_, expected_type, _)) = sig.params.get(i) {
                                let arg_type = self.infer_expr_type(arg);
                                if !self.types_compatible(expected_type, &arg_type) {
                                    self.handler.error(
                                        "E0308",
                                        &format!("Type mismatch in argument {} of function '{}'", i + 1, func_name),
                                        type_mismatch_error(
                                            &self.type_to_string(expected_type),
                                            &self.type_to_string(&arg_type),
                                            self.make_location(1, 1, func_name.len()),
                                            sig.location.clone(),
                                        ),
                                    );
                                }
                            }
                        }
                    }
                } else {
                    self.handler.error(
                        "E0425",
                        &format!("Cannot find function '{}' in this scope", func_name),
                        undefined_function_error(func_name, args.len(), self.make_location(1, 1, func_name.len())),
                    );
                }
            }

            Stmt::Expr(expr) => {
                self.infer_expr_type(expr);
            }
            Stmt::Scope(body) | Stmt::Unsafe(body) => {
                self.enter_scope();
                for stmt in body {
                    self.check_statement(stmt, expected_return_type);
                }
                self.exit_scope();
            }

            Stmt::IndexAssign(obj, indices, value) => {
                let obj_type = self.infer_expr_type(obj);
                let value_type = self.infer_expr_type(value);

                match &obj_type {
                    Type::Array { element, .. } => {
                        if !self.types_compatible(element, &value_type) {
                            self.handler.error(
                                "E0308",
                                "Array element type mismatch",
                                type_mismatch_error(
                                    &self.type_to_string(element),
                                    &self.type_to_string(&value_type),
                                    self.make_location(1, 1, 1),
                                    self.make_location(1, 1, 1),
                                ),
                            );
                        }

                        for idx in indices {
                            let idx_type = self.infer_expr_type(idx);
                            if !self.is_integer_type(&idx_type) {
                                self.handler.error(
                                    "E0308",
                                    "Array index must be integer",
                                    ErrorContext {
                                        primary_location: self.make_location(1, 1, 1),
                                        secondary_locations: vec![],
                                        help_message: Some(format!(
                                            "Expected integer type for array index, found '{}'",
                                            self.type_to_string(&idx_type)
                                        )),
                                        suggestions: vec![
                                            "Use an integer expression".to_string(),
                                            "Convert to integer type".to_string(),
                                        ],
                                    },
                                );
                            }
                        }
                    }
                    _ => {
                        self.handler.error(
                            "E0277",
                            "Cannot index non-array type",
                            ErrorContext {
                                primary_location: self.make_location(1, 1, 1),
                                secondary_locations: vec![],
                                help_message: Some(format!(
                                    "Cannot index into type '{}'. Only arrays support indexing.",
                                    self.type_to_string(&obj_type)
                                )),
                                suggestions: vec![
                                    "Use an array type".to_string(),
                                ],
                            },
                        );
                    }
                }
            }

            Stmt::MemberAssign(obj, field, value) => {
                let obj_type = self.infer_expr_type(obj);
                let value_type = self.infer_expr_type(value);

                if let Expr::Var(struct_name) = obj.as_ref() {
                    if let Some(struct_info) = self.structs.get(struct_name) {
                        if let Some((field_type, _, is_mutable)) = struct_info.fields.get(field) {
                            if !is_mutable {
                                self.handler.error(
                                    "E0594",
                                    &format!("Cannot assign to immutable field '{}'", field),
                                    ErrorContext {
                                        primary_location: self.make_location(1, 1, field.len()),
                                        secondary_locations: vec![],
                                        help_message: Some(format!(
                                            "Field '{}' is declared as immutable and cannot be modified.",
                                            field
                                        )),
                                        suggestions: vec![
                                            format!("Mark field '{}' as mutable in struct definition", field),
                                        ],
                                    },
                                );
                            }

                            if !self.types_compatible(field_type, &value_type) {
                                self.handler.error(
                                    "E0308",
                                    &format!("Type mismatch in field '{}' assignment", field),
                                    type_mismatch_error(
                                        &self.type_to_string(field_type),
                                        &self.type_to_string(&value_type),
                                        self.make_location(1, 1, field.len()),
                                        self.make_location(1, 1, 1),
                                    ),
                                );
                            }
                        } else {
                            self.handler.error(
                                "E0609",
                                &format!("No field '{}' on type '{}'", field, struct_name),
                                ErrorContext {
                                    primary_location: self.make_location(1, 1, field.len()),
                                    secondary_locations: vec![],
                                    help_message: Some(format!(
                                        "Struct '{}' does not have a field named '{}'.",
                                        struct_name, field
                                    )),
                                    suggestions: vec![
                                        "Check field name spelling".to_string(),
                                        format!("Available fields: {}", 
                                            struct_info.fields.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
                                        ),
                                    ],
                                },
                            );
                        }
                    }
                } else {
                    match &obj_type {
                        Type::Struct { name } => {
                            if let Some(struct_info) = self.structs.get(name) {
                                if let Some((field_type, _, _)) = struct_info.fields.get(field) {
                                    if !self.types_compatible(field_type, &value_type) {
                                        self.handler.error(
                                            "E0308",
                                            &format!("Type mismatch in field '{}' assignment", field),
                                            type_mismatch_error(
                                                &self.type_to_string(field_type),
                                                &self.type_to_string(&value_type),
                                                self.make_location(1, 1, field.len()),
                                                self.make_location(1, 1, 1),
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            _ => {}
        }
    }

    fn infer_expr_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Number(_) | Expr::HexNumber(_) | Expr::BinaryNumber(_) | Expr::OctalNumber(_) => Type::i32(),
            Expr::Float(_) => Type::f32(),
            Expr::String(_) => Type::Str { len_type: Box::new(Type::i64()) },
            Expr::Bool(_) => Type::Bool,
            Expr::None => Type::Option { inner: Box::new(Type::Any) },
            Expr::Var(name) => {
                if name == "self" {
                    Type::SelfType
                } else {
                    self.get_variable_type(name).unwrap_or(Type::Any)
                }
            }
             
            Expr::CallNamed(struct_name, _) => {
                if self.structs.contains_key(struct_name) {
                    Type::Struct { name: struct_name.clone() }
                } else {
                    Type::Any
                }
            }
            Expr::BinOp(op, left, right) => {
                let left_type = self.infer_expr_type(left);
                let right_type = self.infer_expr_type(right);

                if self.is_void_type(&left_type) || self.is_void_type(&right_type) {
                    Type::Void
                } else {
                    match op.as_str() {
                        "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||" => Type::Bool,
                        _ => {
                            if matches!(left_type, Type::Float { bits: 32 }) {
                                left_type
                            } else if matches!(right_type, Type::Float { bits: 32 }) {
                                right_type
                            } else {
                                left_type
                            }
                        }
                    }
                }
            }
            Expr::UnOp(op, operand) => {
                let operand_type = self.infer_expr_type(operand);
                if self.is_void_type(&operand_type) {
                    Type::Void
                } else {
                    match op.as_str() {
                        "!" => Type::Bool,
                        "&" => Type::Ref(Box::new(operand_type)),
                        _ => operand_type,
                    }
                }
            }
            Expr::Call(func_name, _) => {
                 
                if self.structs.contains_key(func_name) {
                    return Type::Struct { name: func_name.clone() };
                }
                
                if let Some(sig) = self.functions.get(func_name) {
                    sig.return_type.clone()
                } else if let Some(sig) = self.builtin_functions.get(func_name) {
                    sig.return_type.clone()
                } else if let Some(sig) = self.imported_functions.get(func_name) {
                    sig.return_type.clone()
                } else {
                    Type::Any
                }
            }
            Expr::Array(elements) => {
                if elements.is_empty() {
                    Type::Array {
                        element: Box::new(Type::Any),
                        size: Some(0),
                    }
                } else {
                    let first_type = self.infer_expr_type(&elements[0]);
                    Type::Array {
                        element: Box::new(first_type),
                        size: Some(elements.len()),
                    }
                }
            }
            Expr::Tuple(elements) => {
                let fields = elements.iter().map(|e| self.infer_expr_type(e)).collect();
                Type::Tuple { fields }
            }
            Expr::Index(obj, _) => {
                let obj_type = self.infer_expr_type(obj);
                match obj_type {
                    Type::Array { element, .. } => (*element).clone(),
                    _ => Type::Any,
                }
            }
            Expr::MemberAccess(obj, field) => {
                let obj_type = self.infer_expr_type(obj);
                match obj_type {
                    Type::Struct { name } => {
                        if let Some(struct_info) = self.structs.get(&name) {
                            if let Some((field_type, _, _)) = struct_info.fields.get(field) {
                                field_type.clone()
                            } else {
                                Type::Any
                            }
                        } else {
                            Type::Any
                        }
                    }
                    _ => Type::Any,
                }
            }
            Expr::Some(inner) => Type::Option { inner: Box::new(self.infer_expr_type(inner)) },
            Expr::ResultOk(inner) => Type::Result {
                ok: Box::new(self.infer_expr_type(inner)),
                err: Box::new(Type::Any),
            },
            Expr::ResultErr(inner) => Type::Result {
                ok: Box::new(Type::Any),
                err: Box::new(self.infer_expr_type(inner)),
            },
            Expr::Not(inner) => {
                let _ = self.infer_expr_type(inner);
                Type::Bool
            }
            _ => Type::Any,
        }
    }

    fn is_void_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Void)
    }

    fn is_integer_type(&self, ty: &Type) -> bool {
        matches!(ty, 
            Type::Int { signed: true, bits: 8 | 16 | 32 | 64 | 128 } |
            Type::Int { signed: false, bits: 8 | 16 | 32 | 64 | 128 }
        )
    }

    fn types_compatible(&self, expected: &Type, got: &Type) -> bool {
        if matches!(expected, Type::Any) || matches!(got, Type::Any) {
            return true;
        }

        match (expected, got) {
            (Type::Int { bits: b1, signed: s1 }, Type::Int { bits: b2, signed: s2 }) => b1 == b2 && s1 == s2,
            (Type::Float { bits: b1 }, Type::Float { bits: b2 }) => b1 == b2,
            (Type::Bool, Type::Bool) => true,
            (Type::Void, Type::Void) => true,
            (Type::ConstStr, Type::Str { .. }) => true,
            (Type::Str { .. }, Type::ConstStr) => true,
            (Type::Option { inner: exp }, Type::Option { inner: got }) => self.types_compatible(exp, got),
            
            (Type::Result { ok: exp_ok, err: exp_err }, Type::Result { ok: got_ok, err: got_err }) => {
                self.types_compatible(exp_ok, got_ok) && self.types_compatible(exp_err, got_err)
            }

            (Type::Array { element: exp_elem, .. }, Type::Array { element: got_elem, .. }) => {
                self.types_compatible(exp_elem, got_elem)
            }

            (Type::Tuple { fields: exp_fields }, Type::Tuple { fields: got_fields }) => {
                exp_fields.len() == got_fields.len() &&
                exp_fields.iter().zip(got_fields.iter()).all(|(e, g)| self.types_compatible(e, g))
            }

            (Type::Struct { name: n1 }, Type::Struct { name: n2 }) => n1 == n2,

            (Type::Ref(exp), Type::Ref(got)) | 
            (Type::MutRef(exp), Type::MutRef(got)) |
            (Type::RawPtr(exp), Type::RawPtr(got)) => self.types_compatible(exp, got),

            _ => false,
        }
    }

    fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::Int { bits: 8, signed: true } => "int8".to_string(),
            Type::Int { bits: 16, signed: true } => "int16".to_string(),
            Type::Int { bits: 32, signed: true } => "int32".to_string(),
            Type::Int { bits: 64, signed: true } => "int64".to_string(),
            Type::Int { bits: 8, signed: false } => "uint8".to_string(),
            Type::Int { bits: 16, signed: false } => "uint16".to_string(),
            Type::Int { bits: 32, signed: false } => "uint32".to_string(),
            Type::Int { bits: 64, signed: false } => "uint64".to_string(),
            Type::Int { bits, signed: true } => format!("int{}", bits),
            Type::Int { bits, signed: false } => format!("uint{}", bits),
            Type::Float { bits: 32 } => "float32".to_string(),
            Type::Float { bits: 64 } => "float64".to_string(),
            Type::Float { bits } => format!("float{}", bits),
            Type::Bool => "bool".to_string(),
            Type::Void => "void".to_string(),
            Type::Str { .. } => "str".to_string(),
            Type::ConstStr => "const str".to_string(),
            Type::Any => "any".to_string(),
            Type::SelfType => "Self".to_string(),
            Type::Trait => "trait".to_string(),
            Type::Option { inner } => format!("Option<{}>", self.type_to_string(inner)),
            Type::Result { ok, err } => format!("Result<{}, {}>", self.type_to_string(ok), self.type_to_string(err)),
            Type::Array { element, size } => {
                if let Some(s) = size {
                    format!("[{}; {}]", self.type_to_string(element), s)
                } else {
                    format!("[{}]", self.type_to_string(element))
                }
            }
            Type::Tuple { fields } => {
                let field_strs: Vec<_> = fields.iter().map(|f| self.type_to_string(f)).collect();
                format!("({})", field_strs.join(", "))
            }
            Type::Struct { name } => name.clone(),
            Type::Ref(inner) => format!("&{}", self.type_to_string(inner)),
            Type::MutRef(inner) => format!("&mut {}", self.type_to_string(inner)),
            Type::RawPtr(inner) => format!("^{}", self.type_to_string(inner)),
            Type::Owned(inner) => format!("~{}", self.type_to_string(inner)),
            Type::Ptr(inner) => format!("*{}", self.type_to_string(inner)),
            Type::Union { variants } => {
                let var_strs: Vec<_> = variants.iter().map(|v| self.type_to_string(v)).collect();
                var_strs.join(" | ")
            }
            Type::Intersection { types } => {
                let type_strs: Vec<_> = types.iter().map(|t| self.type_to_string(t)).collect();
                type_strs.join(" & ")
            }
            Type::Const(inner) => format!("const {}", self.type_to_string(inner)),
            Type::MultiArray { element, dimensions } => {
                format!("{}[{}]", self.type_to_string(element), 
                    dimensions.iter().map(|d| d.to_string()).collect::<Vec<_>>().join("]["))
            }
            Type::TripleDot => "...".to_string(),
            Type::Variadic => "...".to_string(),
            Type::Char { bits: 8, .. } => "char".to_string(),
            Type::Char { bits: 32, .. } => "char32".to_string(),
            Type::Char { bits, .. } => format!("char{}", bits),
            Type::StrSlice { .. } => "str".to_string(),
            Type::FnPtr { params, return_type } => {
                let param_strs: Vec<_> = params.iter().map(|p| self.type_to_string(p)).collect();
                format!("fn({}) -> {}", param_strs.join(", "), self.type_to_string(return_type))
            }
        }
    }
}
