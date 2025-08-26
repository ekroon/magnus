//! Types and Traits for working with Ruby’s Numeric class.

use std::fmt;

use rb_sys::{VALUE, rb_num_coerce_bin, rb_num_coerce_bit, rb_num_coerce_cmp, rb_num_coerce_relop};

use crate::{
    Ruby,
    error::{Error, protect},
    into_value::IntoValue,
    try_convert::TryConvert,
    value::{
        IntoId, NonZeroValue, ReprValue, Value,
        private::{self, ReprValue as _},
    },
};

/// Functions available for all of Ruby's Numeric types.
pub trait Numeric: ReprValue + Copy {
    /// Apply the operator `op` with coercion.
    ///
    /// As Ruby's operators are implemented as methods, this function can be
    /// thought of as a specialised version of [`Value::funcall`], just for
    /// subclasses of `Numeric`, and that follows Ruby's coercion protocol.
    ///
    /// Returns `Ok(U)` if the method returns without error and the return
    /// value converts to a `U`, or returns Err if the method raises or the
    /// conversion fails.
    ///
    /// The returned errors are tailored for binary operators such as `+`, `/`,
    /// etc.
    ///
    /// # Examples
    ///
    /// ```
    /// use magnus::{Error, Numeric, Ruby};
    ///
    /// fn example(ruby: &Ruby) -> Result<(), Error> {
    ///     let a = ruby.integer_from_i64(2);
    ///     let b = ruby.integer_from_i64(3);
    ///     let c: i64 = a.coerce_bin(b, "+")?;
    ///     assert_eq!(c, 5);
    ///
    ///     Ok(())
    /// }
    /// # Ruby::init(example).unwrap()
    /// ```
    ///
    /// Avoiding type conversion of the result to demonstrate Ruby is coercing
    /// the types:
    ///
    /// ```
    /// use magnus::{Error, Float, Numeric, Ruby, Value};
    ///
    /// fn example(ruby: &Ruby) -> Result<(), Error> {
    ///     let a = ruby.integer_from_i64(2);
    ///     let b = ruby.float_from_f64(3.5);
    ///     let c: Value = a.coerce_bin(b, "+")?;
    ///     let c = Float::from_value(c);
    ///     assert!(c.is_some());
    ///     assert_eq!(c.unwrap().to_f64(), 5.5);
    ///
    ///     Ok(())
    /// }
    /// # Ruby::init(example).unwrap()
    /// ```
    fn coerce_bin<T, ID, U>(self, other: T, op: ID) -> Result<U, Error>
    where
        T: Numeric,
        ID: IntoId,
        U: TryConvert,
    {
        let op = op.into_id_with(&Ruby::get_with(self));
        protect(|| unsafe {
            Value::new(rb_num_coerce_bin(
                self.as_rb_value(),
                other.as_rb_value(),
                op.as_rb_id(),
            ))
        })
        .and_then(TryConvert::try_convert)
    }

    /// Apply the operator `op` with coercion.
    ///
    /// As Ruby's operators are implemented as methods, this function can be
    /// thought of as a specialised version of [`Value::funcall`], just for
    /// subclasses of `Numeric`, and that follows Ruby's coercion protocol.
    ///
    /// Returns `Ok(U)` if the method returns without error and the return
    /// value converts to a `U`, or returns Err if the method raises or the
    /// conversion fails.
    ///
    /// The returned errors are tailored for comparison operators such as `<=>`.
    ///
    /// Note, if coercion fails this will return `nil`, if you want to detect
    /// this you should set the result type to `Option<U>`. Other errors in
    /// applying `op` will still result in an `Err`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::num::NonZeroI64;
    ///
    /// use magnus::{Error, Numeric, Ruby};
    ///
    /// fn example(ruby: &Ruby) -> Result<(), Error> {
    ///     let a = ruby.rational_new(1, NonZeroI64::new(4).unwrap());
    ///     let b = ruby.float_from_f64(0.3);
    ///     let result: i64 = a.coerce_cmp(b, "<=>")?;
    ///     assert_eq!(result, -1);
    ///
    ///     Ok(())
    /// }
    /// # Ruby::init(example).unwrap()
    /// ```
    fn coerce_cmp<T, ID, U>(self, other: T, op: ID) -> Result<U, Error>
    where
        T: Numeric,
        ID: IntoId,
        U: TryConvert,
    {
        let op = op.into_id_with(&Ruby::get_with(self));
        protect(|| unsafe {
            Value::new(rb_num_coerce_cmp(
                self.as_rb_value(),
                other.as_rb_value(),
                op.as_rb_id(),
            ))
        })
        .and_then(TryConvert::try_convert)
    }

    /// Apply the operator `op` with coercion.
    ///
    /// As Ruby's operators are implemented as methods, this function can be
    /// thought of as a specialised version of [`Value::funcall`], just for
    /// subclasses of `Numeric`, and that follows Ruby's coercion protocol.
    ///
    /// Returns `Ok(U)` if the method returns without error and the return
    /// value converts to a `U`, or returns Err if the method raises or the
    /// conversion fails.
    ///
    /// The returned errors are tailored for relationship operators such as
    /// `<=`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::num::NonZeroI64;
    ///
    /// use magnus::{Error, Numeric, Ruby};
    ///
    /// fn example(ruby: &Ruby) -> Result<(), Error> {
    ///     let a = ruby.float_from_f64(0.3);
    ///     let b = ruby.rational_new(1, NonZeroI64::new(4).unwrap());
    ///     let result: bool = a.coerce_cmp(b, "<=")?;
    ///     assert_eq!(result, false);
    ///
    ///     Ok(())
    /// }
    /// # Ruby::init(example).unwrap()
    /// ```
    fn coerce_relop<T, ID, U>(self, other: T, op: ID) -> Result<U, Error>
    where
        T: Numeric,
        ID: IntoId,
        U: TryConvert,
    {
        let op = op.into_id_with(&Ruby::get_with(self));
        protect(|| unsafe {
            Value::new(rb_num_coerce_relop(
                self.as_rb_value(),
                other.as_rb_value(),
                op.as_rb_id(),
            ))
        })
        .and_then(TryConvert::try_convert)
    }

    /// Apply the operator `op` with coercion.
    ///
    /// As Ruby's operators are implemented as methods, this function can be
    /// thought of as a specialised version of [`Value::funcall`], just for
    /// subclasses of `Numeric`, and that follows Ruby's coercion protocol.
    ///
    /// Returns `Ok(U)` if the method returns without error and the return
    /// value converts to a `U`, or returns Err if the method raises or the
    /// conversion fails.
    ///
    /// The returned errors are tailored for bitwise operators such as `|`,
    /// `^`, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// use magnus::{Error, Numeric, Ruby};
    ///
    /// fn example(ruby: &Ruby) -> Result<(), Error> {
    ///     let a = ruby.integer_from_i64(0b00000011);
    ///     let b = ruby.integer_from_i64(0b00001110);
    ///     let result: i64 = a.coerce_cmp(b, "^")?;
    ///     assert_eq!(result, 0b00001101);
    ///
    ///     Ok(())
    /// }
    /// # Ruby::init(example).unwrap()
    /// ```
    fn coerce_bit<T, ID, U>(self, other: T, op: ID) -> Result<U, Error>
    where
        T: Numeric,
        ID: IntoId,
        U: TryConvert,
    {
        let op = op.into_id_with(&Ruby::get_with(self));
        protect(|| unsafe {
            Value::new(rb_num_coerce_bit(
                self.as_rb_value(),
                other.as_rb_value(),
                op.as_rb_id(),
            ))
        })
        .and_then(TryConvert::try_convert)
    }
}

/// Wrapper type for a Value known to be an instance of Ruby’s Numeric class.
///
/// See the [`ReprValue`] trait for additional methods available on this type.
///
/// # Examples
///
/// ```
/// use std::num::NonZeroI64;
///
/// use magnus::{Error, Ruby, numeric::NumericValue, prelude::*};
///
/// fn example(ruby: &Ruby) -> Result<(), Error> {
///     let a = ruby.integer_from_i64(1);
///     let b = ruby.rational_new(1, NonZeroI64::new(2).unwrap());
///     let c = ruby.float_from_f64(0.3);
///     let d = ruby.integer_from_i64(4);
///
///     let result: NumericValue = a.coerce_bin(b, "+")?;
///     let result: NumericValue = result.coerce_bin(c, "+")?;
///     let result: NumericValue = result.coerce_bin(d, "+")?;
///     assert_eq!(f64::try_convert(result.as_value())?, 5.8);
///
///     Ok(())
/// }
/// # Ruby::init(example).unwrap()
/// ```
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NumericValue(NonZeroValue);

impl NumericValue {
    #[inline]
    unsafe fn from_rb_value_unchecked(val: VALUE) -> Self {
        unsafe { Self(NonZeroValue::new_unchecked(Value::new(val))) }
    }
}

impl fmt::Display for NumericValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.to_s_infallible() })
    }
}

impl fmt::Debug for NumericValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inspect())
    }
}

impl IntoValue for NumericValue {
    #[inline]
    fn into_value_with(self, _: &Ruby) -> Value {
        self.0.get()
    }
}

unsafe impl private::ReprValue for NumericValue {}

impl Numeric for NumericValue {}

impl ReprValue for NumericValue {}

impl TryConvert for NumericValue {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let handle = Ruby::get_with(val);
        val.is_kind_of(handle.class_numeric())
            .then(|| unsafe { Self::from_rb_value_unchecked(val.as_rb_value()) })
            .ok_or_else(|| {
                Error::new(
                    handle.exception_type_error(),
                    format!("no implicit conversion of {} into Numeric", unsafe {
                        val.classname()
                    },),
                )
            })
    }
}
