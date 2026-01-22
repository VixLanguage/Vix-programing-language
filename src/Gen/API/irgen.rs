use crate::import::*;
use std::collections::HashSet;

impl IR {
    pub fn new() -> Self {
        IR {
            headers: String::new(),
            type_definitions: String::new(),
            forward_decls: String::new(),
            function_decls: String::new(),
            helper_functions: String::new(),
            functions: String::new(),
            added_typedefs: HashSet::new(),
            added_function_decls: HashSet::new(),
        }
    }
    
    pub fn finalize(self) -> String {
        let mut output = String::new();

         
        output.push_str("#include <stdio.h>\n");
        output.push_str("#include <stdlib.h>\n");
        output.push_str("#include <stdint.h>\n");
        output.push_str("#include <stdbool.h>\n");
        output.push_str("#include <string.h>\n");
        output.push_str("#include <time.h>\n\n");
        
         
        if !self.headers.is_empty() {
            output.push_str(&self.headers);
            output.push('\n');
        }
        
         
        if !self.type_definitions.is_empty() {
            output.push_str(&self.type_definitions);
            output.push('\n');
        }
        
         
        if !self.forward_decls.is_empty() {
            output.push_str(&self.forward_decls);
            output.push('\n');
        }

         
        if !self.function_decls.is_empty() {
            output.push_str(&self.function_decls);
            output.push('\n');
        }

         
        if !self.helper_functions.is_empty() {
            output.push_str(&self.helper_functions);
            output.push('\n');
        }

         
        if !self.functions.is_empty() {
            output.push_str(&self.functions);
            output.push('\n');
        }

         
        output.push_str("int main() {\n");
        output.push_str("    vix_main();\n");
        output.push_str("    return 0;\n");
        output.push_str("}\n");

        output
    }

    pub fn add_forward_decl(&mut self, decl: String) {
        if !self.forward_decls.contains(&decl) {
            self.forward_decls.push_str(&decl);
            if !decl.ends_with('\n') {
                self.forward_decls.push('\n');
            }
        }
    }
    
    pub fn add_type_definition(&mut self, typedef: String) {
         
         
        let type_key = if let Some(name_start) = typedef.rfind("} ") {
            if let Some(name_end) = typedef[name_start + 2..].find(';') {
                typedef[name_start + 2..name_start + 2 + name_end].trim().to_string()
            } else {
                typedef.clone()
            }
        } else if typedef.starts_with("typedef ") {
             
            typedef.trim_end_matches(';')
                .split_whitespace()
                .last()
                .unwrap_or(&typedef)
                .to_string()
        } else {
            typedef.clone()
        };

         
        if self.added_typedefs.insert(type_key) {
            self.type_definitions.push_str(&typedef);
            if !typedef.ends_with('\n') {
                self.type_definitions.push('\n');
            }
        }
    }
    
    pub fn add_function_decl(&mut self, decl: String) {
        let normalized = decl.split_whitespace().collect::<Vec<_>>().join(" ");
        
        if self.added_function_decls.insert(normalized.clone()) {
            self.function_decls.push_str(&decl);
            if !decl.ends_with('\n') {
                self.function_decls.push('\n');
            }
        }
    }

    pub fn add_function(&mut self, func: String) {
        self.functions.push_str(&func);
        if !func.ends_with('\n') {
            self.functions.push('\n');
        }
    }

    pub fn add_helper_function(&mut self, func_name: &str, func_def: String) {
        if !self.helper_functions.contains(func_name) {
            self.helper_functions.push_str(&func_def);
            if !func_def.ends_with('\n') {
                self.helper_functions.push('\n');
            }
        }
    }

    pub fn add_RuntimeFunction(&mut self, func_name: &str, func_def: String) {
        self.add_helper_function(func_name, func_def);
    }
}

impl Default for IR {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for IR {
    fn clone(&self) -> Self {
        IR {
            headers: self.headers.clone(),
            type_definitions: self.type_definitions.clone(),
            forward_decls: self.forward_decls.clone(),
            function_decls: self.function_decls.clone(),
            helper_functions: self.helper_functions.clone(),
            functions: self.functions.clone(),
            added_typedefs: self.added_typedefs.clone(),
            added_function_decls: self.added_function_decls.clone(),
        }
    } 
}