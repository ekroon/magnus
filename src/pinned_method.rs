//! Extensions to the method system for pinned arguments.

use std::{
    ffi::c_void,
    panic::AssertUnwindSafe,
    pin::Pin,
};

use crate::{
    error::{Error, raise},
    method::ReturnValue,
    pin_convert::StackPinned,
    try_convert::TryConvert,
    value::{ReprValue, Value},
};

/// Helper trait for wrapping a function as a Ruby method with one pinned argument.
///
/// This trait extends the method system to support `Pin<&mut StackPinned<T>>` arguments
/// that are guaranteed to remain at a fixed memory location during method execution.
#[doc(hidden)]
pub trait PinnedMethod1<RbSelf, T, Res>
where
    Self: Sized + Fn(RbSelf, Pin<&mut StackPinned<T>>) -> Res,
    RbSelf: TryConvert,
    T: TryConvert + ReprValue,
    Res: ReturnValue,
{
    #[inline]
    fn call_convert_value_pinned(self, rb_self: Value, arg: Value) -> Result<Value, Error> {
        let converted_self = TryConvert::try_convert(rb_self)?;
        let converted_arg = TryConvert::try_convert(arg)?;
        let mut pinned = StackPinned::new(converted_arg);
        let pin = unsafe { Pin::new_unchecked(&mut pinned) };
        (self)(converted_self, pin).into_return_value()
    }

    #[inline]
    unsafe fn call_handle_error_pinned(self, rb_self: Value, arg: Value) -> Value {
        let res = match std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.call_convert_value_pinned(rb_self, arg)
        })) {
            Ok(v) => v,
            Err(e) => Err(Error::from_panic(e)),
        };
        match res {
            Ok(v) => v,
            Err(e) => raise(e),
        }
    }
}

impl<Func, RbSelf, T, Res> PinnedMethod1<RbSelf, T, Res> for Func
where
    Func: Fn(RbSelf, Pin<&mut StackPinned<T>>) -> Res,
    RbSelf: TryConvert,
    T: TryConvert + ReprValue,
    Res: ReturnValue,
{
}

/// Trait for pinned method function pointers.
pub trait PinnedMethod: private::PinnedMethod {}

impl<T> PinnedMethod for T where T: private::PinnedMethod {}

mod private {
    use super::*;

    pub unsafe trait PinnedMethod {
        fn arity() -> i8;
        fn as_ptr(self) -> *mut c_void;
    }

    unsafe impl PinnedMethod for unsafe extern "C" fn(Value, Value) -> Value {
        fn arity() -> i8 {
            1
        }

        fn as_ptr(self) -> *mut c_void {
            self as *mut c_void
        }
    }
}

/// Wrap a Rust function with pinned arguments for Ruby method registration.
///
/// This macro is similar to `method!` but supports `Pin<&mut StackPinned<T>>` 
/// arguments that are guaranteed to be pinned on the stack during execution.
///
/// # Examples
///
/// ```ignore
/// use magnus::{Error, RString, pin_convert::StackPinned, pinned_method};
/// use std::pin::Pin;
///
/// fn hello_pinned(rb_self: RString, pinned_str: Pin<&mut StackPinned<RString>>) -> String {
///     format!("{}: {}", rb_self, pinned_str.get_value_ref())
/// }
///
/// // Register the method
/// class.define_method("hello_pinned", pinned_method!(hello_pinned, 1))?;
/// ```
#[macro_export]
macro_rules! pinned_method {
    ($name:expr, 1) => {{
        unsafe extern "C" fn anon(rb_self: $crate::Value, a: $crate::Value) -> $crate::Value {
            use $crate::pinned_method::PinnedMethod1;
            $name.call_handle_error_pinned(rb_self, a)
        }
        anon as unsafe extern "C" fn($crate::Value, $crate::Value) -> $crate::Value
    }};
    ($name:expr, $arity:expr) => {
        compile_error!("pinned_method! currently only supports arity 1")
    };
}
