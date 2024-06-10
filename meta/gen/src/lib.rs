#![allow(non_snake_case, dead_code)]

use reaper_low::Reaper as CReaper;

pub struct Reaper(CReaper);

#[macro_export]
macro_rules! wrap_bindgen {
    ($reaper:ident, $method:ident, ($($arg:ident: $arg_type:ty),*)) => {
        impl $reaper {
            pub fn $method(&self, $($arg: $arg_type),*) {
                unsafe { (self.0).$method($($arg),*) }
            }
        }
    };
}

#[cfg(test)]
mod gen_macro_tests {
    use crate::Reaper;
    use std::ffi;

    #[test]
    fn test_wrap_bindgen() {
        wrap_bindgen!(Reaper, ShowConsoleMsg, (msg: *const ffi::c_char));
    }
}

#[cfg(test)]
mod proc_macro_tests {
    use reascript_proc::wrap_bindgen;

    pub struct Inner;

    #[wrap_bindgen]
    impl Inner {
        pub unsafe fn is_true(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_wrap_bindgen() {
        let reaper = REAPER(Inner);
        assert!(reaper.is_true());
    }
}
