### Update Alpha 3.25v
Update contain fixing 3 main bugs and 5 different other bugs are making vix unsable and can lead to a problems in future. Thanks for:
[nevakrien](https://github.com/nevakrien) for telling us about this creatcal problem in vix!.

### Bugs:
- First main bug in "Tuple" generating is not allowing **()** aka: ('empty turple') this couse a problems. When you do empty turple before this bug being fixed it always will generate:
```c
typedef struct {
    void field_0; // "filed_0" is forbien "void"
    Slice_char field_1;
} Tuple_void_char8;
```
- This so wrong. We has been fixed this bug and now it allow empty tuples **()**
```ruby
    func example(): Result[(), str]
        Ok() # Now works fine!
        Err("some err") # Works too!
    end
```

- Second main bug, is a really danger one beacuse it can lead to your program do random behiver beacuse we use [malloc]() for so many things from build in functions and features inside the language. Thanks for [nevakrien](https://github.com/nevakrien) for finding this bug.
- Malloc can be [null]() this what will couse so many problmes like random behiver/crashes etc... we has been fixed this by adding a checker function to make sure it's not [null]():
```c
    void* vix_alloc(size_t s){
      void* ans = malloc(s);
      // we going to add better error handling in future. But for now this going to be the function at least remove the this problem
       If(!ans) exit(1);
       return ans;
    }
```

- Third: Enums before cannot generate ```"Something(filed1: type, filed2: type)``` This happens because the [parser]() never parser that syntax. This has been fixed
- Now when you do:
```ruby
enum Test:
    Player(name: str, health: int)
end

func main(): Test
    return Player.Test(
        name: "Player 1",
        health: 100
    )
end
```
- It generate correct code:
```c
typedef struct Test_Player_Payload {
    Slice_char name;
    int32_t health;
} Test_Player_Payload;

Test_Player_Payload Test_Player_Payload_new(Slice_char name, int32_t health);
typedef enum {
    Test__Player = 0,
} TestTag;

typedef struct {
    TestTag tag;
    union {
        Test_Player_Payload Player;
    } data;
} Test;
int32_t vix_main();

Test_Player_Payload Test_Player_Payload_new(Slice_char name, int32_t health) {
    Test_Player_Payload instance;
    instance.name = name;
    instance.health = health;
    return instance;
}
```

- Mean while we testing it and we found another bug the codegen wasn't compiling:
```ruby
    return Player.Test(
        name: "Player 1",
        health: 100
    )
```
- This bug has been fixed too


- Fith bug is about [while]() when you do any while loop it always will not generate beacuse of ```codegen_enum``` is being failing it always generate: ```lable_0:``` without no loop at all. This problem has been fixed.

- Last bug: When you do example:
```ruby

impl Test:
    func new(): Test
        Test(
            i = 0,
            x = ""
        )
    end

    func test(&mut self, input: int): Result[int, str]
        if input != 0 then
            self.i = 10
            return Ok(0)
        end

        Err("Failed: input is zero")
    end
end

func main()
    create i: int = 10
    create test = Test
    create x = test.test(i)
end
```
- This generate a correct impl IR Code but everything inside ```func main()``` is generating incorrect IR:
```c
int32_t vix_main() {
    int32_t t0 = 10;
    const int32_t var_i = t0;
    test_test(var_i);
    int32_t t1 = 0;
}
```
- This bug has been fixed!
