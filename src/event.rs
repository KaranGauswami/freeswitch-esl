use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
/// Structure of event returned from freeswitch
pub struct Event {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Option<String>,
}
impl Event {
    /// Returns header from event
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
    /// Returns body from event
    pub fn body(&self) -> &Option<String> {
        &self.body
    }
}
