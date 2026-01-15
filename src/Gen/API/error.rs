#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub primary_location: SourceLocation,
    pub secondary_locations: Vec<(SourceLocation, String)>,
    pub help_message: Option<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: String,
    pub message: String,
    pub context: ErrorContext,
}

impl Diagnostic {
    pub fn error(code: &str, message: &str, context: ErrorContext) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            code: code.to_string(),
            message: message.to_string(),
            context,
        }
    }

    pub fn warning(code: &str, message: &str, context: ErrorContext) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            code: code.to_string(),
            message: message.to_string(),
            context,
        }
    }

    pub fn display(&self, source_code: &str) -> String {
        let mut output = String::new();
        
        match self.level {
            DiagnosticLevel::Error => output.push_str(&format!("[Error]: {} Error:\n", self.code)),
            DiagnosticLevel::Warning => output.push_str(&format!("[Warning]: {} Warning:\n", self.code)),
        }
        
        let primary = &self.context.primary_location;
        output.push_str(&format!("   | {}:{}\n", primary.file, primary.line));
        output.push_str("   |\n");
        
        let lines: Vec<&str> = source_code.lines().collect();
        
        let start_line = primary.line.saturating_sub(3);
        for i in start_line..primary.line.saturating_sub(1) {
            if i < lines.len() {
                output.push_str(&format!("{:2} | {}\n", i + 1, lines[i]));
            }
        }
        
        if primary.line > 0 && primary.line <= lines.len() {
            let line_content = lines[primary.line - 1];
            output.push_str(&format!("{:2} |> {}\n", primary.line, line_content));
            
            output.push_str(&format!("   | {}{}^ {}\n", 
                " ".repeat(primary.column.saturating_sub(1)),
                "^".repeat(primary.length.saturating_sub(1).max(0)),
                self.message
            ));
        }
        
        for (sec_loc, sec_msg) in &self.context.secondary_locations {
            output.push_str("   |\n");
            if sec_loc.line > 0 && sec_loc.line <= lines.len() {
                let line_content = lines[sec_loc.line - 1];
                output.push_str(&format!("{:2} |> {}\n", sec_loc.line, line_content));
                output.push_str(&format!("   | {}{} {}\n", 
                    " ".repeat(sec_loc.column.saturating_sub(1)),
                    "^".repeat(sec_loc.length.max(1)),
                    sec_msg
                ));
            }
        }
        
        if let Some(help) = &self.context.help_message {
            output.push_str("   |\n");
            output.push_str(&format!("   {}\n", "-".repeat(72)));
            output.push_str("   |-> help:\n");
            for line in help.lines() {
                output.push_str(&format!("    {}\n", line));
            }
        }
        
        if !self.context.suggestions.is_empty() {
            output.push_str("   |\n");
            output.push_str("   |-> suggestions:\n");
            for suggestion in &self.context.suggestions {
                output.push_str(&format!("    - {}\n", suggestion));
            }
        }
        
        output
    }
}

pub struct DiagnosticHandler {
    diagnostics: Vec<Diagnostic>,
    source_code: String,
    max_errors: usize,
    pub error_count: usize,
    pub warning_count: usize,
}

impl DiagnosticHandler {
    pub fn new(source_code: String) -> Self {
        Self {
            diagnostics: Vec::new(),
            source_code,
            max_errors: 100,
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn error(&mut self, code: &str, message: &str, context: ErrorContext) {
        let diagnostic = Diagnostic::error(code, message, context);
        eprintln!("{}", diagnostic.display(&self.source_code));
        self.diagnostics.push(diagnostic);
        self.error_count += 1;
        
        if self.error_count >= self.max_errors {
            eprintln!("[Warning]: Reached {} errors, but continuing to generate code...", self.error_count);
        }
    }

    pub fn warning(&mut self, code: &str, message: &str, context: ErrorContext) {
        let diagnostic = Diagnostic::warning(code, message, context);
        eprintln!("{}", diagnostic.display(&self.source_code));
        self.diagnostics.push(diagnostic);
        self.warning_count += 1;
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn print_summary(&self) {
        if self.error_count > 0 || self.warning_count > 0 {
            eprintln!("\nCompilation finished with {} error(s) and {} warning(s).", 
                self.error_count, self.warning_count);
        }
    }
    
    pub fn should_continue(&self) -> bool {
        self.error_count < self.max_errors
    }
}

pub fn type_mismatch_error(
    expected_ty: &str,
    got_ty: &str,
    location: SourceLocation,
    source_location: SourceLocation,
) -> ErrorContext {
    ErrorContext {
        primary_location: location.clone(),
        secondary_locations: vec![
            (source_location, format!("sending {} to {}", got_ty, expected_ty))
        ],
        help_message: Some(format!(
            "Expected type '{}', but got '{}'.\n    Consider using Option<{}> or converting the type explicitly.",
            expected_ty, got_ty, expected_ty
        )),
        suggestions: vec![
            format!("Change parameter type to Option<{}>", expected_ty),
            format!("Convert {} to {} explicitly", got_ty, expected_ty),
        ],
    }
}

pub fn return_type_mismatch_error(
    expected_ty: &str,
    got_ty: &str,
    location: SourceLocation,
    function_location: SourceLocation,
) -> ErrorContext {
    ErrorContext {
        primary_location: location.clone(),
        secondary_locations: vec![
            (function_location, format!("function expects to return {}", expected_ty))
        ],
        help_message: Some(format!(
            "Returning {} to {}.\n    The function signature specifies return type '{}', but found '{}'.",
            got_ty, expected_ty, expected_ty, got_ty
        )),
        suggestions: vec![
            format!("Change return type to Option<{}>", expected_ty),
            format!("Wrap return value in Some(...) if using Option"),
            format!("Convert {} to {} before returning", got_ty, expected_ty),
        ],
    }
}

pub fn void_operation_error(
    operation: &str,
    location: SourceLocation,
) -> ErrorContext {
    ErrorContext {
        primary_location: location,
        secondary_locations: vec![],
        help_message: Some(format!("Cannot perform operation '{}' on void type.\n    Void type represents the absence of a value.",operation)),
        suggestions: vec![
            "Remove this operation".to_string(),
            "Check if the expression should return a value".to_string(),
        ],
    }
}

pub fn void_variable_error(
    var_name: &str,
    location: SourceLocation,
) -> ErrorContext {
    ErrorContext {
        primary_location: location,
        secondary_locations: vec![],
        help_message: Some(format!(
            "Variable '{}' cannot have void type.\n    Variables must have concrete types that can hold values.",
            var_name
        )),
        suggestions: vec![
            format!("Change '{}' to a concrete type", var_name),
            "Remove this variable declaration".to_string(),
        ],
    }
}

pub fn void_array_error(
    location: SourceLocation,
) -> ErrorContext {
    ErrorContext {
        primary_location: location,
        secondary_locations: vec![],
        help_message: Some(
            "Arrays cannot contain void elements.\n    Array elements must be concrete types.".to_string()
        ),
        suggestions: vec![
            "Change array element type to a concrete type".to_string(),
            "Use a different data structure".to_string(),
        ],
    }
}

pub fn dereference_void_error(
    location: SourceLocation,
) -> ErrorContext {
    ErrorContext {
        primary_location: location,
        secondary_locations: vec![],
        help_message: Some(
            "Cannot dereference void pointer without explicit cast.\n    Cast to a concrete type before dereferencing.".to_string()
        ),
        suggestions: vec![
            "Cast to appropriate pointer type: (int32_t*)ptr".to_string(),
            "Use a typed pointer instead of void*".to_string(),
        ],
    }
}



pub fn borrow_conflict_error(var: &str, loc: SourceLocation, conflict: SourceLocation) -> ErrorContext {
    ErrorContext {
        primary_location: loc,
        secondary_locations: vec![
            (conflict, format!("first borrow of '{}' occurs here", var))
        ],
        help_message: Some(format!(
            "Cannot borrow '{}' because it is already borrowed.\n    \
            Rust's borrow checker ensures memory safety by preventing multiple mutable borrows or \
            simultaneous mutable and immutable borrows of the same data.",
            var
        )),
        suggestions: vec![
            "Drop the first borrow before creating a new one".to_string(),
            "Use different variables for separate borrows".to_string(),
            "Consider restructuring to avoid overlapping borrows".to_string(),
        ],
    }
}

pub fn undefined_variable_error(name: &str, loc: SourceLocation) -> ErrorContext {
    ErrorContext {
        primary_location: loc,
        secondary_locations: vec![],
        help_message: Some(format!(
            "Variable '{}' is not defined in the current scope.\n    \
            Variables must be declared before use.",
            name
        )),
        suggestions: vec![
            format!("Declare variable '{}' before using it", name),
            "Check variable name spelling".to_string(),
            "Ensure the variable is in scope".to_string(),
        ],
    }
}

pub fn undefined_function_error(name: &str, arg_count: usize, loc: SourceLocation) -> ErrorContext {
    ErrorContext {
        primary_location: loc,
        secondary_locations: vec![],
        help_message: Some(format!(
            "Function '{}' with {} argument(s) is not defined.\n    \
            Functions must be declared before they are called, or imported from external modules.",
            name, arg_count
        )),
        suggestions: vec![
            format!("Define function '{}' before calling it", name),
            format!("Import '{}' from a module or external library", name),
            "Check function name spelling".to_string(),
            "Verify the function signature matches the call".to_string(),
        ],
    }
}
