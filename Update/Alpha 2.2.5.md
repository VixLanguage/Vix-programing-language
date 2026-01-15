### Update Alpha: 2.2.5v
Overall of Alpha 2.2.5v included **mut** keyword for struct fileds and variables and everything now is immutable by default. Implemtion for struct/enum/func are added
- **mut** keyword included and everything is immutable by default
    ```rust
        mut mutable_filed: int = 100
        immutable_filed: int = 500

        print(immutable_filed)
        mutable_filed += 10
        print(mutable_filed)
    ```
- **import** keyword included for implemting public **enum/struct/func**:
    ```ruby
        import ExampleStruct from "src/main.x"

        examplestruct = ExampleStruct(filed1: 0, filed2: "")
    ```
- **Bugs** In this update no bugs are found. 
    