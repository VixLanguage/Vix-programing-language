### Update Alpha: 0.2.0v
---
Update Alpha 0.2.0v included:
- Bug fixing
    - ```const str``` was generating wrong C IR. Before it was generating same IR as normal **str**. Problem fixed by making it generate:
        ```c
            const char* var_example = t2.ptr;
        ```
        - ```str``` keept generating same IR:
        ```c
            String t2 = { .ptr = "Hello world", .len = 10 };
        ```
    - ```codegen_var``` was generating "_error_" for better error handling that has been removed and added a error handling in IR generating:  and problem of "unknow var" is fixed now **codegen_var** should detect all vars without any problems
        ```rust
            self.diagnostics.error(
                "UndefinedVariable",
                &format!("Variable '{}' has not been declared in this scope.", name),
                ErrorContext {
                    primary_location: loc.clone(),
                    secondary_locations: vec![],
                    help_message: Some(format!(
                        "Cannot find variable '{}' in the current scope.", 
                        name
                    )),
                    suggestions: vec![
                        format!("Declare '{}' before using it", name),
                        format!("Use 'let {} = ...' to declare and initialize", name),
                        "Check for typos in the variable name".to_string(),
                    ],
                }
            );
        ```
    - Print was generating a wrong output like: "÷"\1÷" beacuse it's using "%s" for string problem has been fixed by making print use "%s" only for numbers.
    ```python
        print("Hello, world!")
    ```
    - Generate correct C IR code:
    ```c
        String t1 = { .ptr = "\n", .len = 10 };
        int32_t t10 = printf("%s\n", t1.ptr);
    ```
