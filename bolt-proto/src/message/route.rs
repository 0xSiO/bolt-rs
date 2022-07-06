use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, message::SIGNATURE_ROUTE, Value};

#[bolt_structure(SIGNATURE_ROUTE)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Route {
    pub(crate) context: HashMap<String, Value>,
    pub(crate) bookmarks: Vec<String>,
    pub(crate) database: Value,
}

impl Route {
    pub fn new(
        context: HashMap<String, Value>,
        bookmarks: Vec<String>,
        database: Option<String>,
    ) -> Self {
        Self {
            context,
            bookmarks,
            database: Value::from(database),
        }
    }

    pub fn context(&self) -> &HashMap<String, Value> {
        &self.context
    }

    pub fn bookmarks(&self) -> &[String] {
        &self.bookmarks
    }

    pub fn database(&self) -> &Value {
        &self.database
    }
}

impl_try_from_message!(Route, Route);
