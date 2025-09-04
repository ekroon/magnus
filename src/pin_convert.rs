//! Traits and utilities for stack-pinned Ruby value conversion.

use std::{
    marker::PhantomPinned,
    pin::Pin,
};

use crate::{
    error::Error,
    value::{BoxValue, ReprValue, Value},
    try_convert::TryConvert,
};

/// A wrapper type that prevents unpinning of Ruby values.
///
/// This type is `!Unpin` and is designed to be used with `Pin<&mut StackPinned<T>>`
/// to provide stack-pinned Ruby values that cannot be moved. This is useful for
/// scenarios where you need to ensure a Ruby value remains at a fixed memory
/// location on the stack during method execution.
#[repr(transparent)]
pub struct StackPinned<T> {
    value: T,
    _pin: PhantomPinned,
}

impl<T> StackPinned<T> {
    /// Create a new `StackPinned` wrapper around a value.
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            value,
            _pin: PhantomPinned,
        }
    }

    /// Get a reference to the wrapped value.
    #[inline]
    pub fn get_value_ref(self: Pin<&Self>) -> &T {
        // SAFETY: We're only projecting to a field, not moving the struct
        &self.get_ref().value
    }

    /// Get a mutable reference to the wrapped value.
    #[inline]
    pub fn get_value_mut(self: Pin<&mut Self>) -> &mut T {
        // SAFETY: We're only projecting to a field, not moving the struct
        unsafe { &mut self.get_unchecked_mut().value }
    }

    /// Get the wrapped value as a Value.
    #[inline]
    pub fn as_value_ref(self: Pin<&Self>) -> Value
    where
        T: ReprValue,
    {
        // SAFETY: We're only projecting to a field, not moving the struct
        self.get_ref().value.as_value()
    }
}

impl<T> StackPinned<T>
where
    T: ReprValue,
{
    /// Convert the pinned value into a `BoxValue`, moving it out of the `Pin`.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it moves a `!Unpin` type out of a `Pin`.
    /// It should only be used when you can guarantee that no code relies on
    /// the pinned location of the value, and that the value will not be used
    /// after calling this method.
    ///
    /// The resulting `BoxValue` will be heap-allocated and protected from GC.
    pub unsafe fn into_box_value(self: Pin<&mut Self>) -> BoxValue<T> {
        unsafe {
            // SAFETY: Caller guarantees that nothing relies on the pinned location
            let value = std::ptr::read(&self.get_unchecked_mut().value);
            BoxValue::new(value)
        }
    }
}

/// Trait for converting Ruby values to pinned Rust types on the stack.
///
/// This trait is similar to `TryConvert` but is designed for creating
/// stack-pinned values that cannot be moved during method execution.
pub trait TryConvertPinned<T>: Sized {
    /// Convert a Ruby `Value` into a stack-pinned Rust type.
    ///
    /// The conversion creates a pinned value that cannot be moved,
    /// which is useful for ensuring memory stability during method calls.
    fn try_convert_pinned(val: Value) -> Result<Pin<Box<StackPinned<Self>>>, Error>;
}

impl<T> TryConvertPinned<T> for T
where
    T: TryConvert + ReprValue,
{
    fn try_convert_pinned(val: Value) -> Result<Pin<Box<StackPinned<Self>>>, Error> {
        let converted = T::try_convert(val)?;
        let pinned = StackPinned::new(converted);
        Ok(Box::pin(pinned))
    }
}

/// Create a stack-pinned value from any ReprValue type.
///
/// This function uses the `pin!` macro to create a stack-pinned value,
/// which is the recommended way to pin `!Unpin` types on the stack.
///
/// # Examples
///
/// ```
/// use magnus::{Error, Ruby, pin_convert};
///
/// fn example(ruby: &Ruby) -> Result<(), Error> {
///     let ruby_str = ruby.str_new("hello");
///     let pinned = pin_convert::pin_on_stack(ruby_str);
///     
///     // The pinned value is now on the stack and cannot be moved
///     let value_ref = pinned.as_ref().get_value_ref();
///     println!("Pinned: {}", value_ref);
///     
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! pin_on_stack {
    ($value:expr) => {{
        let pinned_value = $crate::pin_convert::StackPinned::new($value);
        std::pin::pin!(pinned_value)
    }};
}

/// Utility function to create a pinned value from any ReprValue type.
///
/// This is a convenience function for creating stack-pinned values
/// that can be used in method signatures. Use the `pin_on_stack!` macro
/// for the most efficient stack allocation.
pub fn pin_value<T>(value: T) -> Pin<Box<StackPinned<T>>>
where
    T: ReprValue,
{
    Box::pin(StackPinned::new(value))
}

/// Convert a pinned value to a BoxValue safely.
///
/// This function provides a safe wrapper around the unsafe `into_box_value` method
/// by ensuring proper handling of the conversion.
pub fn pin_to_box<T>(mut pinned: Pin<Box<StackPinned<T>>>) -> BoxValue<T>
where
    T: ReprValue,
{
    unsafe {
        // SAFETY: We own the pinned box, so we can safely move out of it
        pinned.as_mut().into_box_value()
    }
}
