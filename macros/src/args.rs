use std::collections::HashSet;

use syn::bracketed;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Error, Ident, Path, Token, Type};

#[derive(Hash, PartialEq, Eq)]
pub enum Flag {
    Sync,
}

impl Flag {
    fn from(ident: &Ident) -> Result<Self> {
        match ident.to_string().as_str() {
            "sync" => Ok(Flag::Sync),
            unknown => {
                let msg = format!("Unknown flag: {}", unknown);
                Err(Error::new_spanned(ident, msg))
            }
        }
    }
}

pub struct Targets {
    pub flags: HashSet<Flag>,
    pub paths: Vec<Path>,
}

impl Parse for Targets {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut flags = HashSet::new();
        let mut paths = Vec::new();

        if input.is_empty() {
            return Ok(Targets { flags, paths });
        }

        if input.peek(syn::token::Bracket) {
            let content;
            bracketed!(content in input);
            for ident in Punctuated::<Ident, Token![,]>::parse_terminated(&content)? {
                if !flags.insert(Flag::from(&ident)?) {
                    let msg = format!("Duplicated flag: {}", ident);
                    return Err(Error::new_spanned(ident, msg));
                }
            }
        }

        if input.is_empty() {
            return Ok(Targets { flags, paths });
        }

        paths = Punctuated::<Path, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect();

        Ok(Targets { flags, paths })
    }
}

pub struct Casts {
    pub ty: Type,
    pub targets: Targets,
    pub intertrait_path: Path,
}

impl Parse for Casts {
    fn parse(input: ParseStream) -> Result<Self> {
        let intertrait_path = if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            input.parse()?
        } else {
            syn::parse(quote::quote!(::intertrait).into())?
        };

        let ty: Type = input.parse()?;
        input.parse::<Token![=>]>()?;
        let targets: Targets = input.parse()?;

        Ok(Casts {
            ty,
            targets,
            intertrait_path,
        })
    }
}
