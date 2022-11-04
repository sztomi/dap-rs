use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeserializationError {
  #[error("could not parse value '{value}' to enum variant of '{enum_name}'")]
  StringToEnumParseError { enum_name: String, value: String },
}

#[derive(Debug, Error)]
pub enum ServerError {
  #[error("I/O error")]
  IoError,

  #[error("Parse error")]
  ParseError,
}