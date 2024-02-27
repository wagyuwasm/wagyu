(module
 (export "fib" (func $fib))
 (func $fib (param $n i64) (result i64)
  (if
   (i64.lt_s
    (local.get $n)
    (i64.const 2)
   )
   (return
    (i64.const 1)
   )
  )
  (return
   (i64.add
    (call $fib
     (i64.sub
      (local.get $n)
      (i64.const 2)
     )
    )
    (call $fib
     (i64.sub
      (local.get $n)
      (i64.const 1)
     )
    )
   )
  )
 )
)