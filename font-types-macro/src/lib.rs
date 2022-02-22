use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod parse;

#[proc_macro]
pub fn tables(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as parse::Items);
    let code = input.iter().map(|item| match item {
        parse::Item::Single(item) => generate_item_code(item),
        parse::Item::Group(group) => generate_group(group),
        parse::Item::RawEnum(raw_enum) => generate_raw_enum(raw_enum),
    });
    quote! {
        #(#code)*
    }
    .into()
}

fn generate_item_code(item: &parse::SingleItem) -> proc_macro2::TokenStream {
    if item.fields.iter().all(|x| x.is_single()) {
        generate_zerocopy_impls(item)
    } else {
        generate_view_impls(item)
    }
}

fn generate_group(group: &parse::ItemGroup) -> proc_macro2::TokenStream {
    let name = &group.name;
    let lifetime = group.lifetime.as_ref().map(|_| quote!(<'a>));
    let docs = &group.docs;
    let variants = group.variants.iter().map(|variant| {
        let name = &variant.name;
        let typ = &variant.typ;
        let docs = variant.docs.iter();
        let lifetime = variant.typ_lifetime.as_ref().map(|_| quote!(<'a>));
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

    let var_versions = group.variants.iter().map(|v| &v.version);

    // make sure this is a constant and we aren't accidentally aliasing?
    // I'm not sure if this is necessary.
    let validation_check = quote! {
        #( const _: #format = #var_versions; )*
    };
    let font_read = quote! {

        impl<'a> font_types::FontRead<'a> for #name #lifetime {
            fn read(bytes: &'a [u8]) -> Option<Self> {
                #validation_check
                let version: BigEndian<#format> = font_types::FontRead::read(bytes)?;
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

    quote! {
        #( #docs )*
        pub enum #name #lifetime {
            #( #variants ),*
        }

        #font_read
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
        .map(|field| field.as_single().unwrap().raw_type_tokens());
    let field_docs = item
        .fields
        .iter()
        .map(|fld| {
            let docs = &fld.as_single().unwrap().docs;
            quote!( #( #docs )* )
        })
        .collect::<Vec<_>>();

    let getters = item
        .fields
        .iter()
        .map(|fld| generate_zc_getter(fld.as_single().unwrap()));

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

fn generate_zc_getter(field: &parse::SingleField) -> proc_macro2::TokenStream {
    let name = &field.name;
    let cooked_type = field.cooked_type_tokens();
    if field.is_be_wrapper() {
        quote! {
            pub fn #name(&self) -> #cooked_type {
                self.#name.get()
            }
        }
    } else {
        quote! {
            pub fn #name(&self) -> &#cooked_type {
                &self.name
            }
        }
    }
}

fn generate_view_impls(item: &parse::SingleItem) -> proc_macro2::TokenStream {
    // scalars only get getters? that makes 'count' and friends complicated...
    // we can at least have a 'new' method that does a reasonable job of bounds checking,
    // but then we're going to be unsafing all over. that's also maybe okay though

    let name = &item.name;
    let docs = &item.docs;

    let mut current_offset = quote!(0);
    let mut in_checked_range = true;
    // some fields do not know their length. If we encounter one of those fields,
    // we set this to the initial offset of that field; otherwise we set this
    // to the final calculated offset.
    // We verify that the input data is at least this long when we construct our
    // object, and any fields in this checked data can use get_unchecked in their getters
    let mut checkable_len = None;
    let mut getters = Vec::new();

    for field in &item.fields {
        match field {
            parse::Field::Single(scalar) => {
                if scalar.hidden.is_none() {
                    getters.push(make_scalar_getter(
                        scalar,
                        &mut current_offset,
                        in_checked_range,
                    ));
                } else {
                    let len = scalar.len_tokens();
                    current_offset = quote!( #current_offset + #len );
                }
            }
            parse::Field::Array(array) => {
                if checkable_len.is_none() {
                    checkable_len = Some(quote!(#current_offset));
                    in_checked_range = false;
                }
                let getter = if array.variable_size.is_none() {
                    make_array_getter(array, &mut current_offset, in_checked_range)
                } else {
                    make_var_array_getter(array, &mut current_offset)
                };
                getters.push(getter);
            }
        }
    }
    if checkable_len.is_none() {
        checkable_len = Some(current_offset);
    }
    let field_docs = item.fields.iter().map(|field| {
        let docs = field.docs();
        quote!( #( #docs )* )
    });

    quote! {
        #( #docs )*
        pub struct #name<'a>(&'a [u8]);

        impl<'a> font_types::FontRead<'a> for #name<'a> {
            fn read(bytes: &'a [u8]) -> Option<Self> {
                if bytes.len() < #checkable_len {
                    return None;
                }
                Some(Self(bytes))
            }
        }

        impl<'a> #name<'a> {
            #( #field_docs #getters )*
        }
    }
}

fn make_scalar_getter(
    field: &parse::SingleField,
    offset: &mut proc_macro2::TokenStream,
    checked: bool,
) -> proc_macro2::TokenStream {
    let name = &field.name;
    let len = field.len_tokens();

    let get_bytes = if checked {
        quote!(unsafe { self.0.get_unchecked(#offset..#offset + #len) })
    } else {
        quote!(self.0.get(#offset..#offset + #len).unwrap_or_default())
    };

    *offset = quote!(#offset + #len);
    let return_ty = field.cooked_type_tokens();

    let from_bytes = field.getter_tokens(&get_bytes);
    if checked {
        quote! {
        pub fn #name(&self) -> #return_ty {
            let raw = #from_bytes.unwrap();
            raw
        }
        }
    } else {
        quote! {
            pub fn #name(&self) -> Option<#return_ty> {
                #from_bytes
            }
        }
    }
}

fn make_array_getter(
    field: &parse::ArrayField,
    offset: &mut proc_macro2::TokenStream,
    _checked: bool,
) -> proc_macro2::TokenStream {
    let name = &field.name;
    let start_off = offset.clone();
    let inner_typ = &field.inner_typ;
    assert!(
        field.inner_lifetime.is_none(),
        "inner_lifetime should only exist on variable size fields"
    );
    let len = match &field.count {
        parse::Count::Field(name) => Some(quote!(self.#name() as usize)),
        parse::Count::Function { fn_, args } => {
            let args = args.iter().map(|arg| quote!(self.#arg()));
            Some(quote!(#fn_( #( #args ),* )))
        }
        parse::Count::Literal(lit) => Some(quote! { (#lit as usize) }),
        parse::Count::All(_) => None,
    };

    let range = match len {
        Some(len) => {
            *offset = quote!(#offset + #len);
            //FIXME: we need to figure out our 'get' business
            quote!(#start_off..#start_off + #len * std::mem::size_of::<#inner_typ>())
        }
        None => {
            // guard to ensure that this item is only ever the last:
            *offset = quote!(compile_error!(
                "#[count_all] annotation only valid on last field (TODO: validate before here)"
            ));
            quote!(#start_off..)
        }
    };
    quote! {
        pub fn #name(&self) -> Option<&'a [#inner_typ]> {
            self.0.get(#range)
                .and_then(|bytes| zerocopy::LayoutVerified::new_slice_unaligned(bytes))
                .map(|layout| layout.into_slice())
        }
    }
}

fn make_var_array_getter(
    field: &parse::ArrayField,
    offset: &mut proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let name = &field.name;
    let start_off = offset.clone();
    let inner_typ = &field.inner_typ;
    assert!(
        field.inner_lifetime.is_some(),
        "variable arrays are meaningless without an inner lifetime?"
    );
    *offset = quote!(compile_error!(
        "guard violated: variable_size array must be last field in item."
    ));
    quote! {
        pub fn #name(&self) -> Option<font_types::VarArray<'a, #inner_typ>> {
            self.0.get(#start_off..).map(font_types::VarArray::new)
        }
    }
}