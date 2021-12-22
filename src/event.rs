use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}
impl Event {
    pub fn headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }
    pub fn body(&self) -> Option<String> {
        self.body.clone()
    }
}
