use crate::import::*;

impl Codegen {
    pub fn ensure_unified_push(&mut self) {
        if self.ir.functions.contains("vix_push") {
            return;
        }
        
        let helper = r#"
#define vix_push(arr, elem) _Generic((arr), \
    Slice_int8: vix_push_impl(arr, &(elem), sizeof(int8_t)), \
    Slice_int16: vix_push_impl(arr, &(elem), sizeof(int16_t)), \
    Slice_int32: vix_push_impl(arr, &(elem), sizeof(int32_t)), \
    Slice_int64: vix_push_impl(arr, &(elem), sizeof(int64_t)), \
    Slice_uint8: vix_push_impl(arr, &(elem), sizeof(uint8_t)), \
    Slice_uint16: vix_push_impl(arr, &(elem), sizeof(uint16_t)), \
    Slice_uint32: vix_push_impl(arr, &(elem), sizeof(uint32_t)), \
    Slice_uint64: vix_push_impl(arr, &(elem), sizeof(uint64_t)), \
    Slice_float: vix_push_impl(arr, &(elem), sizeof(float)), \
    Slice_double: vix_push_impl(arr, &(elem), sizeof(double)), \
    Slice_char: vix_push_impl(arr, &(elem), sizeof(char)), \
    default: vix_push_impl(arr, &(elem), sizeof(elem)) \
)

static inline void* vix_push_impl(void* arr_ptr, const void* elem_ptr, size_t elem_size) {
    typedef struct {
        void* ptr;
        size_t len;
    } GenericSlice;

    GenericSlice* arr = (GenericSlice*)arr_ptr;
    size_t new_len = arr->len + 1;

    void* new_ptr = vix_malloc(new_len * elem_size);

    if (arr->len > 0) {
        memcpy(new_ptr, arr->ptr, arr->len * elem_size);
    }

    memcpy((char*)new_ptr + (arr->len * elem_size), elem_ptr, elem_size);

    arr->ptr = new_ptr;
    arr->len = new_len;
    
    return arr_ptr;
}
"#;
        
        self.ir.functions.push_str(helper);
    }

    pub fn ensure_unified_extend(&mut self) {
        if self.ir.functions.contains("vix_extend") {
            return;
        }
        
        let helper = r#"
#define vix_extend(dest, src) _Generic((dest), \
    Slice_int8: vix_extend_impl(&(dest), &(src), sizeof(int8_t)), \
    Slice_int16: vix_extend_impl(&(dest), &(src), sizeof(int16_t)), \
    Slice_int32: vix_extend_impl(&(dest), &(src), sizeof(int32_t)), \
    Slice_int64: vix_extend_impl(&(dest), &(src), sizeof(int64_t)), \
    Slice_uint8: vix_extend_impl(&(dest), &(src), sizeof(uint8_t)), \
    Slice_uint16: vix_extend_impl(&(dest), &(src), sizeof(uint16_t)), \
    Slice_uint32: vix_extend_impl(&(dest), &(src), sizeof(uint32_t)), \
    Slice_uint64: vix_extend_impl(&(dest), &(src), sizeof(uint64_t)), \
    Slice_float: vix_extend_impl(&(dest), &(src), sizeof(float)), \
    Slice_double: vix_extend_impl(&(dest), &(src), sizeof(double)), \
    Slice_char: vix_extend_impl(&(dest), &(src), sizeof(char)), \
    default: vix_extend_impl(&(dest), &(src), sizeof(*(dest).ptr)) \
)

static inline void vix_extend_impl(void* dest_ptr, const void* src_ptr, size_t elem_size) {
    typedef struct {
        void* ptr;
        size_t len;
    } GenericSlice;
    
    GenericSlice* dest = (GenericSlice*)dest_ptr;
    const GenericSlice* src = (const GenericSlice*)src_ptr;

    if (src->len == 0) return;

    size_t new_len = dest->len + src->len;

    void* new_ptr = vix_malloc(new_len * elem_size);

    if (dest->len > 0) {
        memcpy(new_ptr, dest->ptr, dest->len * elem_size);
    }

    memcpy((char*)new_ptr + (dest->len * elem_size), src->ptr, src->len * elem_size);

    dest->ptr = new_ptr;
    dest->len = new_len;
}
"#;
        
        self.ir.functions.push_str(helper);
    }
    
    pub fn ensure_zero_alloc_string_ops(&mut self) {
        if self.ir.functions.contains("vix_str_concat_view") {
            return;
        }
        
        let helper = r#"
static Slice_char vix_str_concat_view(Slice_char s1, Slice_char s2) {
    static __thread char buffer[8192];
    static __thread size_t offset = 0;
    
    size_t total = s1.len + s2.len;

    if (offset + total >= sizeof(buffer)) {
        offset = 0;
    }

    memcpy(buffer + offset, s1.ptr, s1.len);
    memcpy(buffer + offset + s1.len, s2.ptr, s2.len);

    Slice_char result;
    result.ptr = buffer + offset;
    result.len = total;

    offset += total;

    return result;
}

static inline void vix_str_append_inplace(Slice_char* dest, Slice_char src) {

    static __thread char extend_buf[8192];
    static __thread size_t extend_offset = 0;
    
    size_t total = dest->len + src.len;
    
    if (extend_offset + total >= sizeof(extend_buf)) {
        extend_offset = 0;
    }

    memcpy(extend_buf + extend_offset, dest->ptr, dest->len);
    memcpy(extend_buf + extend_offset + dest->len, src.ptr, src.len);
    
    dest->ptr = extend_buf + extend_offset;
    dest->len = total;
    
    extend_offset += total;
}

typedef struct {
    char* base;
    size_t offset;
    size_t capacity;
} Arena;

static Arena global_arena = {0};

static inline void vix_arena_init(size_t capacity) {
    if (!global_arena.base) {
        global_arena.base = (char*)vix_malloc(capacity);
        global_arena.capacity = capacity;
        global_arena.offset = 0;
    }
}

static inline char* vix_arena_alloc(size_t size) {
    if (global_arena.offset + size > global_arena.capacity) {
        global_arena.offset = 0;
    }
    
    char* ptr = global_arena.base + global_arena.offset;
    global_arena.offset += size;
    return ptr;
}

static Slice_char vix_str_concat_arena(Slice_char s1, Slice_char s2) {
    size_t total = s1.len + s2.len;
    char* ptr = vix_arena_alloc(total);
    
    memcpy(ptr, s1.ptr, s1.len);
    memcpy(ptr + s1.len, s2.ptr, s2.len);
    
    Slice_char result;
    result.ptr = ptr;
    result.len = total;
    return result;
}
"#;
        
        self.ir.functions.push_str(helper);
    }

    pub fn codegen_push_unified(&mut self, arr: &str, elem: &str, body: &mut String) {
        self.ensure_unified_push();
        body.push_str(&format!("vix_push({}, {});\n", arr, elem));
    }
    
    pub fn codegen_extend_unified(&mut self, dest: &str, src: &str, body: &mut String) {
        self.ensure_unified_extend();
        body.push_str(&format!("vix_extend({}, {});\n", dest, src));
    }
    
    pub fn codegen_str_concat_zero_alloc(&mut self, s1: &str, s2: &str, body: &mut String) -> String {
        self.ensure_zero_alloc_string_ops();
        
        body.push_str("vix_arena_init(1048576);\n");
        
        let tmp = self.fresh_var();
        body.push_str(&format!("Slice_char {} = vix_str_concat_arena({}, {});\n", tmp, s1, s2));
        tmp
    }
    
    pub fn codegen_str_append_zero_alloc(&mut self, dest: &str, src: &str, body: &mut String) {
        self.ensure_zero_alloc_string_ops();
        body.push_str(&format!("vix_str_append_inplace(&{}, {});\n", dest, src));
    }
}