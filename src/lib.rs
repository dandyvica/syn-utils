// this will help managing enum variants
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, Expr, Fields, FieldsNamed, FieldsUnnamed, Variant};

pub trait VariantHelper {
    // true if the variant is a unit-like variant
    // Ok = 1
    fn is_unit(&self) -> bool;

    // Some(_) if variant has named fields
    // Move {
    //     x: u16,
    //     y: u16,
    // },
    fn is_named(&self) -> Option<&FieldsNamed>;

    // Some(_) if variant has unnamed fields
    // ChangeColor(u16, u16, u16),
    fn is_unnamed(&self) -> Option<&FieldsUnnamed>;

    // check whether the variant has the attribute 'attr'
    fn has_attribute<'a>(&'a self, attr: &str) -> Option<&'a Attribute>;

    // get the literal value of a unit variant
    fn literal(&self) -> TokenStream;
}

impl VariantHelper for Variant {
    fn is_unit(&self) -> bool {
        self.fields == Fields::Unit
    }

    fn is_named(&self) -> Option<&FieldsNamed> {
        if let Fields::Named(f) = &self.fields {
            return Some(f);
        }
        None
    }

    fn is_unnamed(&self) -> Option<&FieldsUnnamed> {
        if let Fields::Unnamed(f) = &self.fields {
            return Some(f);
        }
        None
    }

    fn has_attribute<'a>(&'a self, attr: &str) -> Option<&'a Attribute> {
        self.attrs.iter().find(|a| a.path().is_ident(attr))
    }

    fn literal(&self) -> TokenStream {
        // extract the litteral value of the variant. Ex: Ok = 0
        let value = self.discriminant.as_ref().unwrap_or_else(|| {
            unimplemented!("discriminant for variant {} is not a litteral", self.ident)
        });

        value.1.to_token_stream()
    }
}

// gather all global function under this umbrella
pub struct SynUtils;

impl SynUtils {
    // return the internals of the repr attribute
    // #[repr(u8)] => Some(u8)
    pub fn repr_size(attrs: &[Attribute]) -> Option<proc_macro2::TokenStream> {
        let mut ty = None;

        for attr in attrs {
            if attr.path().is_ident("repr") {
                if let Ok(expr) = attr.parse_args::<Expr>() {
                    ty = Some(expr.to_token_stream());
                }
            }
        }

        ty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn enum_opcode_reserved() {
        let e: syn::ItemEnum = parse_quote!(
            #[repr(u16)]
            enum OpCode {
                Query = 0,  //[RFC1035]
                IQuery = 1, // (Inverse Query, OBSOLETE)	[RFC3425]
                Status = 2, // [RFC1035]
                Unassigned = 3,
                Notify = 4, // [RFC1996]
                Update = 5, // [RFC2136]
                DOS = 6,    // DNS Stateful Operations (DSO)	[RFC8490]
            }
        );

        let repr_size = SynUtils::repr_size(&e.attrs);
        assert!(repr_size.is_some());
        assert_eq!(&repr_size.unwrap().to_string(), "u16");
    }

    #[test]
    fn enum_message() {
        let e: syn::ItemEnum = parse_quote!(
            enum Message {
                Ok = 0,

                #[foo]
                Quit = 1,
                Move {
                    x: u16,
                    y: u16,
                },
                Write(String),
                ChangeColor(u16, u16, u16),
            }
        );

        for v in e.variants.iter() {
            match v.ident.to_string().as_str() {
                "Ok" => {
                    assert!(v.is_unit());
                    assert!(v.has_attribute("foo").is_none());
                    assert_eq!(v.literal().to_string(), "0");
                }
                "Quit" => {
                    assert!(v.is_unit());
                    assert!(v.has_attribute("foo").is_some());
                    assert_eq!(v.literal().to_string(), "1");                    
                }
                "Move" => {
                    assert!(v.is_named().is_some());
                    assert!(!v.has_attribute("foo").is_some());
                }
                "Write" => {
                    assert!(v.is_unnamed().is_some());
                    assert!(!v.has_attribute("foo").is_some());
                }
                _ => (),
            }
        }
    }
}
