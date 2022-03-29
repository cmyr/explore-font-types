use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned};

mod parse;

#[proc_macro]
pub fn tables(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as parse::Items);
    match tables_impl(&input.0) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn tables_impl(items: &[parse::Item]) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut code = Vec::new();
    for item in items {
        let item_code = match item {
            parse::Item::Single(item) => generate_item_code(item),
            parse::Item::Group(group) => generate_group(group, items)?,
            parse::Item::RawEnum(raw_enum) => generate_raw_enum(raw_enum),
            parse::Item::Flags(flags) => generate_flags(flags),
        };
        code.push(item_code);
    }
    Ok(quote! {
        #(#code)*
    })
}

fn generate_item_code(item: &parse::SingleItem) -> proc_macro2::TokenStream {
    if !item.has_references() {
        generate_zerocopy_impls(item)
    } else {
        generate_view_impls(item)
    }
}

fn generate_group(
    group: &parse::ItemGroup,
    all_items: &[parse::Item],
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let name = &group.name;
    let lifetime = group.lifetime.as_ref().map(|_| quote!(<B>));
    let docs = &group.docs;
    let shared_getter_impl = if group.generate_getters.is_some() {
        Some(generate_group_getter_impl(group, all_items)?)
    } else {
        None
    };
    let variants = group.variants.iter().map(|variant| {
        let name = &variant.name;
        let typ = &variant.typ;
        let docs = variant.docs.iter();
        let lifetime = variant.typ_lifetime.as_ref().map(|_| quote!(<B>));
        quote! {
                #( #docs )*
                #name(#typ #lifetime)
        }
    });

    let format = &group.format_typ;
    let match_arms = group.variants.iter().map(|variant| {
        let name = &variant.name;
        let version = &variant.version;
        quote! {
            #version => {
                Some(Self::#name(font_types::FontRead::read(bytes)?))
            }
        }
    });

    let var_versions = group
        .variants
        .iter()
        .filter_map(|v| v.version.const_version_tokens());

    // ensure that constants passed in as versions actually exist, and that we
    // aren't just using them as bindings
    let validation_check = quote! {
        #( const _: #format = #var_versions; )*
    };
    let font_read = quote! {

        impl<B: zerocopy::ByteSlice> font_types::FontRead<B> for #name #lifetime {
            fn read(bytes: B) -> Option<Self> {
                #validation_check
                let slice: &[u8] = &*bytes;
                let version: BigEndian<#format> = font_types::FontRead::read(slice)?;
                match version.get() {
                    #( #match_arms ),*

                        other => {
                            eprintln!("unknown enum variant {:?}", version);
                            None
                        }
                }
            }
        }
    };

    Ok(quote! {
        #( #docs )*
        pub enum #name #lifetime {
            #( #variants ),*
        }

        #font_read

        #shared_getter_impl
    })
}

fn generate_group_getter_impl(
    group: &parse::ItemGroup,
    all_items: &[parse::Item],
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let items = group.variants.iter().map(|var|
        all_items
        .iter()
        .find_map(|item|  match item {
            parse::Item::Single(x) if x.name == var.typ => Some(x),
            _ => None,
        })
        .ok_or_else(|| syn::Error::new(var.typ.span(), "type not in scope (#[generate_getters] requires all items in same macro block)")))
        .collect::<Result<Vec<_>,_>>()?;

    let mut fields = HashMap::new();
    for item in &items {
        for field in item.fields.iter().filter(|fld| fld.visible()) {
            fields
                .entry(field.name())
                .or_insert_with(|| (field, Vec::new()))
                .1
                .push(&item.name);
        }
    }

    let match_left_sides = group
        .variants
        .iter()
        .map(|var| {
            let name = &var.name;
            let span = name.span();
            quote_spanned!(span=> Self::#name(_inner))
        })
        .collect::<Vec<_>>();

    let n_variants = group.variants.len();
    let mut getters = Vec::new();

    for (field, variants) in fields.values() {
        let field_name = field.name();
        let docs = field.docs();
        let (ret_type, match_right_sides) = if variants.len() == n_variants {
            let match_right_sides = (0..n_variants)
                .map(|_| quote!(_inner.#field_name()))
                .collect::<Vec<_>>();
            (field.getter_return_type(), match_right_sides)
        } else {
            let ret_type = field.getter_return_type();
            let ret_type = quote!(Option<#ret_type>);
            let match_right_sides = group
                .variants
                .iter()
                .map(|var| {
                    if variants.contains(&&var.typ) {
                        quote!( Some(_inner.#field_name()) )
                    } else {
                        quote!(None)
                    }
                })
                .collect();
            (ret_type, match_right_sides)
        };

        getters.push(quote! {
            #( #docs )*
            pub fn #field_name(&self) -> #ret_type {
                match self {
                    #( #match_left_sides => #match_right_sides, )*
                }
            }
        });
    }

    let name = &group.name;
    let lifetime = group.lifetime.as_ref().map(|_| quote!(<'a>));

    Ok(quote! {
        impl #lifetime #name #lifetime {
            #( #getters )*
        }
    })
}

fn generate_flags(raw: &parse::BitFlags) -> proc_macro2::TokenStream {
    let name = &raw.name;
    let docs = &raw.docs;
    let type_ = &raw.type_;
    let variants = raw.variants.iter().map(|variant| {
        let name = &variant.name;
        let value = &variant.value;
        let docs = &variant.docs;
        quote! {
            #( #docs )*
            const #name = #value;
        }
    });

    quote! {
        bitflags::bitflags! {
            #( #docs )*
            pub struct #name: #type_ {
                #( #variants )*
            }
        }

        impl font_types::Scalar for #name {
            type Raw = <#type_ as font_types::Scalar>::Raw;

            fn to_raw(self) -> Self::Raw {
                self.bits().to_raw()
            }

            fn from_raw(raw: Self::Raw) -> Self {
                let t = <#type_>::from_raw(raw);
                Self::from_bits_truncate(t)
            }
        }
    }
}

fn generate_raw_enum(raw: &parse::RawEnum) -> proc_macro2::TokenStream {
    let name = &raw.name;
    let docs = &raw.docs;
    let repr = &raw.repr;
    let variants = raw.variants.iter().map(|variant| {
        let name = &variant.name;
        let value = &variant.value;
        let docs = &variant.docs;
        quote! {
            #( #docs )*
            #name = #value,
        }
    });
    let variant_inits = raw.variants.iter().map(|variant| {
        let name = &variant.name;
        let value = &variant.value;
        quote!(#value => Self::#name,)
    });

    quote! {
        #( #docs )*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(#repr)]
        pub enum #name {
            #( #variants )*
            Unknown,
        }

        impl #name {
            /// Create from a raw scalar.
            ///
            /// This will never fail; unknown values will be mapped to the `Unknown` variant
            pub fn new(raw: #repr) -> Self {
                match raw {
                    #( #variant_inits )*
                    _ => Self::Unknown,
                }
            }
        }
    }
}

fn generate_zerocopy_impls(item: &parse::SingleItem) -> proc_macro2::TokenStream {
    assert!(item.lifetime.is_none());
    let name = &item.name;
    let field_names = item
        .fields
        .iter()
        .map(|field| &field.as_single().unwrap().name);
    let docs = &item.docs;
    let field_types = item
        .fields
        .iter()
        .map(|field| &field.as_single().unwrap().typ);
    let field_docs = item
        .fields
        .iter()
        .map(|fld| {
            let docs = &fld.as_single().unwrap().docs;
            quote!( #( #docs )* )
        })
        .collect::<Vec<_>>();

    let getters = item.fields.iter().map(parse::Field::view_getter_fn);

    quote! {
        #( #docs )*
        #[derive(Clone, Copy, Debug, zerocopy::FromBytes, zerocopy::Unaligned)]
        #[repr(C)]
        pub struct #name {
            #( #field_docs pub #field_names: #field_types, )*
        }

        impl #name {
            #(
                #field_docs
                #getters
            )*
        }

    }
}

fn generate_view_impls(item: &parse::SingleItem) -> proc_macro2::TokenStream {
    let name = &item.name;
    let docs = &item.docs;

    // these are fields which are inputs to the size calculations of *other fields.
    // For each of these field, we generate a 'resolved' identifier, and we assign the
    // value of this field to that identifier at init time. Subsequent fields can then
    // access that identifier in their own generated code, to determine the runtime
    // value of something.
    #[allow(clippy::needless_collect)] // bad clippy
    let fields_used_as_inputs = item
        .fields
        .iter()
        .filter_map(parse::Field::as_array)
        .flat_map(|array| array.count.iter_input_fields())
        .collect::<Vec<_>>();

    // The fields in the declaration of the struct.
    let mut field_decls = Vec::new();
    // the getters for those fields that have getters
    let mut getters = Vec::new();
    // just the names; we use these at the end to assemble the struct (shorthand initializer syntax)
    let mut used_field_names = Vec::new();
    // the code to intiailize each field. each block of code is expected to take the form,
    // `let (field_name, bytes) = $expr`, where 'bytes' is the whatever is left over
    // from the input bytes after validating this field.
    let mut field_inits = Vec::new();

    for field in &item.fields {
        if matches!(field, parse::Field::Array(arr) if arr.variable_size.is_some()) {
            continue;
        }

        let name = field.name();
        let span = name.span();

        field_decls.push(field.view_field_decl());
        used_field_names.push(field.name().to_owned());

        if let Some(getter) = field.view_getter_fn() {
            getters.push(getter);
        }

        let field_init = field.view_init_expr();
        // if this field is used by another field, resolve it's current value
        let maybe_resolved_value = fields_used_as_inputs.contains(&name).then(|| {
            let resolved_ident = make_resolved_ident(name);
            let resolve_expr = field.resolve_expr();
            quote_spanned!(span=> let #resolved_ident = #resolve_expr;)
        });

        field_inits.push(quote! {
            #field_init
            #maybe_resolved_value
        });
    }

    let mut offset_host_impl = None;
    if let Some(attr) = item.offset_host.as_ref() {
        let span = attr.span();
        field_decls.push(quote_spanned!(span=> offset_bytes: font_types::OffsetData<B>));
        used_field_names.push(syn::Ident::new("offset_bytes", span));
        // we stash the starting length of the bytes before we read anything,
        // and use this to calculate where the offests start
        field_inits.insert(
            0,
            quote_spanned!(span=> let __initial_byte_len = bytes.len();),
        );
        field_inits.push(quote_spanned! {span=>
            let offset_start = __initial_byte_len - bytes.len();
            let offset_bytes = font_types::OffsetData::new(bytes, offset_start);
        });
        offset_host_impl = Some(quote_spanned! {span=>
            impl<'a, B: zerocopy::ByteSlice + 'a> font_types::OffsetHost2<'a, B> for #name<B> {
                fn data(&self) -> &font_types::OffsetData<B> {
                    &self.offset_bytes
                }
            }
        });
    } else {
        //let span = name.span();
        //field_inits.push(quote_spanned!(span=> let _ = bytes; ));
    }

    let name_span = name.span();
    let init_body = quote_spanned! {name_span=>
        #( #field_inits )*
        Some(#name {
            #( #used_field_names, )*
        })
    };

    let init_impl = if item.init.is_empty() {
        quote! {
            impl<B: zerocopy::ByteSlice> font_types::FontRead<B> for #name<B> {
                fn read(bytes: B) -> Option<Self> {
                    #init_body
                }
            }
        }
    } else {
        let init_args = item.init.iter().map(|arg| {
            let span = arg.span();
            quote_spanned!(span=> #arg: usize)
        });
        let init_aliases = item.init.iter().map(|arg| {
            let span = arg.span();
            let resolved = make_resolved_ident(arg);
            quote_spanned!(span=> let #resolved = #arg;)
        });

        quote_spanned! {name_span=>
            impl<B: zerocopy::ByteSlice> #name<B> {
                pub fn read(bytes: B, #( #init_args ),* ) -> Option<Self> {
                    #( #init_aliases )*
                    #init_body
                }
            }
        }
    };

    quote_spanned! {name_span=>
        #( #docs )*
        pub struct #name<B> {
            #( #field_decls ),*
        }

        #init_impl
        impl<B: zerocopy::ByteSlice> #name<B> {
            #( #getters )*
        }

        #offset_host_impl
    }
}

fn make_resolved_ident(ident: &syn::Ident) -> syn::Ident {
    quote::format_ident!("__resolved_{}", ident)
}
