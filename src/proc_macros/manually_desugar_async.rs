use super::*;

/// Transform an `async fn` into an equivalent `fn … -> impl Future` signature.
pub(crate)
fn manually_desugar_async (
    mut fun: ImplItemMethod,
    lifetimes: &'_ [Lifetime],
) -> ImplItemMethod
{
    fun.sig.asyncness = None;
    let generics = &mut fun.sig.generics;
    let minimum_lifetime = Lifetime::new(
        "'__async_fut",
        Span::call_site(),
    );
    let idx = generics.lifetimes().count();
    generics.params.insert(idx, parse_quote!( #minimum_lifetime ));
    generics
        .make_where_clause()
        .predicates
        .extend(lifetimes.iter().map(|lt| -> WherePredicate {
            parse_quote!(
                //  ≥
                #lt : #minimum_lifetime
            )
        }))
    ;
    let Ret = match ::core::mem::replace(&mut fun.sig.output, ReturnType::Default) {
        | ReturnType::Type(_arrow, ty) => ty,
        | ReturnType::Default => parse_quote!( () ),
    };
    fun.sig.output = parse_quote!(
        ->
        impl
            #minimum_lifetime +
            ::fix_hidden_lifetime_bug::__::core::future::Future<
                Output = #Ret,
            >
    );
    let self_comma = fun.sig.receiver().map(|_| {
        quote!( self, )
    });
    let self_pat_comma = self_comma.as_ref().map(|_| {
        quote!( _, )
    });
    let each_simple_arg_name =
        (
            (if self_pat_comma.is_some() { 1 } else { 0 })
            ..
            fun.sig.inputs.len()
        )
        .map(|i| format_ident!("__arg_{}", i))
        .collect::<Vec<_>>()
    ;
    let each_pat =
        fun .sig
            .inputs
            .iter_mut()
            .skip(if self_pat_comma.is_some() { 1 } else { 0 })
            .zip(&each_simple_arg_name)
            .map(|(arg, simple_name)| match *arg {
                | FnArg::Typed(PatType {
                    ref mut pat,
                    ..
                }) => ::core::mem::replace(
                    &mut **pat,
                    parse_quote!(#simple_name),
                ),

                | _ => unreachable!(),
            })
    ;
    let body = &fun.block;
    fun.block = parse_quote!({
        async move {
            let (
                #self_pat_comma
                #(#each_pat ,)*
            ) = (
                #self_comma
                #(#each_simple_arg_name ,)*
            );
            #body
        }
    });
    fun
}
