use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::bolt::Value;
use crate::error::*;
use crate::native;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x72;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct UnboundRelationship {
    pub(crate) rel_identity: Box<Value>,
    pub(crate) rel_type: Box<Value>,
    pub(crate) properties: Box<Value>,
}

impl From<native::value::UnboundRelationship> for UnboundRelationship {
    fn from(native_ub_rel: native::value::UnboundRelationship) -> Self {
        Self {
            rel_identity: Box::new(Value::from(native_ub_rel.rel_identity)),
            rel_type: Box::new(Value::from(native_ub_rel.rel_type)),
            properties: Box::new(Value::from(native_ub_rel.properties)),
        }
    }
}

impl TryFrom<Value> for UnboundRelationship {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::UnboundRelationship(unbound_rel) => Ok(unbound_rel),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
