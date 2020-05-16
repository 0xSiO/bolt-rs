#[doc(hidden)]
#[macro_export]
macro_rules! define_value_map {
    ($T:ident) => {
        #[derive(Debug, Default, Clone)]
        pub struct $T {
            pub(crate) value: ::std::collections::HashMap<std::string::String, ::bolt_proto::Value>,
        }

        impl<K, V> ::std::convert::From<::std::collections::HashMap<K, V>> for $T
        where
            K: ::std::convert::Into<::std::string::String>,
            V: ::std::convert::Into<::bolt_proto::Value>,
        {
            fn from(
                map: ::std::collections::HashMap<K, V, ::std::collections::hash_map::RandomState>,
            ) -> Self {
                Self {
                    value: map.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
                }
            }
        }

        impl<K, V> ::std::iter::FromIterator<(K, V)> for $T
        where
            K: Eq + ::std::hash::Hash + ::std::convert::Into<std::string::String>,
            V: ::std::convert::Into<::bolt_proto::Value>,
        {
            fn from_iter<T: ::std::iter::IntoIterator<Item = (K, V)>>(iter: T) -> Self {
                Self {
                    value: ::std::collections::HashMap::from_iter(
                        iter.into_iter().map(|(k, v)| (k.into(), v.into())),
                    ),
                }
            }
        }
    };
}
