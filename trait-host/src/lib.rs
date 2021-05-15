#![feature(min_specialization)]

use std::fmt;
use std::fmt::Display;
/// A simple trait that can be specialized and has a trivial
/// default implementation using Into<U>.
pub trait ShowDetails {
    fn fmt_details(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

#[cfg(not(feature = "with-alternative"))]
impl<T> ShowDetails for T where T: Display {
    default fn fmt_details(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

#[cfg(feature = "with-alternative")]
pub use ref_cast::RefCast;

#[cfg(feature = "with-alternative")]
#[derive(RefCast)]
#[repr(transparent)]
pub struct ShowDetailsFromDisplay<T>(T);

#[cfg(feature = "with-alternative")]
impl<T> ShowDetails for ShowDetailsFromDisplay<T> where T: Display {
    fn fmt_details(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <T as Display>::fmt(&self.0, f)
    }
}
