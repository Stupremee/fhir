/// Helper macro that implements [`Display`](std::fmt::Display) for an enum.
///
/// It will use [`serde`] to serialize the enum to a string, and then display that string.
#[macro_export]
macro_rules! enum_display_serde {
    ($name:ident) => {
        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                // TODO: going through `serde_json` is not 100% ideal,
                // but it's fine for now

                let serde_json::Value::String(s) = serde_json::to_value(self).unwrap() else {
                    panic!("enum did not serialize to string");
                };
                write!(f, "{}", s)
            }
        }
    };
}
