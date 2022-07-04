use super::*;

pub
fn append_captures_hack_to_impl_occurrences (
    krate: &'_ Path,
    lifetimes: &'_ [Lifetime],
    output: &'_ mut ReturnType,
)
{
    // use a visitor to make sure _all_ the `impl` occurrences are changed.
    Visitor { krate, lifetimes }.visit_return_type_mut(output);
}

struct Visitor<'__> {
    krate: &'__ Path,
    lifetimes: &'__ [Lifetime],
}

impl VisitMut for Visitor<'_> {
    fn visit_type_impl_trait_mut (
        self: &'_ mut Self,
        impl_trait: &'_ mut TypeImplTrait,
    )
    {
        // Sub-recurse
        visit_mut::visit_type_impl_trait_mut(self, impl_trait);
        let lifetimes_to_add = self.lifetimes.iter().cloned();
        // This is a micro-optimization for when `showme` is requested, so
        // as to remove superfluous mentions of lifetime parameters:
        // they don't technically hurt the definition, but they do hinder
        // a bit the human readability.
        #[cfg(feature = "showme")]
        let lifetimes_to_add = lifetimes_to_add.filter({
            let lifetimes_to_skip = find_all_lifetimes_in_impl_trait(impl_trait);
            move |lt| lifetimes_to_skip.contains(lt).not()
        });
        let krate = self.krate;
        impl_trait.bounds.extend(lifetimes_to_add.map(|lt| -> TypeParamBound {
            parse_quote!(
                #krate::Captures<#lt>
            )
        }));
    }
}

#[cfg(feature = "showme")]
fn find_all_lifetimes_in_impl_trait (
    impl_trait: &'_ mut TypeImplTrait,
) -> Vec<Lifetime>
{
    let mut lifetimes_seen = vec![];
    Visitor(&mut lifetimes_seen)
        .visit_type_impl_trait_mut(impl_trait)
    ;
    return lifetimes_seen;
    // where:
    struct Visitor<'__> (
        /// The set of encountered lifetimes.
        ///
        /// Size is small enough for a `Vec` with dups to still be better than
        /// a `Set`.
        &'__ mut Vec<Lifetime>,
    );
    impl VisitMut for Visitor<'_> {
        // This could technically be using `Visit`, but we still would be
        // unable to just use borrows, and it requires enabling yet another
        // `syn` feature, so we don't do it.
        fn visit_lifetime_mut (
            self: &'_ mut Self,
            lifetime: &'_ mut Lifetime,
        )
        {
            self.0.push(lifetime.clone());
        }

        fn visit_trait_bound_mut (
            self: &'_ mut Self,
            trait_bound: &'_ mut TraitBound,
        )
        {
            // sub-recurse
            visit_mut::visit_trait_bound_mut(self, trait_bound);
            #[cfg(FALSE)] { // Not needed for our actual usage of the visitor,
                            // but a more general one might not have that chance
                if let Some(BoundLifetimes { ref lifetimes, .. })
                    = trait_bound.lifetimes
                {
                    // Now strip the lifetimes that originated from `for` bounds:
                    self.0.retain(|lt| {
                        lifetimes
                            .iter()
                            .all(|for_| for_.lifetime != *lt)
                    })
                }
            }
        }
    }
}
