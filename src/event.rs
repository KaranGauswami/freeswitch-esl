use std::collections::HashMap;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Structure of event returned from freeswitch
pub struct Event {
    pub(crate) headers: HashMap<String, Value>,
    pub(crate) body: Option<String>,
}
impl Event {
    /// Returns header from event
    pub fn headers(&self) -> &HashMap<String, Value> {
        &self.headers
    }
    /// Returns body from event
    pub fn body(&self) -> &Option<String> {
        &self.body
    }
}
