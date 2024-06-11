//! Generative macros.

/// Creates an `impl` block on the wrapper type of methods on the inner type.
/// Useful for when the wrapper type already exists, and manually writing a
/// wrapper method is necessary.
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
