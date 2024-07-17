#[::fix_hidden_lifetime_bug::fix_hidden_lifetime_bug]
fn example(_a: &dyn Send) -> impl Sized {
    ()
}
