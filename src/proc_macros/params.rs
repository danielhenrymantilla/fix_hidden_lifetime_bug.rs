use super::*;

mod kw {
    ::syn::custom_keyword!(showme);
}

#[derive(Default)]
pub(in crate)
struct Params {
    pub(in crate)
    showme: Option<kw::showme>,

    pub(in crate)
    krate: Option<Path>,
}

impl Parse for Params {
    fn parse (input: ParseStream<'_>)
      -> Result<Params>
    {
        let mut ret = Params::default();
        while input.is_empty().not() {
            let lookahead = input.lookahead1();
            match () {
                | _case if lookahead.peek(kw::showme) => {
                    if cfg!(not(feature = "showme")) {
                        return Err(input.error("Missing `showme` Cargo feature."));
                    }
                    let prev = ret.showme.replace(input.parse().unwrap());
                    if prev.is_some() {
                        return Err(input.error("Duplicate parameter"));
                    }
                },
                | _case if lookahead.peek(Token![crate]) => {
                    let _: Token![crate] = input.parse().unwrap();
                    let _: Token![=] = input.parse()?;
                    let prev = ret.krate.replace(input.parse()?);
                    if prev.is_some() {
                        return Err(input.error("Duplicate parameter"));
                    }
                },
                | _extraneous => return Err(lookahead.error()),
            }
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(ret)
    }
}
