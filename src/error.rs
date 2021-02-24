use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("sequence or map is too long (max 4B entries)")]
	OverlyLongSequence,
	#[error("byte slice, string, or struct is too long (max 4GB)")]
	Oversized,
	#[error("unexpected end of input")]
	UnexpectedEndOfInput,
	#[error("invalid character")]
	InvalidChar,
	#[error("invalid UTF-8 data")]
	InvalidUtf8,
	#[error("data beyond end")]
	DataBeyondEnd,
	#[error("data value too large")]
	ValueOverflow,
	#[error("unexpected wire type")]
	UnexpectedWireType,
	#[error("invalid map encoding")]
	InvalidMap,
	#[error("serialization error: {0}")]
	Serialization(String),
	#[error("deserialization error: {0}")]
	Deserialization(String),
	#[error("I/O error: {0}")]
	IO(#[source] std::io::Error),
}

impl serde::ser::Error for Error {
	fn custom<T: std::fmt::Display>(msg: T) -> Self {
		Error::Serialization(msg.to_string())
	}
}

impl serde::de::Error for Error {
	fn custom<T: std::fmt::Display>(msg: T) -> Self {
		Error::Deserialization(msg.to_string())
	}
}

impl From<std::num::TryFromIntError> for Error {
	fn from(_e: std::num::TryFromIntError) -> Self {
		Error::ValueOverflow
	}
}

impl From<std::char::CharTryFromError> for Error {
	fn from(_e: std::char::CharTryFromError) -> Self {
		Error::InvalidChar
	}
}

impl From<std::str::Utf8Error> for Error {
	fn from(_e: std::str::Utf8Error) -> Self {
		Error::InvalidUtf8
	}
}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Self {
		Error::IO(e)
	}
}
