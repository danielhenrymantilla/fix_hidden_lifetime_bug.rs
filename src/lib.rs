#![no_std]
#![cfg_attr(feature = "nightly",
    feature(doc_cfg),
    cfg_attr(all(), doc = include_str!("../README.md")),
)]

// Fix rendering of `<details><summary>` within bulleted lists:
// Credit for this marvelous hack go to: https://github.com/rust-lang/cargo/issues/331#issuecomment-479847157
#![doc(html_favicon_url = "\">
<style>
summary {
    display: list-item;
}
</style>
<meta name=\"")]

#[doc(inline)]
#[cfg(feature = "proc-macros")]
pub use proc_macros::*;

/// The main hack allowing to mention extra lifetime parameters in an
/// `impl Trait` type without expressing an outlives relationship.
///
/// As mentioned in the issues / GitHub comments referenced in the crate docs,
/// this is the main tool to solve the hidden lifetime bugs.
///
/// Indeed, it is even officially present among the internal helper types that
/// the very Rust compiler uses for itself: [`::rustc_data_structures`](
/// https://doc.rust-lang.org/1.52.1/nightly-rustc/rustc_data_structures/captures/trait.Captures.html)
///
///   - If the above link were not to show the `rustc_data_structures`
///     definition, then feel free to follow the following direct commit-tagged
///     [link](https://github.com/rust-lang/rust/blob/9e5f7d5631b8f4009ac1c693e585d4b7108d4275/compiler/rustc_data_structures/src/captures.rs)
pub
trait Captures<'__> {}

impl<T : ?Sized> Captures<'_> for T {}

/// Same as [`Captures`], but taking a type parameter instead.
///
/// It can be a lazy / convenient way to avoid having to think about lifetimes
/// too much, by using instead:
///
/// ```rust,ignore
///   -> impl 'lt + Trait + MentionsTy<Arg1> + MentionsTy<Arg2> + etc
/// ```
///
/// For instance,
///
/// ```rust
/// # use ::fix_hidden_lifetime_bug::MentionsTy;
/// # use ::core::convert::identity as stuff;
/// #
/// fn baz<'a, 'b> (whatever: &'a mut &'b ())
///   -> impl 'a + Sized //   ↕↕↕↕↕↕↕↕↕↕↕↕↕↕
///              + MentionsTy<&'a mut &'b ()>
/// {
///     stuff(whatever)
/// }
/// ```
pub
trait MentionsTy<__ : ?Sized> {}

impl<T : ?Sized, __ : ?Sized> MentionsTy<__> for T {}

/// For compatibility with the `showme` feature to Just Work™,
/// this re-export *is* (exceptionally) part of the public API 🤯
/// Still hiding it because it'd look weird at first flance 😅
#[doc(hidden)]
pub use ::core;

#[cfg(feature = "proc-macros")]
extern crate proc_macros;
