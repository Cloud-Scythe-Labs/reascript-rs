#![cfg(test)]

use reascript_gen::wrap_bindgen;

struct Inner;

#[allow(dead_code)]
impl Inner {
    pub unsafe fn unit(&self) {
        ()
    }
    pub unsafe fn no_args_with_return(&self) -> bool {
        true
    }
    pub unsafe fn with_args(&self, arg1: bool, arg2: u32) -> (bool, u32) {
        (arg1, arg2)
    }
}

struct Outer(Inner);

#[test]
fn test_gen_wrap_bindgen() {
    wrap_bindgen!(Outer, no_args_with_return, (), bool);
    wrap_bindgen!(Outer, with_args, (arg1: bool, arg2: u32), (bool, u32));
    wrap_bindgen!(Outer, unit, (), ());

    let outer = Outer(Inner);
    assert!(outer.no_args_with_return());
    assert_eq!(outer.with_args(true, 0), (true, 0));
    assert_eq!(outer.unit(), ());
}
