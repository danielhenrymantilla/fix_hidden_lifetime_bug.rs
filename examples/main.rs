#![allow(unused)]

use ::fix_hidden_lifetime_bug::fix_hidden_lifetime_bug;

#[fix_hidden_lifetime_bug]
async fn foo<'a> (a: &'static (), b: &'_ (), c: &'_ ()) {
    /* … */
}

#[fix_hidden_lifetime_bug]
async fn bar<'a> (_: &(), _: Box<dyn Send>) {
    /* … */
}

#[fix_hidden_lifetime_bug]
fn baz<'a, 'b> (it: &'a mut &'b ()) -> impl 'a + Sized {
    it
}

fn main ()
{}
