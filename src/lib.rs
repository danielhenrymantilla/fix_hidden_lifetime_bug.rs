#![no_std]
#![cfg_attr(doc,
    feature(external_doc),
    doc(include = "../README.md"),
)]

extern crate proc_macros;

#[doc(no_inline)]
pub use ::proc_macros::*;

pub
trait Captures<'__> {}

impl<T : ?Sized> Captures<'_> for T {}

#[doc(hidden)] /** Not part of the public API! */ pub
mod __ {
    pub use ::core;
}
