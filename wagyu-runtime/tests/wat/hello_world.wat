(module
  (import "stdout" "get-stdout" (func $write (param i32 i32 i32) (result i32)))

  (memory 1)
  (data (i32.const 0) "Hello, World!\n")

  (func (export "helloWorld") (result i32)
    (i32.const 0)            ;; file descriptor: stdout
    (i32.const 0)           ;; address of the string
    (i32.const 13)           ;; length of the string
    call $write             ;; call the write function
    drop                     ;; drop the result (number of bytes written)
    (i32.const 0)            ;; return 0 (success)
  )
)