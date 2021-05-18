use super::*;

pub
fn append_captures_hack_to_impl_occurrences (
    lifetimes: &'_ [Lifetime],
    output: &'_ mut ReturnType,
)
{
    Visitor {
        lifetimes: lifetimes.iter().map(|lt| parse_quote!(
            ::fix_hidden_lifetime_bug::Captures<#lt>
        )).collect(),
    }
    .visit_return_type_mut(output)
    ;
}

struct Visitor {
    lifetimes: Vec<TypeParamBound>,
}

impl VisitMut for Visitor {
    fn visit_type_impl_trait_mut (
        self: &'_ mut Visitor,
        impl_trait: &'_ mut TypeImplTrait,
    )
    {
        // Sub-recurse
        visit_mut::visit_type_impl_trait_mut(self, impl_trait);
        impl_trait.bounds.extend(self.lifetimes.iter().cloned());
    }
}
