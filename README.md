# Cobalt Lang
<ol>
<a href="https://discord.gg/bVmkQTKrqm">Discord</a>
</ol>

> **Warning**
> To compile and use Cobalt on windows you will need to follow this [StackOverflow](https://stackoverflow.com/a/60024490/13098065) post

Ye, im just as confused as you... Who made this? Who thought cobol was a good idea? Is this good? Who knows.
Here is an example :) (Please note this language is REALLY NEW :) so lets not get too excited.)  
```
DEFINE FUNCTION "test_func_call" THAT RETURNS A String: 

    SET "to_print" EQUAL TO "what?".
    CALL FUNCTION "printf" WITH THE ARGUMENT "to_print".

END FUNCTION "test_func_call".

DEFINE FUNCTION "main" THAT RETURNS A String: 

    CALL FUNCTION "test_func_call".

END FUNCTION "main".
```  
Ye, it doesn't do much.

## Using

> **Note**
> The officially supported extensions are `.cbt` and `.cobalt`. However, it doesn't matter what you use.   

**This version of Cobalt will ONLY run the tests specified in `/tests`. To play around with the code you MUST create a file here.**

To get started, you will need to have `LLVM 14`, Rust, C-lang (`clang`). Start by running `cargo run`, this will lex, parse and compile
the files in `tests/`. Then run `clang` on the resulting `.ll` file (i.e. `clang cbt.ll`). Then you can run the resulting executable (most likely just `a.out`).