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
                Some(Self::#name(raw_types::FontRead::read(bytes)?))
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

        impl<'a> raw_types::FontRead<'a> for #name #lifetime {
            fn read(bytes: &'a [u8]) -> Option<Self> {
                #validation_check
                let version: #format = raw_types::FontRead::read(bytes)?;
                match version {
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
    let field_docs = item.fields.iter().map(|fld| {
        let docs = &fld.as_single().unwrap().docs;
        quote!( #( #docs )* )
    });

    quote! {
        #( #docs )*
        #[derive(Clone, Copy, Debug, zerocopy::FromBytes, zerocopy::Unaligned)]
        #[repr(C)]
        pub struct #name {
            #( #field_docs pub #field_names: #field_types, )*
        }

        // and now I want getters. but not for everything? also they return... a different concrete
        // type? or... do I want this, in the zero-copy version?
    }
}

//struct FieldCode {
    //// tokens representing the range of this field in the underlying bytes
    //range: proc_macro2::TokenStream,
    //// the length of this field, if it can be known without access
    //len: Option<proc_macro2::TokenStream>,
//}

//fn generate_field_lens(items: &parse::SingleItem) -> HashMap<&syn::Ident, FieldCode> {
    //let mut result = HashMap::new();
    //let mut current_offset = quote!(0);
    //for field in &items.fields {
        //match field {
            //parse::Field::Scalar(scalar) => {
                //let len = Some(scalar.len_tokens());
                //let range = quote!(#current_offset..#current_offset + #len);
                //current_offset = quote!( #current_offset + #len );
                //result.insert(&scalar.name, FieldCode { range, len });
            //}
            //parse::Field::Array(array) => {
                ////let len_inputs = array.count.iter_input_fields().map(|fld| {
                ////let code = result.get(&field.name()).expect("field names should be validated before codegen");

                ////})
                ////let len = match &array.count {

                ////}
            //}
        //}
    //}

    //result
//}

fn generate_view_impls2(item: &parse::SingleItem) -> Result<proc_macro2::TokenStream, syn::Error> {
    // first get the length of all the contiguious known fields
    let known_lens = item
        .fields
        .iter()
        .map_while(parse::Field::as_single)
        .map(parse::SingleField::len_tokens);
    let head_len = quote!( #( #known_lens )+* );

    // then find every field that is referenced by another field's count/len annotation
    let input_fields = item
        .fields
        .iter()
        .filter_map(|fld| match fld {
            parse::Field::Single(_) => None,
            parse::Field::Array(array) => Some(array.count.iter_input_fields()),
        })
        .flatten();

    //let mut init_input_fields = Vec::new();
    let init_input_fields = input_fields.map(|ident| {
        let field_idx = item.fields.iter().position(|x| x.name() == ident).unwrap();
        let field = item
            .fields
            .get(field_idx)
            .and_then(parse::Field::as_single)
            .unwrap();
        let offset = item.fields[..field_idx]
            .iter()
            .map(|x| x.as_single().unwrap().len_tokens());
        let len = field.len_tokens();

        let bytes = quote! {
        {
        let offset = #( #offset )+*;
        let field_bytes = bytes.get(offset..offset + #len)?
        }
        };
        let getter = field.getter_tokens(&bytes);
        quote!(let #ident = #getter;)
    });

    let len_calcs = item
        .fields
        .iter()
        .skip_while(|x| x.is_single())
        .map(|field| {
            match field {
                parse::Field::Single(scalar) => scalar.len_tokens(),
                _ => quote!(),
                //parse::Field::Array(array)

                // wAAAAAHHHhhhhhhhhhhhhhhhhh I don't wannnnnaaaaaaaaaa
                // it's haaarddddddddddd
            }
        });

    // now calculate the lengths of all remaining fields
    todo!()
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

        impl<'a> raw_types::FontRead<'a> for #name<'a> {
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
    let ty = &field.typ;

    let from_bytes = field.getter_tokens(&get_bytes);
    if checked {
        quote! {
        pub fn #name(&self) -> #ty {
            let raw = #from_bytes.unwrap();
            raw
        }
        }
    } else {
        quote! {
            pub fn #name(&self) -> Option<#ty> {
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
        parse::Count::Field(name) => Some(quote!(self.#name().get() as usize)),
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
        pub fn #name(&self) -> Option<raw_types::VarArray<'a, #inner_typ<'a>>> {
            self.0.get(#start_off..).map(raw_types::VarArray::new)
        }
    }
}
