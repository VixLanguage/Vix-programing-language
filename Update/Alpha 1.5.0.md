### Update Alpha: 0.1.5v
---
Update Alpha 0.1.5v included:
- Bug fixing:
    - **Error handler** was showing false errors:
        - Example script:
        ```ruby
            struct Math:
                name = str
                age = int
            end

            impl Math(input_name: str, input_age: int):
                Math(name = input_name, age = input_age)
                
                func add(mut self, input_name: str, input_age: int): void
                    self.name += input_name
                    self.age += input_age
                end

                func return_data(self): result[(str | int), str]
                    return Ok((self.name, self.age))
                end
            end

            func main(): int
                math: Math = Math("Hello", 10)
            end
        ```
    - The script is returning errors about "undefined variable", "name/age" are not being detected by **Error handler** as a **struct filed** and that what make the error. Even the C code are generated correct and contain no errors about "name/age" however 
    ```c
    typedef struct Math {
        struct { char* ptr; int64_t len; } name;
        int32_t age;
    } Math;

    Math Math_new(String param_input_name, int32_t param_input_age) {
        Math instance;
        instance.name = param_input_name;
        instance.age = param_input_age;
        return instance;
    }
    ```
    - Error handler was returning a error about "name/age" are "undefined variable" this bug as ben fixed and now error handler never trigger **struct filed** as **variables**.
    ```
    [Error]: UndefinedVariable Error:
   | src\main.x:0
   |
   |
   ------------------------------------------------------------------------
   |-> help:
    Cannot perform compound assignment on undefined variable 'name'.
   |
   |-> suggestions:
    - Declare 'name' before using compound assignment
    ```

- Allow "impl Param":
    - Vix officaly allow "impl Name(param)" like in class. Example:
        ```ruby
            impl Example(input1: int32, input2: int32)
                Example(filed1 = input1, filed2 = input2)
            end
        ```
    - Example usage:
    ```ruby
        func main()
            example: Example = Math("Hello world", 67)
        end
    ```
    - with new update ".new()" are allowed:
    ```ruby
        func main()
            example: Example = Math.new("Hello world", 67)
        end
    ```
- Turple was generating wrong IR:
    ```c
        Tuple_int32_int32 t13 = { ._0 = t11, ._1 = t12 };
    ```
    - That has been fixed too to Right IR:
    ```c
    ```

- Ok function is not generating. In codegen_call_expr, the map the string "ok" to codegen_result_ok. However, when the compiler parser generates the AST for Ok(...), it creates an Expr::ResultOk, not an Expr::Call("ok", ...). This what coust the error

- New Updates into the documation:
    - More informations about the language
    - More examples to learn more about the language syntax and developing with
