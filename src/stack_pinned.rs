use std::{ops::{Deref, DerefMut}, pin::Pin};

use crate::{gc, value::ReprValue};

/// Trait for values that can be pinned to the stack so Ruby's GC knows about them.
///
/// Registering a value prevents Ruby's GC from moving or collecting it while
/// the value is pinned. Implementors are expected to call [`gc::register_address`]
/// in [`StackPinned::register`] and [`gc::unregister_address`] in
/// [`StackPinned::deregister`].
///
/// This trait is implemented for all types that implement [`ReprValue`].
pub trait StackPinned: ReprValue {
    /// Register the value with Ruby's GC.
    fn register(&self) {
        gc::register_address(self);
    }

    /// Deregister the value with Ruby's GC.
    fn deregister(&self) {
        gc::unregister_address(self);
    }
}

impl<T> StackPinned for T where T: ReprValue {}

/// Guard that deregisters the wrapped value when dropped.
pub struct PinGuard<'a, T: StackPinned + ?Sized> {
    value: Pin<&'a mut T>,
}

impl<'a, T: StackPinned + ?Sized> PinGuard<'a, T> {
    /// Create a new pinned value, registering it with Ruby's GC.
    ///
    /// This constructor requires `T: Unpin`. Use [`PinGuard::new_unchecked`]
    /// if `T` is not `Unpin`.
    pub fn new(value: &'a mut T) -> Self
    where
        T: Unpin,
    {
        // safe as `T: Unpin`
        let pin = Pin::new(value);
        pin.as_ref().register();
        Self { value: pin }
    }

    /// Create a new pinned value without requiring `T: Unpin`.
    ///
    /// # Safety
    ///
    /// The caller must ensure `value` will not move while the returned
    /// `PinGuard` is alive.
    pub unsafe fn new_unchecked(value: &'a mut T) -> Self {
        let pin = Pin::new_unchecked(value);
        pin.as_ref().register();
        Self { value: pin }
    }

    /// Access the pinned value as `Pin<&mut T>`.
    pub fn as_pin(&mut self) -> Pin<&mut T> {
        self.value.as_mut()
    }
}

impl<'a, T: StackPinned + ?Sized> Deref for PinGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value.as_ref().get_ref()
    }
}

impl<'a, T: StackPinned + ?Sized> DerefMut for PinGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Pin::get_unchecked_mut(self.value.as_mut()) }
    }
}

impl<'a, T: StackPinned + ?Sized> Drop for PinGuard<'a, T> {
    fn drop(&mut self) {
        self.value.as_ref().deregister();
    }
}
