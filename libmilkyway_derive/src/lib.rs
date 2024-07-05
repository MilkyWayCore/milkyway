extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields, Data};


///
/// Macros for deriving Serializble trait automatically
///
#[proc_macro_derive(Serializable)]
pub fn derive_serializable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let fields = match &input.data {
        syn::Data::Struct(s) => &s.fields,
        _ => panic!("Serializable can only be derived for structs"),
    };

    let serialize_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            result.extend(self.#name.serialize());
        }
    });

    let expanded = quote! {
        impl Serializable for #name {
            fn serialize(&self) -> Serialized {
                let mut result = Serialized::new();
                #(#serialize_fields)*
                result
            }
        }
    };

    TokenStream::from(expanded)
}

///
/// Automatic implementation of Deserializable trait
///
/// # Note
/// Compatible only with #[derive(Serializable)] Serializable trait
/// implementations
///
#[proc_macro_derive(Deserializable)]
pub fn derive_deserializable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let fields = match &input.data {
        syn::Data::Struct(s) => &s.fields,
        _ => panic!("Deserializable can only be derived for structs"),
    };

    let fields_quote = fields.iter().enumerate().map(|(_, f)| {
        let name = f.ident.clone().unwrap().clone();
        quote! {#name,}
    });

    let deserialize_fields = fields.iter().enumerate().map(|(_, f)| {
        let name = &f.ident;
        let ty = &f.ty;

        quote! {
            let result = <#ty as Deserializable>::from_serialized(&serialized[offset..].to_vec());
            if result.is_err(){
                return Err(result.err().unwrap());
            }
            let (field, field_size) = result.unwrap();
            offset += field_size;
            let #name = field;
        }
    });



    let expanded = quote! {
        impl Deserializable for #name {
            fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
                let mut offset = 0;
                #(#deserialize_fields)*

                Ok((Self {
                    #(#fields_quote)*
                }, offset))
            }
        }
    };

    TokenStream::from(expanded)
}

/* Enum serialization/deserialization */
///
/// Enum automatic serialization
///
#[proc_macro_derive(EnumSerializable)]
pub fn derive_enum_serializable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let variants = match &input.data {
        Data::Enum(e) => &e.variants,
        _ => panic!("EnumSerializable can only be derived for enums"),
    };

    let serialize_variants = variants.iter().enumerate().map(|(i, v)| {
        let v_name = &v.ident;
        let idx = i as u8;
        match &v.fields {
            Fields::Unit => quote! {
                #name::#v_name => {
                    result.push(#idx);
                }
            },
            Fields::Unnamed(fields) => {
                let field_serializers = fields.unnamed.iter().enumerate().map(|(j, _)| {
                    let idx = syn::Index::from(j);
                    quote! {
                        result.extend(self.#idx.serialize());
                    }
                });
                quote! {
                    #name::#v_name(ref data) => {
                        result.push(#idx);
                        #(#field_serializers)*
                    }
                }
            },
            Fields::Named(fields) => {
                let field_serializers = fields.named.iter().map(|f| {
                    let f_name = &f.ident;
                    quote! {
                        result.extend(data.#f_name.serialize());
                    }
                });
                quote! {
                    #name::#v_name { ref data } => {
                        result.push(#idx);
                        #(#field_serializers)*
                    }
                }
            },
        }
    });

    let expanded = quote! {
        impl Serializable for #name {
            fn serialize(&self) -> Serialized {
                let mut result = Serialized::new();
                match *self {
                    #(#serialize_variants)*
                }
                result
            }
        }
    };

    TokenStream::from(expanded)
}

///
/// Automatic implementation of Deserializable trait for enums
///
/// # Note
/// Compatible only with #[derive(EnumSerializable)] Serializable trait
/// implementations
///
#[proc_macro_derive(EnumDeserializable)]
pub fn derive_enum_deserializable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let variants = match &input.data {
        Data::Enum(e) => &e.variants,
        _ => panic!("EnumDeserializable can only be derived for enums"),
    };

    let deserialize_variants = variants.iter().enumerate().map(|(i, v)| {
        let v_name = &v.ident;
        let idx = i as u8;
        match &v.fields {
            Fields::Unit => quote! {
                #idx => {
                    Ok((#name::#v_name, 1))
                }
            },
            Fields::Unnamed(fields) => {
                let field_deserializers = fields.unnamed.iter().enumerate().map(|(j, f)| {
                    let ty = &f.ty;
                    let idx = syn::Index::from(j);
                    quote! {
                        let (field_ #idx, field_size) = <#ty as Deserializable>::from_serialized(&serialized[offset..])?;
                        offset += field_size;
                    }
                });
                let field_names = (0..fields.unnamed.len()).map(|j| {
                    let idx = syn::Index::from(j);
                    quote! { field_ #idx }
                });
                quote! {
                    #idx => {
                        let mut offset = 1;
                        #(#field_deserializers)*
                        Ok((#name::#v_name(#(#field_names),*), offset))
                    }
                }
            },
            Fields::Named(fields) => {
                let field_deserializers = fields.named.iter().map(|f| {
                    let f_name = &f.ident;
                    let ty = &f.ty;
                    quote! {
                        let (field_ #f_name, field_size) = <#ty as Deserializable>::from_serialized(&serialized[offset..])?;
                        offset += field_size;
                    }
                });
                let field_names = fields.named.iter().map(|f| {
                    let f_name = &f.ident;
                    quote! { #f_name: field_ #f_name }
                });
                quote! {
                    #idx => {
                        let mut offset = 1;
                        #(#field_deserializers)*
                        Ok((#name::#v_name { #(#field_names),* }, offset))
                    }
                }
            },
        }
    });

    let expanded = quote! {
        impl Deserializable for #name {
            fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
                if serialized.len() < 1 {
                    return Err(SerializationError::LengthError);
                }
                let variant_idx = serialized[0];
                match variant_idx {
                    #(#deserialize_variants,)*
                    _ => Err(SerializationError::InvalidDataError("Invalid enum variant")),
                }
            }
        }
    };

    TokenStream::from(expanded)
}