use super::*;

/// Transform an `async fn` into an equivalent `fn … -> impl Future` signature.
pub(in crate)
fn manually_unsugar_async (
    krate: &'_ Path,
    mut fun: ImplItemMethod,
) -> ImplItemMethod
{
    // 1) Remove the `async` keyword to get a normal `fn`.
    fun.sig.asyncness = None;

    // 2) Introduce a new "free" lifetime parameter, which will be the `'lt`
    // in `-> impl 'lt + Future…`
    // For such a lifetime to work (for the body of the function to compile)
    // we need the types of each parameter (captured by the future) to
    // be "usable within that `'lt`", _.i.e_, we need `ArgTy : 'lt` to hold
    // for each `ArgTy` (including `Self`).
    let ref Self_: Option<Type> = fun.sig.receiver().and_then(|it| {
        // If there is a receiver *and* if it is in the shorthand form,
        // we need to forge a `Self` type ourselves.
        if let FnArg::Receiver(rc) = it {
            Some({
                let Self_ = format_ident!("Self", span = rc.self_token.span);
                // If `Some`, it bundles a named lifetime, by construction.
                if let Some((ref amp, ref lt)) = rc.reference {
                    parse_quote!(
                        #amp #lt #Self_
                    )
                } else {
                    parse_quote!(
                        #Self_
                    )
                }
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

    // 3) Replace `-> RetTy` with `-> impl 'lt + Future<Output = RetTy>`
    // (Note: no need to worry about auto-traits here, since they leak anyways)
    let Ret = match mem::replace(&mut fun.sig.output, ReturnType::Default) {
        | ReturnType::Type(_arrow, ty) => ty,
        | ReturnType::Default => parse_quote!( () ),
    };
    fun.sig.output = parse_quote!(
        -> impl
            #minimum_lifetime +
            #krate::core::future::Future<
                Output = #Ret,
            >
    );

    // 4) In the to-be-generated `async move { … }` function body, we need to
    // make sure all the function parameters are captured by it.
    // In order to do that, we emit a dummy `let _ = (arg1, arg2, …);` statement
    // at the beginning of that `async move` block.
    //
    // That being said, if the user fed to the macro some more exotic patterns,
    // such as `async fn f(_: Arg) {}`: we cannot emit `let _ = _;`!
    // (and even if we could, that would not capture that variable).
    // So this gets a bit more annoying: we roll our own set of fallback var
    // names, `__arg_0`, `__arg_1`, …, and replace such patterns with it.
    // Then, to go back to the arg semantics that the user wrote, we change the
    // LHS of the `let` to be the replaced pattern.
    let capture_args_stmt = {
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
                    }) => {
                        simple_name.set_span(pat.span());
                        match **pat {
                            | Pat::Ident(PatIdent { ref ident, .. })
                                if ident.to_string().starts_with("__arg_").not()
                            => {
                                simple_name.clone_from(ident);
                                (**pat).clone()
                            },
                            | ref mut it => mem::replace(
                                it,
                                parse_quote!(#simple_name),
                            ),
                        }
                    },

                    | _ => unreachable!(),
                })
        ;
        // A comment for people looking at the output of `showme` to understand
        // what is going on:
        let comment = if cfg!(feature = "showme") {
            ::quote::quote!(
                "Mention the input vars so that they get captured by the Future";
            )
        } else {
            TokenStream2::new()
        };
        quote!(
            #comment
            let (
                #self_pat_comma #(#each_pat ,)*
            ) = (
                #self_comma     #(#each_simple_arg_name ,)*
            );
        )
    };

    // 5) And voilà! 🙂
    let stmts = &fun.block.stmts;
    fun.block = parse_quote!({
        async move {
            #capture_args_stmt
            #(#stmts)*
        }
    });
    fun
}
