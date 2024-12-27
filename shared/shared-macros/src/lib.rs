
use helpers::{as_array, assert_has_named_fields, assert_is_struct, assert_type, get_field_with_attribute, get_fields_with_attribute, get_option_inner_type};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Lit};

mod helpers;

#[proc_macro_derive(PlayerFields, attributes(name, disconnected))]
pub fn derive_player_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let data = assert_is_struct(&input.data).expect("PlayerFields can only be applied to structs");
    let fields = assert_has_named_fields(&data.fields).expect("PlayerFields can only be applied to structs with named fields");

    let name_field = get_field_with_attribute(&fields, "name").unwrap_or_else(|e| panic!("{}", e));
    let name_field_name = name_field.ident.as_ref().unwrap();
    let name_field_array = as_array(name_field).expect("Field annotated with `#[name]` must be a fixed size array");
    let name_length = if let Expr::Lit(lit) = &name_field_array.len {
        if let Lit::Int(lit) = &lit.lit {
            let length = lit.base10_parse::<usize>().unwrap();
            if length == 0 {
                panic!("Field annotated with `#[name]` must be a fixed size array with a length greater than 0");
            }
            length
        } else {
            panic!("Field annotated with `#[name]` must be a fixed size array with a literal length");
        }
    } else {
        panic!("Field annotated with `#[name]` must be a fixed size array with a literal length");
    };
    
    let disconnected_field = get_field_with_attribute(&fields, "disconnected").unwrap_or_else(|e| panic!("{}", e));
    let disconnected_field_name = disconnected_field.ident.as_ref().unwrap();
    assert_type(disconnected_field, "bool", "Field annotated with `#[disconnected]` must be of type `bool`");

    // Generate methods to get and set the name and disconnected fields
    let expanded = quote! {
        impl shared_core::PlayerFields for #name {
            fn name(&self) -> &[u8; #name_length] {
                &self.#name_field_name
            }

            fn set_name(&mut self, name: &[u8; #name_length]) {
                self.#name_field_name = *name;
            }

            fn disconnected(&self) -> bool {
                self.#disconnected_field_name
            }

            fn set_disconnected(&mut self, disconnected: bool) {
                self.#disconnected_field_name = disconnected;
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(RoomFields, attributes(players, host, player_index))]
pub fn derive_room_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let data = assert_is_struct(&input.data).expect("RoomFields can only be applied to structs");
    let fields = assert_has_named_fields(&data.fields).expect("RoomFields can only be applied to structs with named fields");

    let host_field = get_field_with_attribute(&fields, "host").unwrap_or_else(|e| panic!("{}", e));
    let host_field_name = host_field.ident.as_ref().unwrap();
    assert_type(host_field, "u8", "Field annotated with `#[host]` must be of type `u8`");

    let players_field = get_field_with_attribute(&fields, "players").unwrap_or_else(|e| panic!("{}", e));
    let players_field_name = players_field.ident.as_ref().unwrap();
    let players_field_array = as_array(players_field).expect("Field annotated with `#[players]` must be a fixed size array");
    let player_array_type = get_option_inner_type(&*players_field_array.elem);

    let player_index_field = get_field_with_attribute(&fields, "player_index").unwrap_or_else(|e| panic!("{}", e));
    let player_index_field_name = player_index_field.ident.as_ref().unwrap();
    assert_type(player_index_field, "u8", "Field annotated with `#[player_index]` must be of type `u8`");

    // Generate the RoomFields implementation
    let expanded = quote! {
        impl shared_core::RoomFields for #name {
            type Player = #player_array_type;

            fn host(&self) -> u8 {
                self.#host_field_name
            }

            fn set_host(&mut self, host: u8) {
                self.#host_field_name = host;
            }

            fn players(&self) -> &[Option<Self::Player>; 8] {
                &self.#players_field_name
            }

            fn players_mut(&mut self) -> &mut [Option<Self::Player>; 8] {
                &mut self.#players_field_name
            }

            fn player_index(&self) -> u8 {
                self.#player_index_field_name
            }

            fn set_player_index(&mut self, player_index: u8) {
                self.#player_index_field_name = player_index;
            }
        }
    };

    TokenStream::from(expanded)
}