### Update Alpha: 2.5v
Overall libraries are allowed to use ".something()" and new build in function: ```plan()``` are included with so much usaful things. And ```const``` can be used in top of the file too now

- Plan build in function:
    Plan is like ```format()``` function in rust it allow you to make like:
        ```ruby
            plan("Hello, what is your name {}, e)
        ```
    - It's simple to use, and can be used in a "if" blocks and return too:
        ```ruby
            if plan("age {}", x) == "age 10" then
            end

            ### or

            if a == plan("age {}", x) then
            end
        ```
    - Example script:
        ```ruby
            name = "Bob"
            age = 20

            informations_of_bob = plan("Hi i'm {} nice to meet you, I'm {}", name, age)
        ```
- Libraries can be used their functions/implements etc... like this: "i.check_error()" same goes to your functions too:
    - Example script in root/
        ```ruby
            func is_number(input: int)
                return true
            end

            func example(input1: int, input2: int)
            end

            i = 10  

            # here "()" are optional:
            if i.is_number then
                i.example(10) # "i" is "input1" and "10 is input2"
            end
        ```
    - In libraries can do the same like:
        - library/
            ```ruby
                func is_number(input: int)
                    return true
                end

                func example(input1: int, input2: int)
                end
            ```
        - root/
            ```ruby
                i = 10  

                # here "()" are optional:
                if i.is_number then
                    i.example(10) # "i" is "input1" and "10 is input2"
                end
            ```

- Color library, first library in Vix are made by [Vix developement team]() Used for testing new [library management features]() like 
    Color is simple library used for colors don't take any much space of your ```compile application``` and make you print things with custom colors and types like "bold/italic" etc...
    - Example script:
        ```ruby
            print("Information".blue().bold(), "Write your name in the exam")
            i = plan("{}, Failed to load the game assients", "Error".red().italic())
            x = "Hello world".purple().bold()
            print(i)
            print(x)
        ```
