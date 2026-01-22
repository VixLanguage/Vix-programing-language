use crate::import::*;

impl Codegen {
    pub fn codegen_std_call(&mut self, func: &str, args: &[Expr], body: &mut String, loc: SourceLocation) -> Result<(String, Type), ()> {
        self.ensure_runtime_functions();
        eprintln!("[DEBUG] codegen_std_call: func_name={}", func);
        match func {
            "array" | "Array" => self.codegen_array_init(args, body),
            "vector" | "Vector" => self.codegen_vector_init(args, body),
            "buffer" | "Buffer" => self.codegen_buffer_init(args, body),
            "hashMap" => self.codegen_hashmap_init(args, body),
            "range" => self.codegen_range(args, body),
            "random" => self.codegen_random(args, body),
            _ => self.codegen_call_expr_default(func, args, body, loc),
        }
    }

     
     
    fn ensure_runtime_functions(&mut self) {
        if !self.ir.functions.contains("void* x_array_init") {
            self.ir.functions.push_str(r#"
void* x_array_init() {
    struct { void* ptr; size_t len; size_t capacity; }* arr = malloc(sizeof(*arr));
    arr->ptr = malloc(16 * sizeof(void*));
    arr->len = 0;
    arr->capacity = 16;
    return arr;
}

void* x_hashmap_init() { return malloc(1024); }
void* x_vector_init() { return malloc(1024); }
void* x_buffer_init() { return malloc(1024); }

int x_random(int max) {
    if (max <= 0) return 0;
    static int seeded = 0;
    if (!seeded) { srand((unsigned)time(NULL)); seeded = 1; }
    return rand() % max;
}
"#);
        }
    }

    fn codegen_array_init(&mut self, _args: &[Expr], body: &mut String) -> Result<(String, Type), ()> {
        let tmp = self.fresh_var();
        body.push_str(&format!("void* {} = x_array_init();\n", tmp));
        Ok((tmp, Type::Array { element: Box::new(Type::Any), size: None }))
    }

    fn codegen_range(&mut self, args: &[Expr], body: &mut String) -> Result<(String, Type), ()> {
        let (val, _) = self.codegen_expr(&args[0], body).check_error();
        Ok((val, Type::i32()))
    }

    fn codegen_random(&mut self, args: &[Expr], body: &mut String) -> Result<(String, Type), ()> {
        let max_var = if !args.is_empty() {
            let (v, _) = self.codegen_expr(&args[0], body).check_error();
            v
        } else {
            "100".to_string()
        };
        let tmp = self.fresh_var();
        body.push_str(&format!("int32_t {} = x_random({});\n", tmp, max_var));
        Ok((tmp, Type::i32()))
    }

    fn codegen_hashmap_init(&mut self, _args: &[Expr], body: &mut String) -> Result<(String, Type), ()> {
        let tmp = self.fresh_var();
        body.push_str(&format!("void* {} = x_hashmap_init();\n", tmp));
        Ok((tmp, Type::Any))
    }

    fn codegen_vector_init(&mut self, _args: &[Expr], body: &mut String) -> Result<(String, Type), ()> {
        let tmp = self.fresh_var();
        body.push_str(&format!("void* {} = x_vector_init();\n", tmp));
        Ok((tmp, Type::Any))
    }

    fn codegen_buffer_init(&mut self, _args: &[Expr], body: &mut String) -> Result<(String, Type), ()> {
        let tmp = self.fresh_var();
        body.push_str(&format!("void* {} = x_buffer_init();\n", tmp));
        Ok((tmp, Type::Any))
    }

    fn codegen_call_expr_default(&mut self, func: &str, args: &[Expr], body: &mut String, _loc: SourceLocation) -> Result<(String, Type), ()> {
        let mut arg_vars = Vec::new();

        println!("[DEBUG] codegen_call_expr_default: func={}, user_functions keys={:?}", func, self.user_functions.keys().collect::<Vec<_>>());

        let (param_types, ret_ty) = if let Some(ext_info) = self.extern_functions.get(func) {
            (Some(ext_info.params.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>()), ext_info.return_type.clone())
        } else if let Some((params, ret_ty)) = self.user_functions.get(func) {
            (Some(params.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>()), ret_ty.clone())
        } else {
            (None, Type::i32())
        };

        for (i, arg) in args.iter().enumerate() {
            let (mut var, ty) = self.codegen_expr(arg, body).check_error();

            if let Some(params) = &param_types
                && let Some(param_ty) = params.get(i)
                    && matches!(param_ty, Type::ConstStr) && matches!(ty, Type::Str { .. }) {
                        var = format!("{}.ptr", var);
                    }

            arg_vars.push(var);
        }

        let tmp = self.fresh_var();
        let args_str = arg_vars.join(", ");
        let c_ret_type = ret_ty.to_c_type(&self.arch, &mut self.type_registry);
        body.push_str(&format!("{} {} = {}({});\n", c_ret_type, tmp, func, args_str));
        Ok((tmp, ret_ty))
    }
}
