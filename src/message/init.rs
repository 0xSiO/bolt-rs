use crate::serialize::Serialize;
use crate::value::{Map, String};

struct Init {
    client_name: String,
    // TODO: Impl Serialize for Box<dyn Serialize>
    auth_token: Map<String, Box<dyn Serialize>>,
}
