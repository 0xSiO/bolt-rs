use crate::serialize::Serialize;
use crate::value::{Map, String};

struct Init {
    client_name: String,
    auth_token: Map<String, Box<dyn Serialize>>,
}
