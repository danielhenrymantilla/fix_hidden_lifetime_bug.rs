/// Make `::fix_hidden_lifetime_bug` not work as expected.
extern crate core as fix_hidden_lifetime_bug;

/// But luckily, we can refer to that crate from another path:
/// `crate::module::fix_hidden_lifetime_bug`.
mod module {
    pub
    extern crate fix_hidden_lifetime_bug;
}

// It's obviously a mouthful, but it's just there to be friendly to
// other macros.
#[module::fix_hidden_lifetime_bug::fix_hidden_lifetime_bug(
    crate = crate::module::fix_hidden_lifetime_bug,
)]
async
fn foo (_: &(), _: &())
{}
