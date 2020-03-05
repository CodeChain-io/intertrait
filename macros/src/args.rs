use syn::parse::Result;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Path;
use syn::Token;

pub enum Args {
    None,
    Traits(Vec<Path>),
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Args::None);
        }
        let traits = Punctuated::<Path, Token![,]>::parse_terminated(input)?;
        Ok(Args::Traits(traits.into_iter().collect()))
    }
}
