use crate::import::*;

impl IR {
    pub fn new() -> Self {
        IR {
            headers: String::new(),
            forward_decls: String::new(),
            functions: String::new(),
        }
    }
    
    pub fn finalize(self) -> String {
        let mut output = String::new();

        output.push_str("#include <stdio.h>\n");
        output.push_str("#include <stdlib.h>\n");
        output.push_str("#include <stdint.h>\n");
        output.push_str("#include <stdbool.h>\n");
        output.push_str("#include <string.h>\n");
        output.push_str("#include <time.h>");   
        output.push_str("\n");
        
        if !self.headers.is_empty() {
            output.push_str(&self.headers);
            output.push_str("\n");
        }
        
        if !self.forward_decls.is_empty() {
            output.push_str(&self.forward_decls);
            output.push_str("\n");
        }
        
        if !self.functions.is_empty() {
            output.push_str(&self.functions);
        }

        output
    }

    pub fn add_forward_decl(&mut self, decl: String) {
        self.forward_decls.push_str(&decl);
        self.forward_decls.push_str("\n");
    }
    
    pub fn add_function(&mut self, func: String) {
        self.functions.push_str(&func);
        self.functions.push_str("\n\n");
    }

    pub fn add_RuntimeFunction(&mut self, func_name: &str, func_def: String) {
        if !self.functions.contains(&func_name) {
            self.functions.push_str(&func_def);
            self.functions.push_str("\n\n");
        }
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
            forward_decls: self.forward_decls.clone(),
            functions: self.functions.clone(),
        }
    }
}