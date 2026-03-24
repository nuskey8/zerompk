use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Generics, Ident, Lit, LitInt, LitStr,
    Result, Type, Variant, parse_macro_input, parse_quote, spanned::Spanned,
};

#[derive(Clone, Copy)]
enum DeriveKind {
    To,
    From,
}

#[proc_macro_derive(ToMessagePack, attributes(msgpack))]
pub fn derive_to_message_pack(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand(input, DeriveKind::To) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(FromMessagePack, attributes(msgpack))]
pub fn derive_from_message_pack(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand(input, DeriveKind::From) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Repr {
    Array,
    Map,
}

fn parse_repr_from_attrs(attrs: &[syn::Attribute]) -> Result<Option<Repr>> {
    let mut repr = None;

    for attr in attrs {
        if !attr.path().is_ident("msgpack") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("array") {
                if repr.is_some() {
                    return Err(meta.error("duplicate representation attribute"));
                }
                repr = Some(Repr::Array);
                Ok(())
            } else if meta.path.is_ident("map") {
                if repr.is_some() {
                    return Err(meta.error("duplicate representation attribute"));
                }
                repr = Some(Repr::Map);
                Ok(())
            } else if meta.path.is_ident("key") {
                // handled at field/variant level
                Ok(())
            } else {
                Err(meta.error("expected `array`, `map`, or `key = ...`"))
            }
        })?;
    }

    Ok(repr)
}

fn add_trait_bounds(mut generics: Generics, kind: DeriveKind) -> Generics {
    for type_param in generics.type_params_mut() {
        match kind {
            DeriveKind::To => type_param
                .bounds
                .push(parse_quote!(::zerompk::ToMessagePack)),
            DeriveKind::From => type_param.bounds.push(parse_quote!(
                for<'__msgpack_de> ::zerompk::FromMessagePack<'__msgpack_de>
            )),
        }
    }
    generics
}

fn msgpack_string_size(s: &str) -> usize {
    let len = s.len();
    let header = if len <= 31 {
        1
    } else if len <= 255 {
        2
    } else if len <= 65535 {
        3
    } else {
        5
    };
    header + len
}

fn msgpack_u64_size(v: u64) -> usize {
    match v {
        0..=0x7f => 1,
        0x80..=0xff => 2,
        0x0100..=0xffff => 3,
        0x1_0000..=0xffff_ffff => 5,
        _ => 9,
    }
}

fn pack_u64_le_chunk(bytes: &[u8]) -> u64 {
    let mut value = 0u64;
    for (i, b) in bytes.iter().enumerate() {
        value |= (*b as u64) << (i * 8);
    }
    value
}

fn build_key_chunk_read_expr(len: usize, base: usize) -> proc_macro2::TokenStream {
    match len {
        1 => quote! { (__key_bytes[#base] as u64) },
        2 => quote! {
            (u16::from_le_bytes(unsafe {
                *(__key_bytes.as_ptr().add(#base) as *const [u8; 2])
            }) as u64)
        },
        3 => {
            let p0 = base;
            let p2 = base + 2;
            quote! {
                ((u16::from_le_bytes(unsafe {
                    *(__key_bytes.as_ptr().add(#p0) as *const [u8; 2])
                }) as u64)
                    | ((__key_bytes[#p2] as u64) << 16))
            }
        }
        4 => quote! {
            (u32::from_le_bytes(unsafe {
                *(__key_bytes.as_ptr().add(#base) as *const [u8; 4])
            }) as u64)
        },
        5 => {
            let p0 = base;
            let p4 = base + 4;
            quote! {
                ((u32::from_le_bytes(unsafe {
                    *(__key_bytes.as_ptr().add(#p0) as *const [u8; 4])
                }) as u64)
                    | ((__key_bytes[#p4] as u64) << 32))
            }
        }
        6 => {
            let p0 = base;
            let p4 = base + 4;
            quote! {
                ((u32::from_le_bytes(unsafe {
                    *(__key_bytes.as_ptr().add(#p0) as *const [u8; 4])
                }) as u64)
                    | ((u16::from_le_bytes(unsafe {
                        *(__key_bytes.as_ptr().add(#p4) as *const [u8; 2])
                    }) as u64)
                        << 32))
            }
        }
        7 => {
            let p0 = base;
            let p4 = base + 4;
            let p6 = base + 6;
            quote! {
                ((u32::from_le_bytes(unsafe {
                    *(__key_bytes.as_ptr().add(#p0) as *const [u8; 4])
                }) as u64)
                    | ((u16::from_le_bytes(unsafe {
                        *(__key_bytes.as_ptr().add(#p4) as *const [u8; 2])
                    }) as u64)
                        << 32)
                    | ((__key_bytes[#p6] as u64) << 48))
            }
        }
        8 => quote! {
            u64::from_le_bytes(unsafe {
                *(__key_bytes.as_ptr().add(#base) as *const [u8; 8])
            })
        },
        _ => {
            let terms: Vec<_> = (0..len)
                .map(|i| {
                    let pos = base + i;
                    let shift = i * 8;
                    quote! { ((__key_bytes[#pos] as u64) << #shift) }
                })
                .collect();
            quote! { 0u64 #( | #terms )* }
        }
    }
}

fn build_map_key_chunk_dispatch(
    indices: &[usize],
    chunk_idx: usize,
    chunk_vars: &[Ident],
    key_chunks: &[Vec<u64>],
) -> proc_macro2::TokenStream {
    if indices.is_empty() {
        return quote! { usize::MAX };
    }

    if chunk_idx >= chunk_vars.len() {
        let idx = indices[0];
        return quote! { #idx };
    }

    let mut groups = std::collections::BTreeMap::<u64, Vec<usize>>::new();
    for idx in indices {
        groups
            .entry(key_chunks[*idx][chunk_idx])
            .or_default()
            .push(*idx);
    }

    let var = &chunk_vars[chunk_idx];
    let arms: Vec<_> = groups
        .iter()
        .map(|(chunk, grouped_indices)| {
            let body = build_map_key_chunk_dispatch(
                grouped_indices,
                chunk_idx + 1,
                chunk_vars,
                key_chunks,
            );
            quote! {
                #chunk => {
                    #body
                }
            }
        })
        .collect();

    quote! {
        match #var {
            #( #arms, )*
            _ => usize::MAX,
        }
    }
}

fn build_map_key_dispatch_match(
    key_lits: &[LitStr],
    key_lens: &[usize],
) -> proc_macro2::TokenStream {
    let mut groups = std::collections::BTreeMap::<usize, Vec<usize>>::new();
    for (idx, len) in key_lens.iter().copied().enumerate() {
        groups.entry(len).or_default().push(idx);
    }

    let unknown_key_err = quote! {{
        let __unknown_key = match ::core::str::from_utf8(__key_bytes) {
            Ok(s) => s.into(),
            Err(_) => "<invalid-utf8>".into(),
        };
        Err(::zerompk::Error::UnknownKey(__unknown_key))
    }};

    let len_arms: Vec<_> = groups
        .iter()
        .map(|(len, indices)| {
            let chunk_count = len.div_ceil(8);
            let chunk_vars: Vec<_> = (0..chunk_count)
                .map(|i| format_ident!("__key_chunk_{}", i))
                .collect();
            let chunk_reads: Vec<_> = chunk_vars
                .iter()
                .enumerate()
                .map(|(chunk_idx, chunk_var)| {
                    let base = chunk_idx * 8;
                    let take = usize::min(8, len - base);
                    let read_expr = build_key_chunk_read_expr(take, base);

                    quote! {
                        let #chunk_var: u64 = #read_expr;
                    }
                })
                .collect();

            let mut key_chunks = vec![Vec::<u64>::new(); key_lits.len()];
            for idx in indices {
                let bytes = key_lits[*idx].value().into_bytes();
                key_chunks[*idx] = (0..chunk_count)
                    .map(|chunk_idx| {
                        let base = chunk_idx * 8;
                        let end = usize::min(base + 8, bytes.len());
                        pack_u64_le_chunk(&bytes[base..end])
                    })
                    .collect::<Vec<_>>();
            }

            let dispatch = build_map_key_chunk_dispatch(indices, 0, &chunk_vars, &key_chunks);

            quote! {
                #len => {
                    #( #chunk_reads )*
                    #dispatch
                }
            }
        })
        .collect();

    quote! {
        let __matched_idx: usize = match __key_bytes.len() {
            #( #len_arms, )*
            _ => usize::MAX,
        };

        if __matched_idx != usize::MAX {
            Ok(__matched_idx)
        } else {
            #unknown_key_err
        }
    }
}

#[derive(Clone)]
enum KeyAttr {
    Index(usize),
    Name(LitStr),
}

#[derive(Clone)]
enum VariantTag {
    Index(u64),
    Name(LitStr),
}

struct VariantConfig {
    tag: VariantTag,
    repr: Option<Repr>,
}

#[derive(Clone)]
struct FieldConfig {
    key: Option<KeyAttr>,
    ignore: bool,
}

fn parse_field_config(field: &Field) -> Result<FieldConfig> {
    let mut key: Option<KeyAttr> = None;
    let mut ignore = false;

    for attr in &field.attrs {
        if !attr.path().is_ident("msgpack") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;

                if key.is_some() {
                    return Err(meta.error("duplicate `key` attribute"));
                }

                key = Some(match lit {
                    Lit::Int(v) => KeyAttr::Index(parse_positive_index_usize(&v)?),
                    Lit::Str(v) => KeyAttr::Name(v),
                    _ => {
                        return Err(meta.error("`key` must be an integer (array) or string (map)"));
                    }
                });
                Ok(())
            } else if meta.path.is_ident("ignore") {
                if ignore {
                    return Err(meta.error("duplicate `ignore` attribute"));
                }
                ignore = true;
                Ok(())
            } else if meta.path.is_ident("array") || meta.path.is_ident("map") {
                Err(meta.error("field-level msgpack attribute does not support `array/map`"))
            } else {
                Err(meta
                    .error("field-level msgpack attribute supports only `key = ...` or `ignore`"))
            }
        })?;
    }

    if ignore && key.is_some() {
        return Err(syn::Error::new(
            field.span(),
            "`ignore` cannot be used together with `key`",
        ));
    }

    Ok(FieldConfig { key, ignore })
}

fn parse_variant_config(variant: &Variant) -> Result<VariantConfig> {
    let mut key: Option<VariantTag> = None;
    let mut repr: Option<Repr> = None;

    for attr in &variant.attrs {
        if !attr.path().is_ident("msgpack") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;

                if key.is_some() {
                    return Err(meta.error("duplicate variant `key` attribute"));
                }

                key = Some(match lit {
                    Lit::Int(v) => VariantTag::Index(parse_positive_index_u64(&v)?),
                    Lit::Str(v) => VariantTag::Name(v),
                    _ => {
                        return Err(meta.error("variant `key` must be integer or string"));
                    }
                });
                Ok(())
            } else if meta.path.is_ident("array") {
                if repr.is_some() {
                    return Err(meta.error("duplicate variant representation attribute"));
                }
                repr = Some(Repr::Array);
                Ok(())
            } else if meta.path.is_ident("map") {
                if repr.is_some() {
                    return Err(meta.error("duplicate variant representation attribute"));
                }
                repr = Some(Repr::Map);
                Ok(())
            } else {
                Err(meta.error("expected `key = ...`, `array`, or `map`"))
            }
        })?;
    }

    let default_tag = VariantTag::Name(LitStr::new(
        &variant.ident.to_string(),
        variant.ident.span(),
    ));

    Ok(VariantConfig {
        tag: key.unwrap_or(default_tag),
        repr,
    })
}

fn parse_positive_index_usize(v: &LitInt) -> Result<usize> {
    v.base10_parse::<usize>()
}

fn parse_positive_index_u64(v: &LitInt) -> Result<u64> {
    v.base10_parse::<u64>()
}

fn is_ref_str(ty: &Type) -> bool {
    match ty {
        Type::Reference(reference) => match reference.elem.as_ref() {
            Type::Path(path) => path.path.is_ident("str"),
            _ => false,
        },
        _ => false,
    }
}

fn is_ref_u8_slice(ty: &Type) -> bool {
    match ty {
        Type::Reference(reference) => match reference.elem.as_ref() {
            Type::Slice(slice) => match slice.elem.as_ref() {
                Type::Path(path) => path.path.is_ident("u8"),
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

fn build_read_expr(ty: &Type) -> proc_macro2::TokenStream {
    if is_ref_str(ty) {
        quote! {
            <&'__msgpack_de str as ::zerompk::FromMessagePack<'__msgpack_de>>::read(reader)?
        }
    } else if is_ref_u8_slice(ty) {
        quote! {
            <&'__msgpack_de [u8] as ::zerompk::FromMessagePack<'__msgpack_de>>::read(reader)?
        }
    } else {
        quote! {
            <#ty as ::zerompk::FromMessagePack<'__msgpack_de>>::read(reader)?
        }
    }
}

fn build_write_expr(value: proc_macro2::TokenStream, ty: &Type) -> proc_macro2::TokenStream {
    if is_ref_str(ty) {
        quote! {
            writer.write_string(#value)?;
        }
    } else if is_ref_u8_slice(ty) {
        quote! {
            writer.write_binary(#value)?;
        }
    } else {
        quote! {
            #value.write(writer)?;
        }
    }
}

fn build_named_array_slots(
    fields: &syn::FieldsNamed,
    configs: &[FieldConfig],
) -> Result<Vec<Option<usize>>> {
    let mut field_index_by_slot: Vec<Option<usize>> = Vec::new();
    let mut next_auto_index = 0usize;

    for (decl_idx, field) in fields.named.iter().enumerate() {
        let cfg = &configs[decl_idx];
        if cfg.ignore {
            continue;
        }

        let assigned = match &cfg.key {
            Some(KeyAttr::Index(v)) => *v,
            Some(KeyAttr::Name(_)) => {
                return Err(syn::Error::new(
                    field.span(),
                    "array representation requires integer `key`",
                ));
            }
            None => {
                let assigned = next_auto_index;
                next_auto_index += 1;
                assigned
            }
        };

        if assigned >= field_index_by_slot.len() {
            field_index_by_slot.resize(assigned + 1, None);
        }
        if field_index_by_slot[assigned].is_some() {
            return Err(syn::Error::new(
                field.span(),
                "duplicate array index in `key`",
            ));
        }
        field_index_by_slot[assigned] = Some(decl_idx);
    }

    Ok(field_index_by_slot)
}

fn build_unnamed_array_slots(
    fields: &syn::FieldsUnnamed,
    configs: &[FieldConfig],
) -> Result<Vec<Option<usize>>> {
    let mut field_index_by_slot: Vec<Option<usize>> = Vec::new();
    let mut next_auto_index = 0usize;

    for (decl_idx, field) in fields.unnamed.iter().enumerate() {
        let cfg = &configs[decl_idx];
        if cfg.ignore {
            continue;
        }

        let assigned = match &cfg.key {
            Some(KeyAttr::Index(v)) => *v,
            Some(KeyAttr::Name(_)) => {
                return Err(syn::Error::new(
                    field.span(),
                    "array representation requires integer `key`",
                ));
            }
            None => {
                let assigned = next_auto_index;
                next_auto_index += 1;
                assigned
            }
        };

        if assigned >= field_index_by_slot.len() {
            field_index_by_slot.resize(assigned + 1, None);
        }
        if field_index_by_slot[assigned].is_some() {
            return Err(syn::Error::new(
                field.span(),
                "duplicate array index in `key`",
            ));
        }
        field_index_by_slot[assigned] = Some(decl_idx);
    }

    Ok(field_index_by_slot)
}

fn parse_named_map_keys(
    fields: &syn::FieldsNamed,
    configs: &[FieldConfig],
) -> Result<(Vec<usize>, Vec<LitStr>)> {
    let mut field_indices: Vec<usize> = Vec::with_capacity(fields.named.len());
    let mut keys: Vec<LitStr> = Vec::with_capacity(fields.named.len());
    let mut key_values: Vec<String> = Vec::with_capacity(fields.named.len());

    for (decl_idx, field) in fields.named.iter().enumerate() {
        let cfg = &configs[decl_idx];
        if cfg.ignore {
            continue;
        }

        let fallback = field.ident.clone().expect("named field");
        let key_lit = match &cfg.key {
            Some(KeyAttr::Name(v)) => v.clone(),
            Some(KeyAttr::Index(_)) => {
                return Err(syn::Error::new(
                    field.span(),
                    "map representation requires string `key`",
                ));
            }
            None => LitStr::new(&fallback.to_string(), fallback.span()),
        };

        key_values.push(key_lit.value());
        keys.push(key_lit);
        field_indices.push(decl_idx);
    }

    {
        use std::collections::HashSet;
        let mut seen = HashSet::<&str>::new();
        for key in &key_values {
            if !seen.insert(key.as_str()) {
                return Err(syn::Error::new(fields.span(), "duplicate map key in `key`"));
            }
        }
    }

    Ok((field_indices, keys))
}

fn expand(input: DeriveInput, kind: DeriveKind) -> Result<proc_macro2::TokenStream> {
    let DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    } = input;

    let repr_attr = parse_repr_from_attrs(&attrs)?;
    let generics = add_trait_bounds(generics, kind);
    let lifetime_params: Vec<_> = generics
        .lifetimes()
        .map(|lifetime_def| lifetime_def.lifetime.clone())
        .collect();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = match data {
        Data::Struct(data) => {
            let repr = repr_attr.unwrap_or(Repr::Array);
            match repr {
                Repr::Array => expand_array_struct(&data)?,
                Repr::Map => expand_map_struct(&data)?,
            }
        }
        Data::Enum(data) => {
            if repr_attr.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "enum itself is always encoded as [tag, payload], so top-level #[msgpack(array/map)] is not allowed",
                ));
            }
            expand_enum(&data)?
        }
        _ => {
            return Err(syn::Error::new(
                ident.span(),
                "To/FromMessagePack derive supports structs and enums only",
            ));
        }
    };

    let ImplBody { write, read } = body;

    let tokens = match kind {
        DeriveKind::To => quote! {
            impl #impl_generics ::zerompk::ToMessagePack for #ident #ty_generics #where_clause {
                fn write<W: ::zerompk::Write>(&self, writer: &mut W) -> ::core::result::Result<(), ::zerompk::Error> {
                    #write
                }
            }
        },
        DeriveKind::From => {
            let mut from_generics = generics.clone();
            from_generics.params.insert(0, parse_quote!('__msgpack_de));
            {
                let where_clause = from_generics.make_where_clause();
                for lifetime in &lifetime_params {
                    where_clause
                        .predicates
                        .push(parse_quote!('__msgpack_de: #lifetime));
                }
            }
            let (from_impl_generics, _, from_where_clause) = from_generics.split_for_impl();

            quote! {
                impl #from_impl_generics ::zerompk::FromMessagePack<'__msgpack_de> for #ident #ty_generics #from_where_clause {
                    fn read<R: ::zerompk::Read<'__msgpack_de>>(reader: &mut R) -> ::core::result::Result<Self, ::zerompk::Error>
                    where
                        Self: Sized,
                    {
                        reader.increment_depth()?;
                        let __result = {
                            #read
                        };
                        reader.decrement_depth();
                        __result
                    }
                }
            }
        }
    };

    Ok(tokens)
}

struct ImplBody {
    write: proc_macro2::TokenStream,
    read: proc_macro2::TokenStream,
}

fn expand_array_struct(data: &DataStruct) -> Result<ImplBody> {
    match &data.fields {
        Fields::Named(fields) => {
            let names: Vec<_> = fields
                .named
                .iter()
                .map(|f| f.ident.clone().expect("named field"))
                .collect();
            let tys: Vec<_> = fields.named.iter().map(|f| f.ty.clone()).collect();
            let field_configs: Vec<_> = fields
                .named
                .iter()
                .map(parse_field_config)
                .collect::<Result<_>>()?;
            let field_index_by_slot = build_named_array_slots(fields, &field_configs)?;

            let array_len = field_index_by_slot.len();
            let is_dense_sequential = field_index_by_slot.len() == names.len()
                && field_index_by_slot
                    .iter()
                    .enumerate()
                    .all(|(slot_idx, slot)| matches!(slot, Some(i) if *i == slot_idx))
                && field_configs.iter().all(|cfg| !cfg.ignore);
            let slot_writes: Vec<_> = field_index_by_slot
                .iter()
                .map(|slot| match slot {
                    Some(i) => {
                        let name = &names[*i];
                        let ty = &tys[*i];
                        build_write_expr(quote! { self.#name }, ty)
                    }
                    None => quote! { writer.write_nil()?; },
                })
                .collect();

            let read_slots: Vec<_> = field_index_by_slot
                .iter()
                .map(|slot| match slot {
                    Some(i) => {
                        let name = &names[*i];
                        let ty = &tys[*i];
                        let read_expr = build_read_expr(ty);
                        quote! { let #name = #read_expr; }
                    }
                    None => quote! { reader.read_nil()?; },
                })
                .collect();

            let init_fields: Vec<_> = names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let ty = &tys[i];
                    if field_configs[i].ignore {
                        quote! { #name: <#ty as ::core::default::Default>::default() }
                    } else {
                        quote! { #name: #name }
                    }
                })
                .collect();

            let write = quote! {
                writer.write_array_len(#array_len)?;
                #( #slot_writes )*
                Ok(())
            };

            let read = if is_dense_sequential {
                let direct_fields: Vec<_> = names
                    .iter()
                    .zip(tys.iter())
                    .map(|(name, ty)| {
                        let read_expr = build_read_expr(ty);
                        quote! { #name: #read_expr }
                    })
                    .collect();

                quote! {
                    reader.check_array_len(#array_len)?;
                    Ok(Self { #( #direct_fields ),* })
                }
            } else {
                quote! {
                    reader.check_array_len(#array_len)?;
                    #( #read_slots )*
                    Ok(Self { #( #init_fields ),* })
                }
            };

            Ok(ImplBody { write, read })
        }
        Fields::Unnamed(fields) => {
            let count = fields.unnamed.len();
            let field_configs: Vec<_> = fields
                .unnamed
                .iter()
                .map(parse_field_config)
                .collect::<Result<_>>()?;

            if count == 1 && !field_configs[0].ignore {
                let ty = fields
                    .unnamed
                    .first()
                    .expect("single unnamed field")
                    .ty
                    .clone();
                let write_expr = build_write_expr(quote! { self.0 }, &ty);
                let read_expr = build_read_expr(&ty);

                let write = quote! {
                    #write_expr
                    Ok(())
                };

                let read = quote! {
                    let __f0 = #read_expr;
                    Ok(Self(__f0))
                };

                return Ok(ImplBody { write, read });
            }

            let idx: Vec<_> = (0..count).map(syn::Index::from).collect();
            let vars: Vec<_> = (0..count).map(|i| format_ident!("__r{i}")).collect();
            let tys: Vec<_> = fields.unnamed.iter().map(|f| f.ty.clone()).collect();
            let field_index_by_slot = build_unnamed_array_slots(fields, &field_configs)?;

            let array_len = field_index_by_slot.len();
            let is_dense_sequential = field_index_by_slot.len() == count
                && field_index_by_slot
                    .iter()
                    .enumerate()
                    .all(|(slot_idx, slot)| matches!(slot, Some(i) if *i == slot_idx))
                && field_configs.iter().all(|cfg| !cfg.ignore);
            let slot_writes: Vec<_> = field_index_by_slot
                .iter()
                .map(|slot| match slot {
                    Some(i) => {
                        let field_idx = &idx[*i];
                        let ty = &tys[*i];
                        build_write_expr(quote! { self.#field_idx }, ty)
                    }
                    None => quote! { writer.write_nil()?; },
                })
                .collect();

            let read_slots: Vec<_> = field_index_by_slot
                .iter()
                .map(|slot| match slot {
                    Some(i) => {
                        let var = &vars[*i];
                        let ty = &tys[*i];
                        let read_expr = build_read_expr(ty);
                        quote! { let #var = #read_expr; }
                    }
                    None => quote! { reader.read_nil()?; },
                })
                .collect();

            let write = quote! {
                writer.write_array_len(#array_len)?;
                #( #slot_writes )*
                Ok(())
            };

            let ctor_values: Vec<_> = vars
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let ty = &tys[i];
                    if field_configs[i].ignore {
                        quote! { <#ty as ::core::default::Default>::default() }
                    } else {
                        quote! { #v }
                    }
                })
                .collect();

            let read = if is_dense_sequential {
                let direct_values: Vec<_> = tys.iter().map(build_read_expr).collect();

                quote! {
                    reader.check_array_len(#array_len)?;
                    Ok(Self( #( #direct_values ),* ))
                }
            } else {
                quote! {
                    reader.check_array_len(#array_len)?;
                    #( #read_slots )*
                    Ok(Self( #( #ctor_values ),* ))
                }
            };

            Ok(ImplBody { write, read })
        }
        Fields::Unit => Ok(ImplBody {
            write: quote! {
                writer.write_nil()?;
                Ok(())
            },
            read: quote! {
                reader.read_nil()?;
                Ok(Self)
            },
        }),
    }
}

fn expand_map_struct(data: &DataStruct) -> Result<ImplBody> {
    let fields = match &data.fields {
        Fields::Named(fields) => fields,
        Fields::Unnamed(_) | Fields::Unit => {
            return Err(syn::Error::new(
                data.fields.span(),
                "#[msgpack(map)] is supported only for structs with named fields",
            ));
        }
    };

    let names_all: Vec<_> = fields
        .named
        .iter()
        .map(|f| f.ident.clone().expect("named field"))
        .collect();
    let tys_all: Vec<_> = fields.named.iter().map(|f| f.ty.clone()).collect();
    let field_configs: Vec<_> = fields
        .named
        .iter()
        .map(parse_field_config)
        .collect::<Result<_>>()?;
    let (field_indices, key_lits) = parse_named_map_keys(fields, &field_configs)?;
    let count = field_indices.len();
    let names: Vec<_> = field_indices
        .iter()
        .map(|i| names_all[*i].clone())
        .collect();
    let tys: Vec<_> = field_indices.iter().map(|i| tys_all[*i].clone()).collect();
    let key_lens: Vec<_> = key_lits.iter().map(|k| k.value().len()).collect();
    let slots: Vec<_> = names
        .iter()
        .map(|n| format_ident!("__slot_{}", n))
        .collect();
    let key_dispatch = build_map_key_dispatch_match(&key_lits, &key_lens);
    let read_value_arms: Vec<_> = (0..count)
        .map(|idx| {
            let key_name = &key_lits[idx];
            let slot = &slots[idx];
            let ty = &tys[idx];
            let read_expr = build_read_expr(ty);
            quote! {
                #idx => {
                    if #slot.is_some() {
                        return Err(::zerompk::Error::KeyDuplicated(#key_name.into()));
                    }
                    #slot = ::core::option::Option::Some(#read_expr);
                }
            }
        })
        .collect();
    let init_fields: Vec<_> = names_all
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let ty = &tys_all[i];
            if field_configs[i].ignore {
                quote! { #name: <#ty as ::core::default::Default>::default() }
            } else {
                quote! { #name: #name }
            }
        })
        .collect();
    let value_writes: Vec<_> = names
        .iter()
        .zip(tys.iter())
        .map(|(name, ty)| build_write_expr(quote! { self.#name }, ty))
        .collect();

    let write = quote! {
        writer.write_map_len(#count)?;
        #(
            writer.write_string(#key_lits)?;
            #value_writes
        )*
        Ok(())
    };

    let read = quote! {
        reader.check_map_len(#count)?;

        #( let mut #slots: ::core::option::Option<#tys> = ::core::option::Option::None; )*

        for _ in 0..#count {
            let __key_bytes = reader.read_string_bytes()?;
            let __key_bytes = __key_bytes.as_ref();
            let __key_index = (|| -> ::zerompk::Result<usize> {
                #key_dispatch
            })()?;

            match __key_index {
                #( #read_value_arms )*
                _ => unreachable!(),
            }
        }

        #(
            let #names = #slots.ok_or_else(|| ::zerompk::Error::KeyNotFound(#key_lits.into()))?;
        )*

        Ok(Self { #( #init_fields ),* })
    };

    Ok(ImplBody { write, read })
}

fn expand_enum(data: &DataEnum) -> Result<ImplBody> {
    let mut seen_str_tags: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut seen_int_tags: std::collections::HashSet<u64> = std::collections::HashSet::new();

    let mut max_arms = Vec::new();
    let mut write_arms = Vec::new();
    let mut read_str_arms = Vec::new();
    let mut read_int_arms = Vec::new();

    for variant in &data.variants {
        let v_ident = &variant.ident;
        let cfg = parse_variant_config(variant)?;

        match &cfg.tag {
            VariantTag::Name(s) => {
                let value = s.value();
                if !seen_str_tags.insert(value) {
                    return Err(syn::Error::new(v_ident.span(), "duplicate enum string tag"));
                }
            }
            VariantTag::Index(i) => {
                if !seen_int_tags.insert(*i) {
                    return Err(syn::Error::new(
                        v_ident.span(),
                        "duplicate enum integer tag",
                    ));
                }
            }
        }

        let (max_pat, max_payload_size, write_pat, write_payload, read_ctor) =
            build_enum_variant_payload(variant, &cfg)?;

        let (tag_size_expr, tag_write_expr, str_arm, int_arm) = match &cfg.tag {
            VariantTag::Name(s) => {
                let tag_size = msgpack_string_size(&s.value());
                let str_arm = {
                    let s = s.clone();
                    quote! {
                        else if __tag == #s {
                            #read_ctor
                        }
                    }
                };
                (
                    quote! { #tag_size },
                    quote! { writer.write_string(#s)?; },
                    Some(str_arm),
                    None,
                )
            }
            VariantTag::Index(i) => {
                let tag_size = msgpack_u64_size(*i);
                let int_arm = {
                    let i = *i;
                    quote! { #i => { #read_ctor } }
                };
                (
                    quote! { #tag_size },
                    quote! { writer.write_u64(#i)?; },
                    None,
                    Some(int_arm),
                )
            }
        };

        max_arms.push(quote! {
            #max_pat => {
                1 + #tag_size_expr + #max_payload_size
            }
        });

        write_arms.push(quote! {
            #write_pat => {
                writer.write_array_len(2)?;
                #tag_write_expr
                #write_payload
                Ok(())
            }
        });

        if let Some(a) = str_arm {
            read_str_arms.push(a);
        }
        if let Some(a) = int_arm {
            read_int_arms.push(a);
        }
    }

    let write = quote! {
        match self {
            #( #write_arms ),*
        }
    };

    let read_string_branch = if read_str_arms.is_empty() {
        quote! {
            Err(::zerompk::Error::InvalidMarker(0))
        }
    } else {
        quote! {
            if false {
                unreachable!();
            }
            #( #read_str_arms )*
            else {
                Err(::zerompk::Error::InvalidMarker(0))
            }
        }
    };

    let read_int_branch = if read_int_arms.is_empty() {
        quote! {
            Err(::zerompk::Error::InvalidMarker(0))
        }
    } else {
        quote! {
            match __i {
                #( #read_int_arms ),*,
                _ => Err(::zerompk::Error::InvalidMarker(0)),
            }
        }
    };

    let read = quote! {
        reader.check_array_len(2)?;

        match reader.read_tag()? {
            ::zerompk::Tag::String(__tag) => {
                #read_string_branch
            }
            ::zerompk::Tag::Int(__i) => {
                #read_int_branch
            }
        }
    };

    Ok(ImplBody { write, read })
}

fn build_enum_variant_payload(
    variant: &Variant,
    cfg: &VariantConfig,
) -> Result<(
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
)> {
    let v_ident = &variant.ident;

    match &variant.fields {
        Fields::Unit => {
            if cfg.repr.is_some() {
                return Err(syn::Error::new(
                    variant.span(),
                    "unit variant does not support #[msgpack(array/map)]",
                ));
            }

            let max_pat = quote! { Self::#v_ident };
            let max_payload_size = quote! { 1usize };

            let write_pat = quote! { Self::#v_ident };
            let write_payload = quote! {
                writer.write_nil()?;
            };

            let read_ctor = quote! {
                reader.read_nil()?;
                Ok(Self::#v_ident)
            };

            Ok((
                max_pat,
                max_payload_size,
                write_pat,
                write_payload,
                read_ctor,
            ))
        }
        Fields::Unnamed(fields) => {
            if matches!(cfg.repr, Some(Repr::Map)) {
                return Err(syn::Error::new(
                    variant.span(),
                    "tuple variant does not support #[msgpack(map)]",
                ));
            }

            let count = fields.unnamed.len();
            let field_configs: Vec<_> = fields
                .unnamed
                .iter()
                .map(parse_field_config)
                .collect::<Result<_>>()?;
            let bind_vars: Vec<_> = (0..count).map(|i| format_ident!("__f{i}")).collect();
            let tys: Vec<Type> = fields.unnamed.iter().map(|f| f.ty.clone()).collect();
            let slots = build_unnamed_array_slots(fields, &field_configs)?;
            let payload_len = slots.len();
            let is_dense_sequential = slots.len() == count
                && slots
                    .iter()
                    .enumerate()
                    .all(|(slot_idx, slot)| matches!(slot, Some(i) if *i == slot_idx))
                && field_configs.iter().all(|cfg| !cfg.ignore);

            let payload_max_parts: Vec<_> = slots
                .iter()
                .map(|slot| match slot {
                    Some(i) => {
                        let v = &bind_vars[*i];
                        quote! { #v.max_size() }
                    }
                    None => quote! { 1usize },
                })
                .collect();

            let payload_write_parts: Vec<_> = slots
                .iter()
                .map(|slot| match slot {
                    Some(i) => {
                        let v = &bind_vars[*i];
                        let ty = &tys[*i];
                        build_write_expr(quote! { #v }, ty)
                    }
                    None => quote! { writer.write_nil()?; },
                })
                .collect();

            let read_vars: Vec<_> = (0..count).map(|i| format_ident!("__r{i}")).collect();
            let read_slots: Vec<_> = slots
                .iter()
                .map(|slot| match slot {
                    Some(i) => {
                        let rv = &read_vars[*i];
                        let ty = &tys[*i];
                        let read_expr = build_read_expr(ty);
                        quote! { let #rv = #read_expr; }
                    }
                    None => quote! { reader.read_nil()?; },
                })
                .collect();

            let ctor_values: Vec<_> = read_vars
                .iter()
                .enumerate()
                .map(|(i, rv)| {
                    let ty = &tys[i];
                    if field_configs[i].ignore {
                        quote! { <#ty as ::core::default::Default>::default() }
                    } else {
                        quote! { #rv }
                    }
                })
                .collect();

            let max_pat = quote! { Self::#v_ident( #( #bind_vars ),* ) };
            let max_payload_size = quote! { 1 #( + #payload_max_parts )* };

            let write_pat = quote! { Self::#v_ident( #( #bind_vars ),* ) };
            let write_payload = quote! {
                writer.write_array_len(#payload_len)?;
                #( #payload_write_parts )*
            };

            let read_ctor = if is_dense_sequential {
                let direct_values: Vec<_> = tys.iter().map(build_read_expr).collect();

                quote! {
                    reader.check_array_len(#payload_len)?;
                    Ok(Self::#v_ident( #( #direct_values ),* ))
                }
            } else {
                quote! {
                    reader.check_array_len(#payload_len)?;
                    #( #read_slots )*
                    Ok(Self::#v_ident( #( #ctor_values ),* ))
                }
            };

            Ok((
                max_pat,
                max_payload_size,
                write_pat,
                write_payload,
                read_ctor,
            ))
        }
        Fields::Named(fields) => {
            let repr = cfg.repr.unwrap_or(Repr::Array);

            let names: Vec<Ident> = fields
                .named
                .iter()
                .map(|f| f.ident.clone().expect("named field"))
                .collect();
            let tys: Vec<Type> = fields.named.iter().map(|f| f.ty.clone()).collect();
            let field_configs: Vec<_> = fields
                .named
                .iter()
                .map(parse_field_config)
                .collect::<Result<_>>()?;
            let pat_fields: Vec<_> = names
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    if field_configs[i].ignore {
                        quote! { #n: _ }
                    } else {
                        quote! { #n }
                    }
                })
                .collect();

            match repr {
                Repr::Array => {
                    let slots = build_named_array_slots(fields, &field_configs)?;
                    let payload_len = slots.len();
                    let is_dense_sequential = slots.len() == names.len()
                        && slots
                            .iter()
                            .enumerate()
                            .all(|(slot_idx, slot)| matches!(slot, Some(i) if *i == slot_idx))
                        && field_configs.iter().all(|cfg| !cfg.ignore);

                    let payload_max_parts: Vec<_> = slots
                        .iter()
                        .map(|slot| match slot {
                            Some(i) => {
                                let n = &names[*i];
                                quote! { #n.max_size() }
                            }
                            None => quote! { 1usize },
                        })
                        .collect();

                    let payload_write_parts: Vec<_> = slots
                        .iter()
                        .map(|slot| match slot {
                            Some(i) => {
                                let n = &names[*i];
                                let ty = &tys[*i];
                                build_write_expr(quote! { #n }, ty)
                            }
                            None => quote! { writer.write_nil()?; },
                        })
                        .collect();

                    let read_slots: Vec<_> = slots
                        .iter()
                        .map(|slot| match slot {
                            Some(i) => {
                                let n = &names[*i];
                                let ty = &tys[*i];
                                let read_expr = build_read_expr(ty);
                                quote! { let #n = #read_expr; }
                            }
                            None => quote! { reader.read_nil()?; },
                        })
                        .collect();

                    let init_fields: Vec<_> = names
                        .iter()
                        .enumerate()
                        .map(|(i, n)| {
                            let ty = &tys[i];
                            if field_configs[i].ignore {
                                quote! { #n: <#ty as ::core::default::Default>::default() }
                            } else {
                                quote! { #n: #n }
                            }
                        })
                        .collect();

                    let max_pat = quote! { Self::#v_ident { #( #pat_fields ),* } };
                    let max_payload_size = quote! { 1 #( + #payload_max_parts )* };

                    let write_pat = quote! { Self::#v_ident { #( #pat_fields ),* } };
                    let write_payload = quote! {
                        writer.write_array_len(#payload_len)?;
                        #( #payload_write_parts )*
                    };

                    let read_ctor = if is_dense_sequential {
                        let direct_fields: Vec<_> = names
                            .iter()
                            .zip(tys.iter())
                            .map(|(n, ty)| {
                                let read_expr = build_read_expr(ty);
                                quote! { #n: #read_expr }
                            })
                            .collect();

                        quote! {
                            reader.check_array_len(#payload_len)?;
                            Ok(Self::#v_ident { #( #direct_fields ),* })
                        }
                    } else {
                        quote! {
                            reader.check_array_len(#payload_len)?;
                            #( #read_slots )*
                            Ok(Self::#v_ident { #( #init_fields ),* })
                        }
                    };

                    Ok((
                        max_pat,
                        max_payload_size,
                        write_pat,
                        write_payload,
                        read_ctor,
                    ))
                }
                Repr::Map => {
                    let (field_indices, key_lits) = parse_named_map_keys(fields, &field_configs)?;
                    let active_names: Vec<_> =
                        field_indices.iter().map(|i| names[*i].clone()).collect();
                    let active_tys: Vec<_> =
                        field_indices.iter().map(|i| tys[*i].clone()).collect();
                    let key_lens: Vec<_> = key_lits.iter().map(|k| k.value().len()).collect();
                    let key_sizes: Vec<_> = key_lits
                        .iter()
                        .map(|k| msgpack_string_size(&k.value()))
                        .collect();

                    let slot_vars: Vec<_> = active_names
                        .iter()
                        .map(|n| format_ident!("__slot_{}", n))
                        .collect();
                    let key_dispatch = build_map_key_dispatch_match(&key_lits, &key_lens);

                    let count = field_indices.len();
                    let read_value_arms: Vec<_> = (0..count)
                        .map(|idx| {
                            let key_name = &key_lits[idx];
                            let slot = &slot_vars[idx];
                            let ty = &active_tys[idx];
                            let read_expr = build_read_expr(ty);
                            quote! {
                                #idx => {
                                    if #slot.is_some() {
                                        return Err(::zerompk::Error::KeyDuplicated(#key_name.into()));
                                    }
                                    #slot = ::core::option::Option::Some(#read_expr);
                                }
                            }
                        })
                        .collect();

                    let init_fields: Vec<_> = names
                        .iter()
                        .enumerate()
                        .map(|(i, n)| {
                            let ty = &tys[i];
                            if field_configs[i].ignore {
                                quote! { #n: <#ty as ::core::default::Default>::default() }
                            } else {
                                quote! { #n: #n }
                            }
                        })
                        .collect();

                    let max_pat = quote! { Self::#v_ident { #( #pat_fields ),* } };
                    let max_payload_size =
                        quote! { 1 #( + #key_sizes + #active_names.max_size() )* };

                    let write_pat = quote! { Self::#v_ident { #( #pat_fields ),* } };
                    let active_write_parts: Vec<_> = active_names
                        .iter()
                        .zip(active_tys.iter())
                        .map(|(name, ty)| build_write_expr(quote! { #name }, ty))
                        .collect();
                    let write_payload = quote! {
                        writer.write_map_len(#count)?;
                        #(
                            writer.write_string(#key_lits)?;
                            #active_write_parts
                        )*
                    };

                    let read_ctor = quote! {
                        reader.check_map_len(#count)?;

                        #( let mut #slot_vars: ::core::option::Option<#active_tys> = ::core::option::Option::None; )*

                        for _ in 0..#count {
                            let __key_bytes = reader.read_string_bytes()?;
                            let __key_bytes = __key_bytes.as_ref();
                            let __key_index = (|| -> ::zerompk::Result<usize> {
                                #key_dispatch
                            })()?;

                            match __key_index {
                                #( #read_value_arms )*
                                _ => unreachable!(),
                            }
                        }

                        #(
                            let #active_names = #slot_vars.ok_or_else(|| ::zerompk::Error::KeyNotFound(#key_lits.into()))?;
                        )*

                        Ok(Self::#v_ident { #( #init_fields ),* })
                    };

                    Ok((
                        max_pat,
                        max_payload_size,
                        write_pat,
                        write_payload,
                        read_ctor,
                    ))
                }
            }
        }
    }
}
