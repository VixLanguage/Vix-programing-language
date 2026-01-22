use crate::import::*;

impl Codegen {
    pub fn ensure_string_concat(&mut self) {
        if self.ir.functions.contains("vix_string_concat") {
            return;
        }
        
        let string_type = self.type_registry.generate_slices(&Type::char8(), &self.arch);
        
        let helper = format!(r#"
    static {ST} vix_string_concat({ST} s1, {ST} s2) {{
        {ST} res;
        res.len = s1.len + s2.len;
        res.ptr = (char*)malloc(res.len + 1);
        if (res.ptr) {{
            memcpy(res.ptr, s1.ptr, s1.len);
            memcpy(res.ptr + s1.len, s2.ptr, s2.len);
            res.ptr[res.len] = '\0';
        }}
        return res;
    }}
    "#, ST = string_type);

        self.ir.functions.push_str(&helper);
    }

    pub fn ensure_repeat_helper(&mut self) {
        if self.ir.functions.contains("vix_repeat") {
            return;
        }
        
        let string_type = self.type_registry.generate_slices(&Type::char8(), &self.arch);
        
        let helper = format!(r#"
    static {ST} vix_repeat({ST} s, int64_t count) {{
        {ST} res;
        if (count <= 0) {{ res.ptr = ""; res.len = 0; return res; }}
        res.len = s.len * count;
        res.ptr = (char*)malloc(res.len + 1);
        if (res.ptr) {{
            for (int64_t i = 0; i < count; i++) {{
                memcpy(res.ptr + (i * s.len), s.ptr, s.len);
            }}
            res.ptr[res.len] = '\0';
        }}
        return res;
    }}
    "#, ST = string_type);

        self.ir.functions.push_str(&helper);
    }

    pub fn ensure_to_string_helpers(&mut self) {
        if self.ir.functions.contains("vix_int_to_str") {
            return;
        }
        
        let string_type = self.type_registry.generate_slices(&Type::char8(), &self.arch);
        
        let helper = format!(r#"
static {ST} vix_int_to_str(int64_t value) {{
    static char buf[32];
    int len = snprintf(buf, sizeof(buf), "%lld", (long long)value);
    return ({ST}){{ .ptr = buf, .len = (size_t)len }};
}}

static {ST} vix_float_to_str(double value) {{
    static char buf[64];
    int len = snprintf(buf, sizeof(buf), "%g", value);
    return ({ST}){{ .ptr = buf, .len = (size_t)len }};
}}

static {ST} vix_bool_to_str(bool value) {{
    return value ? ({ST}){{ .ptr = "true", .len = 4 }} : ({ST}){{ .ptr = "false", .len = 5 }};
}}
"#, ST = string_type);

        self.ir.functions.push_str(&helper);
    }

    pub fn ensure_malloc_helpers(&mut self) {
        if self.ir.functions.contains("vix_malloc") {
            return;
        }
        
        let helper = r#"
static inline void* vix_malloc(size_t size) {
    if (size == 0) return NULL;
    void* ptr = malloc(size);
    if (!ptr) {
        fprintf(stderr, "allocation failed: tried to allocate %zu bytes\n", size);
        abort();
    }
    return ptr;
}

static inline void* vix_realloc(void* old_ptr, size_t new_size) {
    if (new_size == 0) {
        free(old_ptr);
        return NULL;
    }
    void* ptr = realloc(old_ptr, new_size);
    if (!ptr) {
        fprintf(stderr, "reallocation failed: tried to reallocate to %zu bytes\n", new_size);
        abort();
    }
    return ptr;
}

static inline void* vix_calloc(size_t count, size_t size) {
    if (count == 0 || size == 0) return NULL;
    void* ptr = calloc(count, size);
    if (!ptr) {
        fprintf(stderr, "allocation failed: tried to allocate %zu * %zu bytes\n", count, size);
        abort();
    }
    return ptr;
}
"#;

        self.ir.functions.push_str(helper);
    }

    pub fn ensure_std_str(&mut self) {
        if self.ir.type_definitions.contains("String") {
            return;
        }
        
        let typedef = r#"
typedef struct {
    char* ptr;
    size_t len;
    size_t capacity;
} String;
"#;
        
        self.ir.type_definitions.push_str(typedef);
        self.ir.type_definitions.push('\n');
        
         
        let helpers = r#"
static inline String String_new(void) {
    String s;
    s.ptr = NULL;
    s.len = 0;
    s.capacity = 0;
    return s;
}

static inline String String_with_capacity(size_t capacity) {
    String s;
    if (capacity == 0) {
        s.ptr = NULL;
        s.len = 0;
        s.capacity = 0;
    } else {
        s.ptr = (char*)vix_malloc(capacity);
        s.len = 0;
        s.capacity = capacity;
    }
    return s;
}

static inline String String_from(const char* cstr) {
    size_t len = strlen(cstr);
    String s;
    s.len = len;
    s.capacity = len + 1;
    s.ptr = (char*)vix_malloc(s.capacity);
    memcpy(s.ptr, cstr, len);
    s.ptr[len] = '\0';
    return s;
}

static inline String String_from_str(Slice_char source) {
    String s;
    s.len = source.len;
    s.capacity = source.len + 1;
    s.ptr = (char*)vix_malloc(s.capacity);
    memcpy(s.ptr, source.ptr, source.len);
    s.ptr[source.len] = '\0';
    return s;
}

static inline void String_drop(String* s) {
    if (s->ptr != NULL) {
        free(s->ptr);
    }
    s->ptr = NULL;
    s->len = 0;
    s->capacity = 0;
}

static inline void String_push(String* s, char c) {
    if (s->len + 2 > s->capacity) {
        size_t new_cap = s->capacity == 0 ? 8 : s->capacity * 2;
        s->ptr = (char*)vix_realloc(s->ptr, new_cap);
        s->capacity = new_cap;
    }
    s->ptr[s->len] = c;
    s->len++;
    s->ptr[s->len] = '\0';
}

static inline void String_push_str(String* dest, Slice_char src) {
    if (src.len == 0) return;
    
    size_t required = dest->len + src.len + 1;
    if (required > dest->capacity) {
        size_t new_cap = dest->capacity == 0 ? 8 : dest->capacity;
        while (new_cap < required) {
            new_cap *= 2;
        }
        dest->ptr = (char*)vix_realloc(dest->ptr, new_cap);
        dest->capacity = new_cap;
    }
    
    memcpy(dest->ptr + dest->len, src.ptr, src.len);
    dest->len += src.len;
    dest->ptr[dest->len] = '\0';
}

static inline Slice_char String_as_str(const String* s) {
    Slice_char result;
    result.ptr = s->ptr;
    result.len = s->len;
    return result;
}
"#;
        
        self.ir.functions.push_str(helpers);
    }
    
    fn codegen_panic(&mut self, message: &str, location: Option<&str>) -> String {
        let mut code = String::new();
        
        let _loc = location.unwrap_or("unknown");
        code.push_str(&format!("fprintf(stderr, \"panic: {}\\n\");\n", message.replace("\"", "\\\"")));
        code.push_str("abort();\n");

        code
    }

    pub fn try_auto_convert(&self, from: &Type, to: &Type) -> bool {
        match (from, to) {
            (Type::StdStr, Type::Str { .. }) => true,
            (Type::Str { .. }, Type::StdStr) => false,
            _ if self.types_compatible(from, to) => true,
            
            _ => false,
        }
    }


    pub fn auto_convert(
        &mut self,
        var: String,
        from_ty: &Type,
        to_ty: &Type,
        body: &mut String
    ) -> Result<String, ()> {
        if self.types_compatible(from_ty, to_ty) {
            return Ok(var);
        }
        
        match (from_ty, to_ty) {
             
            (Type::StdStr, Type::Str { .. }) => {
                let tmp = self.fresh_var();
                body.push_str(&format!("Slice_char {} = String_as_str(&{});\n", tmp, var));
                Ok(tmp)
            }
            
            _ => {
                Err(())
            }
        }
    }

     
     
    pub fn ensure_array_extend(&mut self, elem_type: &Type) {
        let elem_c_type = elem_type.to_c_type(&self.arch, &mut self.type_registry);
        let array_type = Type::Array { 
            element: Box::new(elem_type.clone()), 
            size: None 
        }.to_c_type(&self.arch, &mut self.type_registry);
        
        let func_name = format!("vix_array_extend_{}", elem_type.name().replace("::", "_"));
        
        if self.ir.functions.contains(&func_name) {
            return;
        }
        
         
        let helper = format!(r#"
static inline {ARRAY_TYPE} vix_array_extend_{SUFFIX}({ARRAY_TYPE} dest, {ARRAY_TYPE} src) {{
    size_t new_len = dest.len + src.len;
    {ARRAY_TYPE} result;
    result.len = new_len;
    
     
    result.ptr = ({ELEM_TYPE} *)malloc(new_len * sizeof({ELEM_TYPE}));
    if (!result.ptr && new_len > 0) {{
        result.len = 0;
        return result;
    }}
    
     
    if (dest.len > 0) {{
        memcpy(result.ptr, dest.ptr, dest.len * sizeof({ELEM_TYPE}));
    }}
    if (src.len > 0) {{
        memcpy(result.ptr + dest.len, src.ptr, src.len * sizeof({ELEM_TYPE}));
    }}
    
     
    
    return result;
}}
"#,
            ARRAY_TYPE = array_type,
            SUFFIX = elem_type.name().replace("::", "_"),
            ELEM_TYPE = elem_c_type
        );
        
        self.ir.functions.push_str(&helper);
    }

    pub fn codegen_extend(
        &mut self,
        dest_var: &str,
        src_var: &str,
        elem_type: &Type,
        body: &mut String
    ) {
        self.ensure_array_extend(elem_type);
        
        let func_name = format!("vix_array_extend_{}", elem_type.name().replace("::", "_"));
        
        body.push_str(&format!(
            "{} = {}({}, {});\n",
            dest_var, func_name, dest_var, src_var
        ));
    }
}
