use syn::{punctuated::Punctuated, token::Comma, Attribute, DataStruct, Field, Type, TypeArray};

pub fn assert_is_struct(data: &syn::Data) -> Result<&DataStruct, String> {
    if let syn::Data::Struct(data) = data {
        return Ok(data);
    }

    Err("".to_string())
}

pub fn assert_has_named_fields(fields: &syn::Fields) -> Result<&Punctuated<Field, Comma>, String> {
    if let syn::Fields::Named(fields) = fields {
        return Ok(&fields.named);
    }

    Err("".to_string())
}

pub fn assert_type(field: &Field, ty: &str, message: &str) {
    if let syn::Type::Path(path) = &field.ty {
        if path.path.is_ident(ty) {
            return;
        }
    }

    panic!("{}", message);
}

pub fn as_array(field: &Field) -> Result<&TypeArray, &'static str> {
    if let syn::Type::Array(array) = &field.ty {
        return Ok(array);
    }

    Err("")
}

pub fn get_field_with_attribute<'a>(fields: &'a Punctuated<Field, Comma>, attribute: &str) -> Result<&'a Field, String> {
    let fields: Vec<&Field> = fields.iter().filter(|field| {
        has_attr(&field.attrs, attribute)
    }).collect();

    if fields.len() == 1 {
        Ok(&fields[0])
    } else if fields.len() > 1 {
        Err(format!("Only one field can be annotated with `{}`", attribute))
    } else {
        Err(format!("Missing field annotated with `{}`", attribute))
    }
}

pub fn get_fields_with_attribute<'a>(fields: &'a Punctuated<Field, Comma>, attribute: &str) -> Vec<&'a Field> {
    fields.iter().filter(|field| {
        has_attr(&field.attrs, attribute)
    }).collect()
}

fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident(name)
    })
}

pub fn get_option_inner_type(ty: &Type) -> Option<&syn::Type> {
    if let syn::Type::Path(path) = &ty {
        if let Some(segment) = path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let syn::GenericArgument::Type(ty) = &args.args[0] {
                        return Some(ty);
                    }
                }
            }
        }
    }

    None
}
