#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Code {
    Ok,
    Err,
    Unknown,
}

pub(crate) trait ParseCode {
    fn parse_code(self) -> Result<Code, crate::EslError>;
}
impl ParseCode for &str {
    fn parse_code(self) -> Result<Code, crate::EslError> {
        match self {
            "+OK" => Ok(Code::Ok),
            "-ERR" => Ok(Code::Err),
            _ => Ok(Code::Unknown),
        }
    }
}
