use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeserializationError {
  #[error("could not parse value '{value}' to enum variant of '{enum_name}'")]
  StringToEnumParseError { enum_name: String, value: String },
}
