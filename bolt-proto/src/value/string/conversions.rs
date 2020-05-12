use crate::impl_try_from_value;
use crate::value::String;

impl From<&str> for String {
    fn from(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl From<std::string::String> for String {
    fn from(value: std::string::String) -> Self {
        Self { value }
    }
}

impl From<String> for std::string::String {
    fn from(string: String) -> Self {
        string.value
    }
}

impl_try_from_value!(String, String);
