# Vix-programing-languaage
Vix programing language good language in making a fast applications, low level or high level easily and fast. Not like python or rust this memory safe + fast + simple to understand and fast to develop with. The language still on the alpha version and still more coming soon. Check our discord server for more help: https://discord.gg/CAemjRc4ya

# Vix programing-language
Welcome to Vix programing language. This the alpha version is not useable yet but this version only for fixing bugs and make everything works.

### Vix Programing language
---
## **Story**
Vix programing language invented by "Vix developing team" with leader ship of **[MrBatata](https://discord.com/1251680506429313076)**. Project started in earlys of 2024 by MrBatata with no knowlage or exp in compilers. He started by making things like Rex **[Vm](https://en.wikipedia.org/wiki/Virtual_machine)** then start working on Luma programing language is knowen today as **[Vix](https://)**. 

Vix went throw so many dramas and challanges but with leading of  **[MrBatata](https://discord.com/1251680506429313076)** and help of Co owner **[kat](https://discord.com/1253355817973973024)** The team continued and went throw all the drama. After time new member joined the team **[Masashi](https://discord.com/763299991850057729)** an exp developer on compilers and VMs and university studient. Helped for building and creating the language compiler and reviewing the compiler at **pre alpha** version. Who now is the manager of **[Vix](https://)**.

Vix took 2 years to develop the **Pre alpha** with over 6 different **[Compilers](https:///information/compilers)**. Even the team went throw so many dramas and challanages at end we got into the end and now Vix is officaly released to the public. Time you reading this document Vix is just on alpha version still getting daily updates from fixing bugs and adding fetruers and much more coming soon.

- After all that Vix is invented and released to the public at 1/1/2026. The project who made the language that balance between diffecuty and performance.

- Vix is low and high level programing language with C Performance. Allow running C/C++ libraries easily. For more information about how to run C/C++ libraries in Vix please check the help document [LibraryInstalling](https:///help/library_installing)

---

### **Memory safety**
---

Vix is a beginner friendly language uses "end" instend of "{}" for keeping everything simple for beginners. It's a good language for fast developing and deep it's not like languages like python slow or made only for simple protyping. Vix is highly optimizated compiles to LLVM using [Clang](https://en.wikipedia.org/wiki/Clang) Compiler. That what make it fast and highly secure. With Simple syntax make the language so easy to develop anything with it no need for changing into another language beacuse Vix can have C/C++ libraries can use for any low or high level tasks and good for game developing beacuse it allow C++ libraries just with so much simpler syntax.

Vix uses only "Struct/enum/impl" for keeping everything simple and no confusing with over [40 build in functions](https://vim.dev/information/Buildinfunctions) and giant Std library + allow to use C/C++ std libraries too for anything. Vix contain simple [type keywords](https://vim.dev/information/type_keyword) to remember like **std**/**int**/**float**/**maybe**/**char**/**any**. That what keep it simple to understand and use all thos keyword

Vix in a memory safety language. Use [immutable variables](https://en.wikipedia.org/wiki/Immutable_object) by defualt. To make an [mutable variables](https://en.wikipedia.org/wiki/Mutable_object). That what make it less likely to make errors on it. Vix don't have ```null``` it contain ```None``` that what make it less danger to manage memory with and using functions like [option]() and [result]() and [Some]() and much more... in build in functions make it so simple to manager memory safely

- example of **Option/Result/Ok/Some** and much more:
```rust
    // No crash and everything handled safely:
    i: Option[int32] = None
    // Some function use when you add "Option" type and not a None. Example:
    x: Option[int32] = Some(30)
    // Result function can be used if you want to expect the correct result:
    x: Result[int32, str] = [100, "Hello world"]
```
- Thoes functions can be used in a function too example:
```ruby
    func example(): Result[Option[int32], str]
        return Ok((100, "Hello world")) 
    end
```
- If you need to build something require Unsafe and no memory safety like OSes or anything you can use unsafe blocks or functions 
```rust
    unsafe: 
        // in Unsafe blocks there is no overflow checks. When you do this. It never will stop beacuse there is nothing limit it.
        mut i = 0
        
        forever do
            i += 10
        end
        
        // You can fix this problem by adding checks. Limit everything but can have an overhead.
        i: int32 = 0
           ^^^^^

        // ownership/borrow don't work in unsafe blocks so when you do anything danger like: This will give will crash at run time beacuse there is no compilion time error about it.
        mut i = 10
        mut x = ptr* uint8

        free(x)
        print(x)
    end

    unsafe func example():
        // Unsafe functions are same deal
    end
```
- Unsafe blocks are recommaded for high performance with less over head possible but that on coust of safety.
- All variables on unsafe blocks or unsafe functions use "raw pointers" not like in safe blocks use alloc make it fast but unsafe.
- Vix compiler use SSA for better performance
- For more information about Build in functions in Vix please visit our website: https:///information or https:///help/memory_safety

### **Performance**:
Vix is highly optimized programing language compiles to C then to LLVM using [Clang compiler](https://en.wikipedia.org/wiki/Clang). Vix Compiler is a middle end. Used for memory checks and generate C IR code.
```py
    Source Code -> Vix Compiler -> C -> Clang Compiler
```
What makes Xeon so fast and good for high performance applications that it's optimized and memory safe same time use Clang -O3 optimzations, inlineand much more for generate highly optimized IR code and binary.

Vix allow importing C/C++ libraries easily that makes you use highly fast languags libraries on Vix it self. And all libraries on Vix highly optimized and easily to install and implement for more help about implemeting a library on Vix we'd recommand you to visit: https:///help/library_installion and can check our marketplace contain so many free libraries: https:///marketplace.

Vix use on safe blocks alloc for safety but it coust of speed that make it slower then languages like C/C++. In unsafe blocks you use raw pointers allow you to get the raw performance of Vix but with unsafe memory managemnt coust. You can use libraries to allow Vix to get into even more speed but that will be a thrid party modifications yes can give you more optimization and speed but that not build in inside the compiler

Vix use in safe blocks for dynamic arraies or params ```<T>``` type with int32 limit/str as usize len. in Unsafe blocks everything be different beacuse Vix use Uinon for more performance but it coust some customizaitons and safety.

- You can make your own library in C/C++/Vix it self easily or even LLVM is avaiable.
- Vix allow you to make custom compilers and put them into marketplace.
- Every library are fully optimized and checked so it give you the best performance possible.
- For more informations about Vix banchmarks please check our website https://informations/banachmarks

### Library Implemention & Custom Compilers and toolchains:
Vix is a good language in Library implementing libraries from C/C++ or LLVM or any other place. Using commands like ```'package install "library name"'``` will install the library automaticly from our website: https://marketplace/library_name from the same website you can install the library manully. Libraries could be open or close source with so much security agiest the malwares make libraries so safe to use and install. 

- ```Warning:``` Even Vix have smart and powerful anti virus and security agiest malwares from libraries/compilers/tool chains/frame works etc... but there is chance that a library can go throw the security. If you found a library/custom compiler or anything contain a malware we'd recommand you to report it on https://report. Thank you for understanding.
"
for install a library version you can use  ```'package install "name_of_library" --version'```. For checking what library version you have or all libraries you have. You can use command: check libraries: ```"package check --library"``` check library and version: ```package check -library_name -version```.To delete an installed library or anything use ```package uninstall "library_name"``` To implement a library you can use ```import library_name``` if you wanna implement something inside a library using: ```import something from library```.

- Vix use Javascript for library management. Like manage errors/warnings/syntax and everything else inside the library or anything. 
- Vix allow C/C++/LLVM libraries to implement libraries from them you can use ```import library.h``` you can use Vix Std or C/C++ Std libraries too. You can make LLVM IR and use it in a library. 
- You can use C/C++/LLVM not only for libraries can be used for implemting custom compilers/frame work and everything else easily.
- ```Warning:``` Implementing some libraries from C/C++ require unsafe block usage. Not all libraries can be used in safe blocks

Installing a library could be a [memory shared files like ```dll/dylib etc...```](https://https://en.wikipedia.org/wiki/Shared_library) this could be if the library is close source or generate a IR in compilion time more informations about [Generating IR using libraries](https:///information/IRLibrary) with this way libraryies can generate the IR or a binary directly.

Vix implement 2 types of libraries you can use and both use for different things:
- ```1: Binary compilion:``` The library compile to binary and be inside the compiled application: bigger compiled app/easy to develop the library. This type of libraries can be ".bin" files too mean you can implement/make a library in any language that generate ".bin" files 
- ```2: IR Generating:``` The library generate the IR for the compiler: smaller compiled app/harder to develop


#### how C/C++/LLVM libraries compile in Vix library manager:
Vix allow you to run ```package compile``` command for compile libraries like C/C++/LLVM easily and automaticlly. But how this process goes when i run that command? The answer of your question is simple Deal. When you run ```package compile``` you acutlly run [package.*](https://https://github.com/) that first see if your language made with Vix if yes compile it into .bin file into this path:
```toml
    your_library
    |
    |- src
    |   |- main.c/main.cpp
    |
    |- bin
        |- your_library.bin # Here where your library binary lives.   
```
C/C++:
[package.*](https://https://github.com/) run clang and compile your main.c/main.cpp into this path:
```toml
    your_library
    |
    |- src
    |   |- main.c/main.cpp
    |   
    |- include
    |   |- include_main.c
```
- Example: include_main.c/include_main.cpp contain:
```C
    #ifndef MYLIB_H
    #define MYLIB_H

    int add(int a, int b);
    int sub(int a, int b);

    #endif
```
- **info:** You not focus to write anythuing inside include folder it's automaticlly generated by [package.*](https://https://github.com/)
- **info:** You can let your library compile by compiler in compilion time but that aren't recommanded beacuse your library will make compilion slower. That why ```.bin``` file are recommanded.
- You can use ```.bin``` for close libraries too
- C never know about the library it self by default that make it return errors like:
```c
    output.c:31:14: error: call to undeclared function 'add_something'; ISO C99 and later do not support implicit function declarations [-Wimplicit-function-declaration]
    31 | int32_t t2 = add_something(t0, t1);
    |                   ^
    1 error generated.

```
- Vix fix this by telling "C" about the coming functions/impls from the libraries them self so C never give a error.

### Library management & Compilion and creaton:
Vix use way to compile libraries without getting slow compiling time or giant compiled applications or take a big footprint space too. Vix compiler compiles everything from classes/functions/impls etc... that happen when you install a new library compile everything into .bin file with this way: ```Library Source code -> C -> Clang -> binary``` if the library build into C/C++ it don't go throw Vix compiler. The library be compiled by Clang compiler directly to binary. 
- You can make your own library in any language just you need LLVM/C/C++/Vix so compiler can use them.

To start your own library you can use ```package new "your library name"``` it will generate everything.
- for making your library be inside the compiled application:

```toml
    ExampleLibrary
    |
    |- src # your library source code can be in Vix/llvm/C/C++
    |   |- main.*
    |
    |- syntax # files to manage all the syntax
    |   |- error.js
    |   |- warning.js
    |   |- syntax.js
    |  # main manager for managing everything
    |- package.json
```

- Example ```Package.json```:
```json
    {
        "pacakge": {
            "name": "example_library",
            "version": "1.3.5",
                // language you wanna use or compiler
                "Compile": {
                    "Clang": "1.18.8"
                },
                // import library from any compiler you need
                "include": {
                    "Clang": [
                        "#include <time.h>"
                    ]
                },

        }
    }
```

To make a library in You will need to follow these steps:
- ```1:``` run in terminal: **package new "your library name"**
- ```2:``` make main.vix inside your library folder in src.
- ```3:``` Write the function or impl or struct inside your main.vix file

### Example:

```ruby
    func print_from_my_lib(s: str)
        print(str)
    end

    struct My_library:
        name = str
        health = int
    end

    impl My_library:
        My_library(
            name = "",
            health = 0
        )

        func add(&mut self, input_name: str, input_health: int)
            self.name += input_name
            self.health += input_health
        end

        func return_data(&self): Result[str, Option[int]]
            return Ok((self.name, self.health))
        end
    end
```

- Example usage:

```ruby
    import My_library

    func main()
        print_from_my_lib("Hello, world")

        My_library.add("player 1", 100)
        data = My_library.return_data()
    end
```

- You can compile your own library to .bin using: ```Package compile``` will automaticlly generate the *.bin
- To publish your library first: you need an account in https:// if you don't have an account sign up using an github or gmail account in https://signup use ```package upload```

- ```Warning:``` Making a library contain a malware can get your account banned. And can lead into legal problems.
- To make your own library in C/C++ follow these steps:

- ```1:``` make main.c in your library folder inside src
- ```2:``` make sure that in your package.json you have "Clang" in your include folder
- ```Warning:``` Compilers like gcc and g++ aren't allowed you can use only Clang. If you don't have clang follow these steps to install it:
    - On windows: Go to msys2 offical website: https://www.msys2.org/ install it then open msys2 terminal in your search bar write "msys2" you will find it: 
    ![Alt text](image/image.png)
    - ```First:``` update the packages by runnign this command in msys2 terminal:
    ```bash
        pacman -Syu
    ```
    - ```Second:``` Install clang for:
        - windows 64x: ```pacman -S mingw-w64-x86_64-clang```
        - windows 32x: ```pacman -S mingw-w64-i686-clang```
    
- For linux everything different you need to run this command on terminal:
    - ```first:``` open your terminal:
    - ```Second:``` run this command 
        - Arch Linux / Manjaro: ```sudo pacman -S clang```
        - Fedora: ```sudo pacman -S clang```
        - Ubuntu / Debian: ```sudo apt update``` then ```sudo apt install clang```
- For macOS:
    - ```first:``` open your terminal:
    - ```Second:``` run this command: ```xcode-select --install```


- Example code into your package.json:
```json
    {
        "pacakge": {
            "name": "example_library",
            "version": 1.3.5,
            // install libraries your library need to use
            "libraries": {
                "example_library": 1.2.0
            },
            // Compilers your library needs
            "include": {
                "Clang": 1.18.8
            }
        }
    }
```
- Create main.c into your src folder.
- Example code into your main.c:

```c
    int print_from_my_lib(char* s)
    {
        printf(%s, s)
    }
```
- To compile run "package compile".

To make a library in C++ it's the same deal as C:
- Example code into your package.json:
```json
    {
        "pacakge": {
            "name": "example_library",
            "version": 1.3.5,
            // install libraries your library need to use
            "libraries": {
                "example_library": 1.2.0
            },
            // Compilers your library needs
            "include": {
                "Clang": 1.18.8
            }
        }
    }
```

#### How Vix allow you to make their libraries in any language:
Vix compile libraries to ".bin" files that will be linked with final compiled application. Best thing about Vix library system that it allow any ".bin" file that mean you compile the library with any language you think of can generate ".bin" files and you can use it not only C/C++ but any other language too. 
- Example of linking a rust library to Vix library:
- lib.rs:
```rust
    fn add(i: i32, x: i32) {
        return i + x
    }
```
- main.vix:
```ruby
    func remove(i: int32, x: int32): int
        return i - x
    end
```
- package.json:

```json
    {
        "pacakge": {
            "name": "example_library",
            "version": 1.3.5,
            // install libraries your library need to use
            "libraries": {
                "example_library": 1.2.0
            },
            // Compilers your library needs
            "include": {
                "RustC": 1.10.10,
            }
        }
    }
```
- Run ```package compile``` and you will get inside "bin" folder: example_library.bin
- import your library example:

```ruby
    import example_library

    a = add(1, 3)
    b = remove(1, 5)

    print(a, b)
```
- Done. You have Vim + Rust. You can do any mix with any language can generate "bin" files
#### How libraries compile:
After main.rs take all libraries to ```impl LibraryManager``` it start compile by:
- Reading package.json files: Get include libraries/etc...
- Take all lines:
    - Vim: Send to: ```Lexer -> Parser -> TypeChecker -> AST -> Codegen -> IR -> Clang -> *.bin```
    - C/C++ Send to: ```Clang -> *.bin```
    - Other languages: 
        - install the language compiler. Compile it to *.bin file
- Link everything togather into final *.bin file 
All this done by this function in **Library Manager** After compiling it and compile the binary of **compiled application** link everything togather for final **compiled application**

### Compiler:
Vix compiler made using [Rust programing language](https://en.wikipedia.org/wiki/Rust_(programming_language)) that later went be a wrong move for the developing team. Vix compiles dirctly to C IR using [Vix.*](https:///information/compiler) middle man compiler for: 
- Generate C IR code.
- Memory safety checks
- Support Vix libraries
#### - blueprint:
Vix compiler works with cut the lines into tokens, send then back to **main.rs** as a tokens that will be send to **parser.rs** and types generate alone by **Type.rs**. Generate build in functions and send the types to **Type.rs** to compile from **functions.rs**. **Parser.rs** Generate the tokens and cut them into stmt/expr/types and then everything be handled by **TypeChecker.rs** check for type errors before send everything into the **Expr/Stmt/Codegen** to start generating the IR.

Vix compile to C IR is different code then normal Vaild C code. It's more complex/harder to understand and read/harder to analyze manually.
- Example C IR compiled by Vix compiler:
- Vix example code
```ruby
func main(): int
    extern "stdcall" from "user32.dll":
        func MessageBoxA(s: int, title: const str, text: const str, x: int)
    end

    func main(): int 
        MessageBoxA(0, "Test", "Hello, world", 0)
    end
end
```
- Generated C IR:
```c
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

extern void __stdcall MessageBoxA(int32_t s, const char* title, const char* text, int32_t x);

int32_t main() {
    int32_t t0 = 0;
    const char* t1 = "Test";
    const char* t2 = "Hello, world";
    int32_t t3 = 0;

    MessageBoxA(t0, t1, t2, t3);
}
```
#### - Entry/main.rs:
When you run on your terminal ```Vix command``` you call **fn main** inside the main.rs that what cuts your command and know what you need:
```rust
    let args: Vec<String> = env::args().collect();
```
- The main.rs detect if you need a help run **print_help()**, When detect **run** then:
    - check all files inside src that **.vix** and take all content inside them
    - get your current OS using: **let current_os = TargetOS::current();** Or if you ran "run --customOS" will compile to OS you selected
    - Call the Lexer to send all the lines to compile into Tokens: **let mut lexer = Lexer::new(&source_code);**
    - Send all tokens to Parser for generate stmt/expr **let parser = Parser::new(tokens, source_code.clone(), lexer.spans.clone());**
    - Get all program/struct/enum/impls back from parser **let (program, structs, enums, externs, _, _, _, impls, _, _, import_decls) = parser.parse();**
    - Get all libraries from "import" **LibraryManager::process_imports_from_decls(&all_import_decls, Some(target))**
    - Call codegen for generate stmt/expr: **let mut codegen = Codegen::new(arch, combined_source, main_filename);**
    - Generate impl/struct/impls etc... in:
    ```rust
        let c_code = match codegen.codegen_program_full(&program, &all_structs, &all_enums, &all_impls, &all_externs) {
            Ok(code) => {
                code
            }
            Err(_) => {
                eprintln!();
                eprintln!("{} Code generation failed", "Error:".red());
                std::process::exit(1);
            }
        };
    ```

```rust
    let command = if args.len() > 1 {
        args[1].as_str()
            } else {
        "run"
    };

    if command == "help" || command == "--help" || command == "-h" {
        print_help();
        return;
    }
```

- Vix in any IR generating add headers:
```c
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
```
- Compile type tokens like: **int/const/char/str/float/maybe** from Vix to C IR using ```to_c_type``` function.
```rust
    pub fn to_c_type(&self, arch: &ArchConfig) -> String {
        match self {
            Type::Const(inner) => {
                format!("const {}", inner.to_c_type(arch))
            }
            
            Type::Int { bits, signed } => match (bits, signed) {
                (8, true) => "int8_t".to_string(),
                (16, true) => "int16_t".to_string(),
                (32, true) => "int32_t".to_string(),
                (64, true) => "int64_t".to_string(),
                (128, true) => "__int128".to_string(),
                (8, false) => "uint8_t".to_string(),
                (16, false) => "uint16_t".to_string(),
                (32, false) => "uint32_t".to_string(),
                (64, false) => "uint64_t".to_string(),
                (128, false) => "unsigned __int128".to_string(),
                (b, true) => format!("int{}_t", b),
                (b, false) => format!("uint{}_t", b),
            },
            
            Type::Float { bits } => match bits {
                32 => "float".to_string(),
                64 => "double".to_string(),
                128 => "long double".to_string(),
                _ => format!("_Float{}", bits),
            },
        
            Type::Char { bits, .. } => match bits {
                8 => "char".to_string(),
                32 => "uint32_t".to_string(),
                _ => format!("uint{}_t", bits),
            },
            Type::SelfType => "Self".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Void => "void".to_string(),
            Type::Ptr(inner) => format!("{}*", inner.to_c_type(arch)),
            Type::RawPtr(inner) => format!("{}*", inner.to_c_type(arch)),
            Type::ConstStr { .. } => "const char*".to_string(),
            Type::Str { len_type } => format!("struct {{ char* ptr; {} len; }}", len_type.to_c_type(arch)),
            Type::StrSlice { char_type, length_type } => format!("struct {{ {}* ptr; {} len; }}",  char_type.to_c_type(arch),  length_type.to_c_type(arch)),
            Type::Struct { name } => name.clone(),
            Type::Array { element, size: Some(_) } => {element.to_c_type(arch)}
            Type::Array { element, size: None } => {format!("struct {{ {}* ptr; size_t len; }}", element.to_c_type(arch))}
            Type::Intersection { types } => {types.first().map(|t| t.to_c_type(arch)).unwrap_or_else(|| "void".to_string())}
            Type::TripleDot => "...".to_string(),
            Type::MultiArray { element, dimensions: _ } => {element.to_c_type(arch)}
            Type::Variadic => "...".to_string(),
            Type::Any => "void*".to_string(),
            Type::Trait => "void*".to_string(),
            Type::Owned(inner) | Type::Ref(inner) | Type::MutRef(inner) => format!("{}*", inner.to_c_type(arch)),
            Type::FnPtr { params, return_type } => {
                let param_types: Vec<String> = params.iter().map(|p| p.to_c_type(arch)).collect();
                format!("{} (*)({})", return_type.to_c_type(arch), param_types.join(", "))
            }

            Type::Tuple { fields } => {
                let sanitized: Vec<String> = fields.iter().map(|f| f.name().chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect()).collect();
                format!("Tuple_{}", sanitized.join("_"))
            }
            
            
            Type::Union { variants } => {
                let sanitized: Vec<String> = variants.iter().map(|v| v.name().chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect()).collect();
                format!("Union_{}", sanitized.join("_"))
            }

            Type::Option { inner } => {
                let sanitized = inner.name().chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect::<String>();
                format!("Option_{}", sanitized)
            }
            
            Type::Result { ok, err } => {
                let ok_sanitized = ok.name().chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect::<String>();
                let err_sanitized = err.name().chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect::<String>();
                
                format!("Result_{}_{}", ok_sanitized, err_sanitized)
            }
        }
    }
```
- Take the generated AST into a Codegen struct:
```rust
pub struct Codegen {
    pub config: CodegenConfig,
    pub type_registry: TypeRegistry,
    pub impl_methods: HashMap<(String, String), (Vec<(String, Type)>, Type)>,
    pub c_code: String,
    pub globals: String,
    pub var_count: usize,
    pub label_count: usize,
    pub vars: HashMap<String, (String, Type)>,
    pub owned_vars: HashSet<String>,
    pub extern_functions: HashMap<String, ExternFunctionMap>,
    pub extern_block: HashMap<String, ExternFunctionMap>,
    pub structs: HashMap<String, StructInfo>,
    pub module_vars: HashMap<(String, String), (String, Type, bool)>,
    pub module_functions: HashMap<(String, String), (Vec<(String, Type)>, Type)>,
    pub compilation_mode: CompilationMode,
    pub exported_functions: Vec<String>,
    pub externs_with_bodies: HashSet<String>,
    pub unsafe_depth: usize,
    pub scope_depth: usize,
    pub user_functions: HashMap<String, (Vec<(String, Type)>, Type)>,
    pub ir: IR,
    pub arch: ArchConfig,
    pub diagnostics: DiagnosticHandler,
    pub source_code: String,
    pub current_file: String,
    pub linked_libraries: Vec<String>,
}
```
- Call function: stmt/expr to call functions that generate the IR:
```rust
    pub fn codegen_expr(&mut self, expr: &Expr, body: &mut String) -> Result<(String, Type), ()>
    pub fn codegen_stmt(&mut self, stmt: &Stmt, body: &mut String) -> Result<(), ()> 
```
- ```codegen_expr``` used for compile for generate IR for ```int/float/str/maybe/char``` and generate things like ```array/var/build in functions etc``` etc... all being generated by it.
- ```codegen_stmt``` used for compile blocks like ```if statment```, ```functions```, ```scope```, ```struct``` etc... into Codegen IR.
- **Info:** codegen_stmt functions use ```codegen_expr```

- More infomrations coming soon:
