//! Generative macros.

/// Creates an `impl` block on the wrapper type of methods on the inner type.
#[macro_export]
macro_rules! wrap_bindgen {
    ($wrapper:ident, $method:ident, ($($arg:ident: $arg_type:ty),*), $return_ty:ty) => {
        impl $wrapper {
            pub fn $method(&self, $($arg: $arg_type),*) -> $return_ty {
                unsafe { (self.0).$method($($arg),*) }
            }
        }
    };
}
