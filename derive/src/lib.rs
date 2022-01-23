mod derive_enum;
mod derive_struct;

use virtue::{prelude::*, utils::parse_tagged_attribute};

#[proc_macro_derive(Encode, attributes(bincode))]
pub fn derive_encode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_encode_inner(input).unwrap_or_else(|e| e.into_token_stream())
}

fn derive_encode_inner(input: TokenStream) -> Result<TokenStream> {
    let parse = Parse::new(input)?;
    let (mut generator, attributes, body) = parse.into_generator();
    let crate_name = attributes
        .get_attribute::<CrateNameAttribute>()?
        .unwrap_or_default();

    match body {
        Body::Struct(body) => {
            derive_struct::DeriveStruct {
                fields: body.fields,
                crate_name,
            }
            .generate_encode(&mut generator)?;
        }
        Body::Enum(body) => {
            derive_enum::DeriveEnum {
                variants: body.variants,
                crate_name,
            }
            .generate_encode(&mut generator)?;
        }
    }

    let name = generator.target_name().clone();
    let stream = generator.finish()?;
    dump_output(name, "Encode", &stream);
    Ok(stream)
}

#[proc_macro_derive(Decode, attributes(bincode))]
pub fn derive_decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_decode_inner(input).unwrap_or_else(|e| e.into_token_stream())
}

fn derive_decode_inner(input: TokenStream) -> Result<TokenStream> {
    let parse = Parse::new(input)?;
    let (mut generator, attributes, body) = parse.into_generator();
    let crate_name = attributes
        .get_attribute::<CrateNameAttribute>()?
        .unwrap_or_default();

    match body {
        Body::Struct(body) => {
            derive_struct::DeriveStruct {
                fields: body.fields,
                crate_name,
            }
            .generate_decode(&mut generator)?;
        }
        Body::Enum(body) => {
            derive_enum::DeriveEnum {
                variants: body.variants,
                crate_name,
            }
            .generate_decode(&mut generator)?;
        }
    }

    let name = generator.target_name().clone();
    let stream = generator.finish()?;
    dump_output(name, "Decode", &stream);
    Ok(stream)
}

#[proc_macro_derive(BorrowDecode, attributes(bincode))]
pub fn derive_brrow_decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_borrow_decode_inner(input).unwrap_or_else(|e| e.into_token_stream())
}

fn derive_borrow_decode_inner(input: TokenStream) -> Result<TokenStream> {
    let parse = Parse::new(input)?;
    let (mut generator, attributes, body) = parse.into_generator();
    let crate_name = attributes
        .get_attribute::<CrateNameAttribute>()?
        .unwrap_or_default();

    match body {
        Body::Struct(body) => {
            derive_struct::DeriveStruct {
                fields: body.fields,
                crate_name,
            }
            .generate_borrow_decode(&mut generator)?;
        }
        Body::Enum(body) => {
            derive_enum::DeriveEnum {
                variants: body.variants,
                crate_name,
            }
            .generate_borrow_decode(&mut generator)?;
        }
    }

    let name = generator.target_name().clone();
    let stream = generator.finish()?;
    dump_output(name, "BorrowDecode", &stream);
    Ok(stream)
}

fn dump_output(name: Ident, derive: &str, stream: &TokenStream) {
    use std::io::Write;

    if let Ok(var) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(var);
        loop {
            {
                let mut path = path.clone();
                path.push("target");
                if path.exists() {
                    path.push(format!("{}_{}.rs", name, derive));
                    if let Ok(mut file) = std::fs::File::create(path) {
                        let _ = file.write_all(stream.to_string().as_bytes());
                    }
                    break;
                }
            }
            if let Some(parent) = path.parent() {
                path = parent.to_owned();
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum FieldAttribute {
    WithSerde,
}

impl FromAttribute for FieldAttribute {
    fn parse(group: &Group) -> Result<Option<Self>> {
        let body = match virtue::utils::parse_tagged_attribute(group, "bincode") {
            Some(body) => body,
            None => return Ok(None),
        };
        match body.into_iter().next() {
            Some(TokenTree::Ident(ident)) if ident.to_string() == "with_serde" => {
                Ok(Some(Self::WithSerde))
            }
            token => Err(virtue::Error::custom_at_opt_token(
                "Unknown attribute, expected one of: \"with_serde\"",
                token,
            )),
        }
    }
}

pub(crate) struct CrateNameAttribute {
    pub name: String,
}

impl CrateNameAttribute {
    pub fn ty(&self, ty: &str) -> String {
        format!("::{}::{}", self.name, ty)
    }
}

impl Default for CrateNameAttribute {
    fn default() -> Self {
        Self {
            name: String::from("bincode"),
        }
    }
}

impl FromAttribute for CrateNameAttribute {
    fn parse(group: &Group) -> Result<Option<Self>> {
        let stream: TokenStream = match parse_tagged_attribute(group, "bincode") {
            None => return Ok(None),
            Some(s) => s,
        };
        let mut iter = stream.into_iter();
        while let Some(item) = iter.next() {
            if let TokenTree::Ident(ident) = item {
                if ident.to_string() == "crate" {
                    try_consume_char(&mut iter, '=')?;
                    return match iter.next() {
                        Some(TokenTree::Literal(lit)) => {
                            let crate_name = lit.to_string();
                            if !crate_name.starts_with('"') && !crate_name.ends_with('"') {
                                Err(virtue::Error::Custom {
                                    error: format!("Expected string, found {:?}", lit),
                                    span: Some(lit.span()),
                                })
                            } else {
                                Ok(Some(Self {
                                    name: crate_name[1..crate_name.len() - 1].to_string(),
                                }))
                            }
                        }
                        t => Err(virtue::Error::Custom {
                            error: format!("Expected crate name, found {:?}", t),
                            span: t.map(|t| t.span()),
                        }),
                    };
                }
            }
        }

        Ok(None)
    }
}

fn try_consume_char(iter: &mut impl Iterator<Item = TokenTree>, char: char) -> Result<()> {
    match iter.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == char => Ok(()),
        t => Err(virtue::Error::Custom {
            error: format!("Expected `key = val`, found {:?}", t),
            span: t.map(|t| t.span()),
        }),
    }
}
