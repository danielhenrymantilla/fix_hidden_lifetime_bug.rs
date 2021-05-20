use super::*;

/// Identify all the lifetime parameters, introducing new ones to make sure
/// there are no elided lifetime parameters, and then emit the list of all such
/// lifetime parameters appearing in the function signature.
pub(in crate)
fn collect_lifetime_params (
    sig: &'_ mut Signature,
    outer_scope: Option<&'_ mut ItemImpl>,
) -> Vec<Lifetime>
{
    let mut lifetimes: Vec<Lifetime> =
        sig .generics
            .lifetimes()
            .map(|lt_def| lt_def.lifetime.clone())
            .collect()
    ;
    let mut visitor = Visitor {
        lifetimes: &mut lifetimes,
        next_idx: 0 ..,
        params: &mut sig.generics.params,
    };
    sig.inputs.iter_mut().for_each(|arg| visitor.visit_fn_arg_mut(arg));
    if let Some(item_impl) = outer_scope {
        let mut impl_lifetimes: Vec<Lifetime> =
            item_impl
                .generics
                .lifetimes()
                .map(|lt_def| lt_def.lifetime.clone())
                .collect()
        ;
        let mut visitor = Visitor {
            lifetimes: &mut impl_lifetimes,
            next_idx: visitor.next_idx.next().unwrap() ..,
            params: &mut item_impl.generics.params,
        };
        if let Some((_, ref mut trait_, _)) = item_impl.trait_ {
            // Currently unreachable since we guard against this;
            // but keeping this to avoid forgetting about this should support
            // for trait sbe added.
            visitor.visit_path_mut(trait_);
        }
        visitor.visit_type_mut(&mut *item_impl.self_ty);
        impl_lifetimes.extend(lifetimes);
        impl_lifetimes
    } else {
        lifetimes
    }
}

struct Visitor<'__> {
    lifetimes: &'__ mut Vec<Lifetime>,
    // No more than 64K lifetimes, methinks 🙃
    next_idx: ::core::ops::RangeFrom<u16>,
    params: &'__ mut Punctuated<GenericParam, Token![,]>,
}

impl Visitor<'_> {
    fn new_lifetime_param (self: &'_ mut Self, span: Span)
      -> Lifetime
    {
        let mut lifetime;
        while {
            // do
            lifetime = Lifetime::new(
                &format!("'_{}", self.next_idx.next().unwrap()),
                span,
            );
            // while
            self.lifetimes.contains(&lifetime)
        } {}
        self.params.insert(self.lifetimes.len(), parse_quote!(#lifetime));
        self.lifetimes.push(lifetime.clone());
        lifetime
    }
}

impl VisitMut for Visitor<'_> {
    fn visit_receiver_mut (
        self: &'_ mut Self,
        receiver: &'_ mut Receiver,
    )
    {
        match receiver.reference {
            | Some((ampersand, ref mut elided @ None)) => {
                *elided = Some(self.new_lifetime_param(ampersand.span()))
            }
            | Some((_ampersand, Some(ref mut elided)))
                if elided.ident == "_"
            => {
                *elided = self.new_lifetime_param(elided.span());
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
                *elided = Some(self.new_lifetime_param(reference.and_token.span()));
            },
            | Some(ref mut elided)
                if elided.ident == "_"
            => {
                *elided = self.new_lifetime_param(elided.span());
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
            *lifetime = self.new_lifetime_param(lifetime.span());
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
            let lt = self.new_lifetime_param(dyn_trait.span());
            dyn_trait.bounds.push(TypeParamBound::Lifetime(lt));
        }
    }
}
