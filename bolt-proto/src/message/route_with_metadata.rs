use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, message::SIGNATURE_ROUTE, Value};

#[bolt_structure(SIGNATURE_ROUTE)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RouteWithMetadata {
    pub(crate) context: HashMap<String, Value>,
    pub(crate) bookmarks: Vec<String>,
    pub(crate) metadata: HashMap<String, Value>,
}

impl RouteWithMetadata {
    pub fn new(
        context: HashMap<String, Value>,
        bookmarks: Vec<String>,
        metadata: HashMap<String, Value>,
    ) -> Self {
        Self {
            context,
            bookmarks,
            metadata,
        }
    }

    pub fn context(&self) -> &HashMap<String, Value> {
        &self.context
    }

    pub fn bookmarks(&self) -> &[String] {
        &self.bookmarks
    }

    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }
}

impl_try_from_message!(RouteWithMetadata, RouteWithMetadata);
