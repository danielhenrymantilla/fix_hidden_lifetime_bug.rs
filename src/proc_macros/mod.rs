#![allow(nonstandard_style, unused_imports)]

//! Internal crate, do not use it directly.

extern crate proc_macro;

use ::core::{mem, ops::Not as _};
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

use self::{
    append_captures_hack_to_impl_occurrences::append_captures_hack_to_impl_occurrences,
    collect_lifetime_params::collect_lifetime_params,
    manually_unsugar_async::manually_unsugar_async,
    params::Params,
};

mod append_captures_hack_to_impl_occurrences;

mod collect_lifetime_params;

mod manually_unsugar_async;

mod params;

#[cfg(feature = "showme")]
mod showme;

/// <span class="stab portability" title="This is supported on crate feature `proc-macros` only"><code>feature = "proc-macros"</code></span>See [the crate docs](https://docs.rs/fix_hidden_lifetime_bug) for more info.
#[proc_macro_attribute] pub
fn fix_hidden_lifetime_bug (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    let Params {
        krate,
        showme,
    } = parse_macro_input!(attrs);
    let ref krate = krate.unwrap_or_else(|| parse_quote!(
        ::fix_hidden_lifetime_bug
    ));
    match parse_macro_input!(input) {
        | Item::Fn(ItemFn {
            attrs, vis, sig, block,
        }) => {
            let fun = ImplItemMethod {
                attrs, vis, sig, block: *block, defaultness: None,
            };
            fix_fn(krate, fun, None).map(ToTokens::into_token_stream)
        },
        | Item::Impl(impl_) => fix_impl(krate, impl_).map(ToTokens::into_token_stream),
        | _ => Err(Error::new(
            Span::call_site(),
            "expected `fn`, or `impl` block",
        )),
    }
    .map(|output| {
        let _ = showme;
        #[cfg(feature = "showme")] {
            if showme.is_some() {
                showme::pretty_print_tokenstream(&output);
                eprintln!("{}", showme::BANNER);
            }
        }
        output
    })
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

fn fix_fn (
    krate: &'_ Path,
    mut fun: ImplItemMethod,
    outer_scope: Option<&'_ mut ItemImpl>,
) -> Result<ImplItemMethod>
{
    let ref lifetimes = collect_lifetime_params(&mut fun.sig, outer_scope);
    if fun.sig.asyncness.is_some() {
        fun = manually_unsugar_async(krate, fun);
    }
    append_captures_hack_to_impl_occurrences(
        krate,
        lifetimes,
        &mut fun.sig.output,
    );
    Ok(fun)
}

fn fix_impl (
    krate: &'_ Path,
    mut impl_: ItemImpl,
) -> Result<TokenStream2>
{
    if let Some((_, ref trait_, _)) = impl_.trait_ {
        return Err(Error::new_spanned(
            trait_,
            "`#[fix_hidden_lifetime_bug]` does not support traits yet.",
        ));
    }
    let items = mem::replace(&mut impl_.items, vec![]);
    impl_.items = items.into_iter().map(|it| Ok(match it {
        | ImplItem::Method(mut fun) => {
            let mut process_current = false;
            fun.attrs.retain(|attr| (
                attr.path.segments.last().unwrap().ident
                !=
                "fix_hidden_lifetime_bug"
                || {
                    process_current = true;
                    false
                }
            ));
            if process_current {
                fun = fix_fn(krate, fun, Some(&mut impl_))?;
            }
            ImplItem::Method(fun)
        },
        | _ => it,
    })).collect::<Result<_>>()?;
    Ok(impl_.into_token_stream())
}

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
