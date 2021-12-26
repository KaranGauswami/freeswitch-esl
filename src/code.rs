#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Code {
    Ok,
    Err,
    Unknown,
}

pub trait ParseCode {
    fn parse_code(self) -> Result<Code, crate::InboundError>;
}
impl ParseCode for &str {
    fn parse_code(self) -> Result<Code, crate::InboundError> {
        match self {
            "+OK" => Ok(Code::Ok),
            "-ERR" => Ok(Code::Err),
            _ => Ok(Code::Unknown),
        }
    }
}
