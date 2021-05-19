use super::*;

/// Transform an `async fn` into an equivalent `fn … -> impl Future` signature.
pub(in crate)
fn manually_desugar_async (
    mut fun: ImplItemMethod,
) -> ImplItemMethod
{
    fun.sig.asyncness = None;
    let ref Self_ = fun.sig.receiver().and_then(|it| {
        if let FnArg::Receiver(it) = it {
            Some::<Type>({
                let Self_ = format_ident!("Self", span = it.self_token.span);
                parse_quote!( #Self_ )
            })
        } else {
            None
        }
    });
    let generics = &mut fun.sig.generics;
    let minimum_lifetime = Lifetime::new(
        "'__async_fut",
        Span::call_site(),
    );
    let idx = generics.lifetimes().count();
    generics.params.insert(idx, parse_quote!( #minimum_lifetime ));
    let input_tys = fun.sig.inputs.iter().map(|fn_arg| match *fn_arg {
        | FnArg::Receiver(_) => Self_.as_ref().unwrap(),
        | FnArg::Typed(PatType { ref ty, .. }) => ty,
    });
    generics
        .make_where_clause()
        .predicates
        .extend(input_tys.map(|ty| -> WherePredicate {
            parse_quote!(
                //  ≥
                #ty : #minimum_lifetime
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
    let mut each_simple_arg_name =
        (0 .. fun.sig.inputs.len())
            .map(|i| format_ident!("__arg_{}", i))
            .collect::<Vec<_>>()
    ;
    let mut each_simple_arg_name = &mut each_simple_arg_name[..];
    let mut inputs =
        fun .sig
            .inputs
            .iter_mut()
    ;
    if self_pat_comma.is_some() {
        each_simple_arg_name = &mut each_simple_arg_name[1 ..];
        inputs.by_ref().take(1).for_each(drop);
    }
    let each_pat =
        inputs
            .zip(&mut each_simple_arg_name[..])
            .map(|(arg_pat, simple_name)| match *arg_pat {
                | FnArg::Typed(PatType {
                    ref mut pat,
                    ..
                }) => match **pat {
                    | Pat::Ident(PatIdent { ref ident, .. })
                        if ident.to_string().starts_with("__arg_").not()
                    => {
                        simple_name.clone_from(ident);
                        parse_quote!( _ )
                    },
                    | ref mut it => ::core::mem::replace(
                        it,
                        parse_quote!(#simple_name),
                    ),
                },

                | _ => unreachable!(),
            })
    ;
    let stmts = &fun.block.stmts;
    let comment = if cfg!(feature = "showme") {
        ::quote::quote!(
            "Mention the input vars so that they get captured by the Future";
        )
    } else {
        TokenStream2::new()
    };
    fun.block = parse_quote!({
        async move {
            #comment
            let (
                #self_pat_comma
                #(#each_pat ,)*
            ) = (
                #self_comma
                #(#each_simple_arg_name ,)*
            );
            #(#stmts)*
        }
    });
    fun
}
