(module
  (func (export "helloWorld") (result i32)
    (i32.const 0)            ;; file descriptor: stdout
    (i32.const 16)           ;; address of the string
    (i32.const 13)           ;; length of the string
    call $write             ;; call the write function
    drop                     ;; drop the result (number of bytes written)
    (i32.const 0)            ;; return 0 (success)
  )

  (func $write
    (import "env" "write" (func $write (param i32 i32 i32) (result i32)))
  )

  (data (i32.const 16) "Hello, World!\n")
)