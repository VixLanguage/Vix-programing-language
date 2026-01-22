use crate::import::*;

impl Codegen {
    pub fn ensure_string_typedef(&mut self) {
         
        if !self.ir.type_definitions.contains("Slice_char") {
            let char_c_type = Type::char8().to_c_type(&self.arch, &mut self.type_registry);
            let typedef = format!(
                "typedef struct {{\n    {}* ptr;\n    size_t len;\n}} Slice_char;\n",
                char_c_type
            );
            self.ir.type_definitions.push_str(&typedef);
            self.ir.type_definitions.push('\n');
        }
    }


    pub fn codegen_string(&mut self, s: &str, body: &mut String) -> (String, Type) {
        self.ensure_string_typedef();
        
         
        let ty = Type::Str { len_type: Box::new(Type::i64()) };
        let c_type = ty.to_c_type(&self.arch, &mut self.type_registry);
        
        let tmp = self.fresh_var();
        let escaped = s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        
         
        body.push_str(&format!("{} {} = {{ .ptr = \"{}\", .len = {} }};\n", 
            c_type, tmp, escaped, s.len()));
        
        (tmp, ty)
    }

    pub fn codegen_number(&mut self, n: i64, body: &mut String) -> (String, Type) {
        let tmp = self.fresh_var();
        if n >= i32::MIN as i64 && n <= i32::MAX as i64 {
            body.push_str(&format!("int32_t {} = {};\n", tmp, n));
            (tmp, Type::i32())
        } else {
            body.push_str(&format!("int64_t {} = {}LL;\n", tmp, n));
            (tmp, Type::i64())
        }
    }

    pub fn codegen_float(&mut self, f: f32, body: &mut String) -> (String, Type) {
        let tmp = self.fresh_var();
        body.push_str(&format!("float {} = {};\n", tmp, f));
        (tmp, Type::f32())
    }

    pub fn codegen_bool(&mut self, b: bool, body: &mut String) -> (String, Type) {
        let tmp = self.fresh_var();
        body.push_str(&format!("bool {} = {};\n", tmp, if b { "true" } else { "false" }));
        (tmp, Type::Bool)
    }

    pub fn codegen_char(&mut self, c: i32, body: &mut String) -> (String, Type) {
        let tmp = self.fresh_var();
        body.push_str(&format!("char {} = {};\n", tmp, c));
        (tmp, Type::char8())
    }

    pub fn codegen_hex_number(&mut self, n: i32, body: &mut String) -> (String, Type) {
        let tmp = self.fresh_var();
        body.push_str(&format!("int32_t {} = 0x{:x};\n", tmp, n));
        (tmp, Type::i32())
    }

    pub fn codegen_binary_number(&mut self, n: i32, body: &mut String) -> (String, Type) {
        let tmp = self.fresh_var();
        body.push_str(&format!("int32_t {} = 0b{:b};\n", tmp, n));
        (tmp, Type::i32())
    }

    pub fn codegen_octal_number(&mut self, n: i32, body: &mut String) -> (String, Type) {
        let tmp = self.fresh_var();
        body.push_str(&format!("int32_t {} = 0{:o};\n", tmp, n));
        (tmp, Type::i32())
    }
}