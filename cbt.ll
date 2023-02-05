; ModuleID = 'main'
source_filename = "main"

@only_global_var = private unnamed_addr constant [14 x i8] c"Snout is cute\00", align 1

declare i32 @printf(...)

define i32 @not_main(...) {
entry:
  %func_printf_call = call i32 (...) @printf()
  ret i32 %func_printf_call
}

define i32 @main() {
entry:
  %call_anon_func = call i32 (...) @not_main(i8* getelementptr inbounds ([14 x i8], [14 x i8]* @only_global_var, i32 0, i32 0))
  ret i32 %call_anon_func
}
