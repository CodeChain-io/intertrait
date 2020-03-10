use syn::parse::Result;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Token;
use syn::{Path, Type};

pub struct CastableToArgs {
    pub ty: Type,
    pub traits: Vec<Path>,
}

impl Parse for CastableToArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty: Type = input.parse()?;
        input.parse::<Token![:]>()?;
        let traits = Punctuated::<Path, Token![,]>::parse_terminated(input)?;
        Ok(CastableToArgs {
            ty,
            traits: traits.into_iter().collect(),
        })
    }
}
