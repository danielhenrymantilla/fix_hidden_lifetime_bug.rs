#![allow(nonstandard_style, unused_imports)]

//! Internal crate, do not use it directly.

extern crate proc_macro;

use ::core::ops::Not as _;
use ::proc_macro::TokenStream;
use ::proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use ::quote::{
    format_ident,
    quote,
    quote_spanned,
    ToTokens,
};
use ::syn::{*,
    parse::{Parse, Parser, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::VisitMut,
    Result,
};

/* Uncomment to debug panics on parse_quote calls */
// macro_rules! __parse_quote__ {(
//     $($code:tt)*
// ) => (
//     (|| {
//         fn type_of_some<T> (_: Option<T>)
//           -> &'static str
//         {
//             ::core::any::type_name::<T>()
//         }
//         let target_ty = None; if false { return target_ty.unwrap(); }
//         let code = ::quote::quote!( $($code)* );
//         eprintln!(
//             "[{}:{}:{}:parse_quote!]\n  - ty: `{ty}`\n  - code: `{code}`",
//             file!(), line!(), column!(),
//             code = code,
//             ty = type_of_some(target_ty),
//         );
//         ::syn::parse_quote!( #code )
//     })()
// )} use __parse_quote__ as parse_quote;

use self::collect_lifetime_params::*;
mod collect_lifetime_params;

use self::manually_desugar_async::*;
mod manually_desugar_async;

use self::append_captures_hack_to_impl_occurrences::*;
mod append_captures_hack_to_impl_occurrences;

/// See [the crate docs](https://docs.rs/fix_hidden_lifetime_bug) for more info.
#[proc_macro_attribute] pub
fn fix_hidden_lifetime_bug (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    let _: parse::Nothing = parse_macro_input!(attrs);
    match parse_macro_input!(input) {
        | Item::Fn(ItemFn {
            attrs, vis, sig, block,
        }) => {
            let fun = ImplItemMethod {
                attrs, vis, sig, block: *block,
                defaultness: None,
            };
            fix_fn(fun)
        },

        // | Item::Impl(impl_) => fix_impl(impl_),
        // | Item::Trait(trait_) => fix_trait(trait_),
        | _ => Err(Error::new(Span::call_site(), concat!(
            "expected ",
            "`fn`",
            // ", or `impl` block", /* Does not seem to be needed */
            ".",
        ))),
    }
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

fn fix_fn (mut fun: ImplItemMethod)
  -> Result<TokenStream2>
{
    let lifetimes = collect_lifetime_params(&mut fun.sig);
    if fun.sig.asyncness.is_some() {
        fun = manually_desugar_async(fun, &lifetimes);
    }
    append_captures_hack_to_impl_occurrences(
        &lifetimes,
        &mut fun.sig.output,
    );
    Ok(fun.into_token_stream())
}

// fn fix_impl (mut impl_: ItemImpl)
//   -> Result<TokenStream2>
// {
//     let span = impl_.impl_token.span;
//     if let Some((_bang, trait_, _for)) = &impl_.trait_ {
//         return Err(Error::new_spanned(trait_, "Use `#[async_trait]` for this instead."));
//     }
//     Err(Error::new(span, "`#[fix_hidden_lifetime_bug]` does not support this yet."))
// }

// fn fix_trait (mut trait_: ItemTrait)
//   -> Result<TokenStream2>
// {
//     let span = trait_.trait_token.span;
//     Err(Error::new(span, "Use `#[async_trait]` for this instead."))
// }
