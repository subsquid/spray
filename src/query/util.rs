
pub fn is_default<T: Default + Eq>(value: &T) -> bool {
    value.eq(&T::default())
}


macro_rules! item_field_selection {
    (
        $( $type_name:ident { $(  $field:ident, )* } )*
    ) => {
        $(
        #[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase", default, deny_unknown_fields)]
        pub struct $type_name {
            $(
                #[serde(skip_serializing_if = "std::ops::Not::not")]
                pub $field: bool
            ),*
        }
        )*
    };
}
pub(super) use item_field_selection;


macro_rules! field_selection {
    (
        $($item_name:ident: $field_selection:ty ,)*
    ) => {
        #[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase", default, deny_unknown_fields)]
        pub struct FieldSelection {
            $(
                #[serde(skip_serializing_if = "super::util::is_default")]
                pub $item_name: $field_selection,
            )*
        }
    };
}
pub(crate) use field_selection;


macro_rules! request {
    ($(
        pub struct $name:ident {
            $(
                $(#[serde($($serde_attr:tt)*)])?
                pub $field:ident: $field_type:ty,
            )*
        }
    )*) => {
        $(
            #[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
            #[serde(rename_all = "camelCase", default, deny_unknown_fields)]
            pub struct $name {
                $(
                    #[serde(skip_serializing_if = "super::util::is_default" $(, $($serde_attr)*)*)]
                    pub $field: $field_type,
                )*
            }
        )*
    };
}
pub(super) use request;


pub fn parse_hex(s: &str) -> Option<Vec<u8>> {
    if !s.starts_with("0x") {
        return None
    }
    if s.len() % 2 != 0 {
        return None
    }
    let mut bytes = vec![0; s.len() / 2 - 1];
    faster_hex::hex_decode(s[2..].as_bytes(), &mut bytes)
        .ok()
        .map(|_| bytes)
}