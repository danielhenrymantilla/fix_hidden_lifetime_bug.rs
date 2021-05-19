use super::*;

mod kw {
    ::syn::custom_keyword!(showme);
}

pub(in crate)
struct Params {
    pub(in crate)
    showme: Option<kw::showme>,
}

impl Parse for Params {
    fn parse (input: ParseStream<'_>)
      -> Result<Params>
    {
        let mut showme = None::<kw::showme>;
        while input.is_empty().not() {
            let lookahead = input.lookahead1();
            match () {
                | _case if lookahead.peek(kw::showme) => {
                    if cfg!(not(feature = "showme")) {
                        return Err(input.error("Missing `showme` Cargo feature."));
                    }
                    let prev = showme.replace(input.parse().unwrap());
                    if prev.is_some() {
                        return Err(input.error("Duplicate parameter"));
                    }
                },
                | _extraneous => return Err(lookahead.error()),
            }
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(Params {
            showme,
        })
    }
}
