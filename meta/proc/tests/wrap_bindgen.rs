#![cfg(test)]
#![allow(clippy::unit_cmp)]
#![allow(clippy::missing_safety_doc)]

use reascript_proc::wrap_bindgen;

pub struct Inner;

#[wrap_bindgen]
impl Inner {
    #[allow(clippy::unused_unit)]
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

#[test]
fn test_proc_wrap_bindgen() {
    let reaper = REAPER(Inner);
    assert!(reaper.no_args_with_return());
    assert_eq!(reaper.with_args(true, 0), (true, 0));
    assert_eq!(reaper.unit(), ());
}
