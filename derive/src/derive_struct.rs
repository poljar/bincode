use super::FieldAttribute;
use crate::CrateNameAttribute;
use virtue::generate::Generator;
use virtue::parse::Fields;
use virtue::prelude::*;

pub(crate) struct DeriveStruct {
    pub fields: Fields,
    pub crate_name: CrateNameAttribute,
}

impl DeriveStruct {
    pub fn generate_encode(self, generator: &mut Generator) -> Result<()> {
        let DeriveStruct { fields, crate_name } = self;

        generator
            .impl_for(&crate_name.ty("Encode"))?
            .modify_generic_constraints(|generics, where_constraints| {
                for g in generics.iter_generics() {
                    where_constraints
                        .push_constraint(g, crate_name.ty("Encode"))
                        .unwrap();
                }
            })
            .generate_fn("encode")
            .with_generic_deps("E", [crate_name.ty("enc::Encoder")])
            .with_self_arg(virtue::generate::FnSelfArg::RefSelf)
            .with_arg("encoder", "&mut E")
            .with_return_type(format!(
                "core::result::Result<(), {}::error::EncodeError>",
                crate_name.name
            ))
            .body(|fn_body| {
                for field in fields.names() {
                    if field
                        .attributes()
                        .has_attribute(FieldAttribute::WithSerde)?
                    {
                        fn_body.push_parsed(format!(
                            "::{0}::Encode::encode(&{0}::serde::Compat(&self.{1}), encoder)?;",
                            crate_name.name, field
                        ))?;
                    } else {
                        fn_body.push_parsed(format!(
                            "::{}::Encode::encode(&self.{}, encoder)?;",
                            crate_name.name, field
                        ))?;
                    }
                }
                fn_body.push_parsed("Ok(())")?;
                Ok(())
            })?;
        Ok(())
    }

    pub fn generate_decode(self, generator: &mut Generator) -> Result<()> {
        // Remember to keep this mostly in sync with generate_borrow_decode
        let DeriveStruct { fields, crate_name } = self;

        generator
            .impl_for(crate_name.ty("Decode"))?
            .modify_generic_constraints(|generics, where_constraints| {
                for g in generics.iter_generics() {
                    where_constraints.push_constraint(g, crate_name.ty("Decode")).unwrap();
                }
            })
            .generate_fn("decode")
            .with_generic_deps("D", [crate_name.ty("de::Decoder")])
            .with_arg("decoder", "&mut D")
            .with_return_type(format!("core::result::Result<Self, {}::error::DecodeError>", crate_name.name))
            .body(|fn_body| {
                // Ok(Self {
                fn_body.ident_str("Ok");
                fn_body.group(Delimiter::Parenthesis, |ok_group| {
                    ok_group.ident_str("Self");
                    ok_group.group(Delimiter::Brace, |struct_body| {
                        // Fields
                        // {
                        //      a: bincode::Decode::decode(decoder)?,
                        //      b: bincode::Decode::decode(decoder)?,
                        //      ...
                        // }
                        for field in fields.names() {
                            if field.attributes().has_attribute(FieldAttribute::WithSerde)? {
                                struct_body
                                    .push_parsed(format!(
                                        "{1}: (<{0}::serde::Compat<_> as {0}::Decode>::decode(decoder)?).0,",
                                        crate_name.name,
                                        field
                                    ))?;
                            } else {
                                struct_body
                                    .push_parsed(format!(
                                        "{1}: {0}::Decode::decode(decoder)?,",
                                        crate_name.name,
                                        field
                                    ))?;
                            }
                        }
                        Ok(())
                    })?;
                    Ok(())
                })?;
                Ok(())
            })?;
        Ok(())
    }

    pub fn generate_borrow_decode(self, generator: &mut Generator) -> Result<()> {
        // Remember to keep this mostly in sync with generate_decode
        let DeriveStruct { fields, crate_name } = self;

        generator
            .impl_for_with_lifetimes(crate_name.ty("BorrowDecode"), ["__de"])?
            .modify_generic_constraints(|generics, where_constraints| {
                for g in generics.iter_generics() {
                    where_constraints.push_constraint(g, crate_name.ty("BorrowDecode")).unwrap();
                }
            })
            .generate_fn("borrow_decode")
            .with_generic_deps("D", [crate_name.ty("de::BorrowDecoder<'__de>")])
            .with_arg("decoder", "&mut D")
            .with_return_type(format!("core::result::Result<Self, {}::error::DecodeError>", crate_name.name))
            .body(|fn_body| {
                // Ok(Self {
                fn_body.ident_str("Ok");
                fn_body.group(Delimiter::Parenthesis, |ok_group| {
                    ok_group.ident_str("Self");
                    ok_group.group(Delimiter::Brace, |struct_body| {
                        for field in fields.names() {
                            if field.attributes().has_attribute(FieldAttribute::WithSerde)? {
                                struct_body
                                    .push_parsed(format!(
                                        "{1}: (<{0}::serde::BorrowCompat<_> as {0}::BorrowDecode>::borrow_decode(decoder)?).0,",
                                        crate_name.name,
                                        field
                                    ))?;
                            } else {
                                struct_body
                                    .push_parsed(format!(
                                        "{1}: {0}::BorrowDecode::borrow_decode(decoder)?,",
                                        crate_name.name,
                                        field
                                    ))?;
                            }
                        }
                        Ok(())
                    })?;
                    Ok(())
                })?;
                Ok(())
            })?;
        Ok(())
    }
}
