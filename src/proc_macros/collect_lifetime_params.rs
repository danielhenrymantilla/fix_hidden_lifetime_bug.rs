use super::*;

/// Identify all the lifetime parameters, introducing new ones to make sure
/// there are no elided lifetime parameters, and then emit the list of all such
/// introduced lifetime parameters.
pub(crate)
fn collect_lifetime_params (
    sig: &'_ mut Signature,
) -> Vec<Lifetime>
{
    let mut lifetimes: Vec<Lifetime> =
        sig .generics
            .lifetimes()
            .map(|&LifetimeDef { ref lifetime, .. }| lifetime.clone())
            .collect()
    ;
    let params = &mut sig.generics.params;
    let mut ns = 0_u16 ..;
    let mut visitor = Visitor {
        new_lifetime_param: |span: Span| -> Lifetime {
            let mut lifetime;
            while {
                // do
                lifetime = Lifetime::new(
                    &format!("'_{}", ns.next().unwrap()),
                    span,
                );
                // while
                lifetimes.contains(&lifetime)
            } {}
            params.insert(lifetimes.len(), parse_quote!(#lifetime));
            lifetimes.push(lifetime.clone());
            lifetime
        },
    };
    sig.inputs.iter_mut().for_each(|arg| visitor.visit_fn_arg_mut(arg));
    lifetimes
}

struct Visitor<NewLifetimeParam : FnMut(Span) -> Lifetime> {
    new_lifetime_param: NewLifetimeParam,
}

impl<NewLifetimeParam : FnMut(Span) -> Lifetime> VisitMut
    for Visitor<NewLifetimeParam>
{
    fn visit_receiver_mut (
        self: &'_ mut Self,
        receiver: &'_ mut Receiver,
    )
    {
        match receiver.reference {
            | Some((ampersand, ref mut elided @ None)) => {
                *elided = Some((self.new_lifetime_param)(ampersand.span()))
            }
            | Some((_ampersand, Some(ref mut elided)))
                if elided.ident == "_"
            => {
                *elided = (self.new_lifetime_param)(elided.span());
            },

            | _ => {},
        }
    }

    fn visit_type_reference_mut (
        self: &'_ mut Self,
        reference: &'_ mut TypeReference,
    )
    {
        visit_mut::visit_type_reference_mut(self, reference);
        match reference.lifetime {
            | ref mut elided @ None => {
                *elided = Some((self.new_lifetime_param)(reference.and_token.span()));
            },
            | Some(ref mut elided)
                if elided.ident == "_"
            => {
                *elided = (self.new_lifetime_param)(elided.span());
            },

            | _ => {},
        }
    }

    fn visit_lifetime_mut (
        self: &'_ mut Self,
        lifetime: &'_ mut Lifetime,
    )
    {
        if lifetime.ident == "_" {
            *lifetime = (self.new_lifetime_param)(lifetime.span());
        }
    }

    /// Skip elided lifetimes that appear within `Fn…` signatures.
    fn visit_path_arguments_mut (
        self: &'_ mut Self,
        path_arguments: &'_ mut PathArguments,
    )
    {
        if let PathArguments::Parenthesized(_) = path_arguments {
            // Do not sub-recurse
            return;
        }
        visit_mut::visit_path_arguments_mut(self, path_arguments);
    }

    fn visit_type_trait_object_mut (
        self: &'_ mut Self,
        dyn_trait: &mut TypeTraitObject,
    )
    {
        // Subrecurse.
        visit_mut::visit_type_trait_object_mut(self, dyn_trait);
        // Is there a `+ 'lifetime` bound?
        if dyn_trait
            .bounds
            .iter()
            .any(|bound| match *bound {
                | TypeParamBound::Trait(_) => false,
                | TypeParamBound::Lifetime(_) => true,
            })
            .not()
        {
            // No `+ 'lifetime` on this trait object; insert one.
            let lt = (self.new_lifetime_param)(dyn_trait.span());
            dyn_trait.bounds.push(TypeParamBound::Lifetime(lt));
        }
    }
}
