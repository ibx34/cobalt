# Cobalt Lang
<ol>
<a href="https://discord.gg/bVmkQTKrqm">Discord</a>
</ol>


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
