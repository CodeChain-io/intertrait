use syn::parse::Result;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Path;
use syn::Token;

pub enum CastToArgs {
    None,
    Traits(Vec<Path>),
}

impl Parse for CastToArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(CastToArgs::None);
        }
        let traits = Punctuated::<Path, Token![,]>::parse_terminated(input)?;
        Ok(CastToArgs::Traits(traits.into_iter().collect()))
    }
}
