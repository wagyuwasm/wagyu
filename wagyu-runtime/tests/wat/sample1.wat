(module
  (func (export "addTwo") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
  )
  (func
    return
  )
  (func
    return
  )
  (func $temp (param $x i64) (result i64)
    local.get 0
    i64.const -65
    i64.add
  )
  (func (param $x f32) (result f32)
    local.get 0
    f32.const 1
    f32.add
  )
)
