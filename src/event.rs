use std::collections::HashMap;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub headers: HashMap<String, Value>,
    pub body: Option<String>,
}
impl Event {
    pub fn headers(&self) -> HashMap<String, Value> {
        self.headers.clone()
    }
    pub fn body(&self) -> Option<String> {
        self.body.clone()
    }
}
