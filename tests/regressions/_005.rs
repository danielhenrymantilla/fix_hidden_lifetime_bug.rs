use ::fix_hidden_lifetime_bug::fix_hidden_lifetime_bug;

#[fix_hidden_lifetime_bug]
async fn baz<'a> (a: &'static (), b: &'_ (), url: String)
{
    consume(url);
}

fn consume (u: String)
{
    println!("{}", u);
}
