use crate::import::*;

impl Codegen {
    pub fn ensure_string_typedef(&mut self) {
        let typedef = "typedef struct { char* ptr; int64_t len; } String;\n\n";
        let helpers = r#"
static String vix_string_concat(String s1, String s2) {
    String res;
    res.len = s1.len + s2.len;
    res.ptr = (char*)malloc(res.len + 1);
    if (res.ptr) {
        memcpy(res.ptr, s1.ptr, s1.len);
        memcpy(res.ptr + s1.len, s2.ptr, s2.len);
        res.ptr[res.len] = '\0';
    }
    return res;
}

static String vix_int_to_str(int64_t val) {
    char buf[32];
    int len = snprintf(buf, sizeof(buf), "%lld", (long long)val);
    String res;
    res.len = len;
    res.ptr = (char*)malloc(len + 1);
    if (res.ptr) {
        memcpy(res.ptr, buf, len);
        res.ptr[len] = '\0';
    }
    return res;
}

static String vix_string_from_const(const char* s) {
    String res;
    res.ptr = (char*)s;
    res.len = s ? strlen(s) : 0;
    return res;
}

"#;
        if !self.ir.forward_decls.contains("typedef struct { char* ptr; int64_t len; } String;") {
            self.ir.forward_decls.insert_str(0, typedef);
            self.ir.forward_decls.push_str(helpers);
        }
    }

    pub fn codegen_string(&mut self, s: &str, body: &mut String) -> (String, Type) {
        self.ensure_string_typedef();
        
        let tmp = self.fresh_var();
        let escaped = s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        
        body.push_str(&format!("String {} = {{ .ptr = \"{}\", .len = {} }};\n", tmp, escaped, s.len()
        ));
        
        (tmp, Type::Str { len_type: Box::new(Type::i64()) })
    }


    pub fn codegen_number(&mut self, n: i32, body: &mut String) -> (String, Type) {
        let tmp = self.fresh_var();
        body.push_str(&format!("int32_t {} = {};\n", tmp, n));
        (tmp, Type::i32())
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