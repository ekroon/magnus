use magnus::{embed::init, pinned_value, Ruby, Value};

#[magnus::pin_args]
fn echo(_ruby: &Ruby, mut val: Value) -> Value {
    {
        let _p = pinned_value!(val);
        // value is pinned for the scope of `_p`
    }
    val
}

#[test]
fn it_pins_value() {
    let ruby = unsafe { init() };
    let val: Value = ruby.eval("'hello'").unwrap();
    let res = echo(&ruby, val);
    assert_eq!(res.to_string(), "hello");
}
