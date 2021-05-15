#![cfg_attr(feature = "with-specialization", feature(min_specialization))]

pub struct FizzBuzzer;
impl FizzBuzzer {
    pub fn new() -> FizzBuzzer { FizzBuzzer }
}

impl std::fmt::Display for FizzBuzzer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("Inoccuous delegation from derived display impl"))
    }
}

#[cfg(any(feature = "with-override", feature = "with-alternative"))]
use trait_host::ShowDetails;
#[cfg(feature = "with-override")]
impl ShowDetails for FizzBuzzer {
    fn fmt_details(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        #[cfg(feature = "with-specialization")]
        return f.write_fmt(format_args!("Hostile takeover from new ShowDetails impl"));
        #[cfg(not(feature = "with-specialization"))]
        return f.write_fmt(format_args!("Normal takeover from new ShowDetails impl"));
    }
}

#[cfg(all(feature = "with-alternative", not(feature = "with-override")))]
use trait_host::{RefCast, ShowDetailsFromDisplay};
#[cfg(all(feature = "with-alternative", not(feature = "with-override")))]
use delegate::delegate;
#[cfg(all(feature = "with-alternative", not(feature = "with-override")))]
impl ShowDetails for FizzBuzzer {
    delegate! {
        to ShowDetailsFromDisplay::ref_cast(self) {
            fn fmt_details(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
        }
    }
}
