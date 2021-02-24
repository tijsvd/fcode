use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	/// The input was incomplete.
	#[error("unexpected end of input")]
	UnexpectedEndOfInput,
	/// The value read was not a valid `char`.
	#[error("invalid character")]
	InvalidChar,
	/// The byte array read did not contain valid UTF-8.
	#[error("invalid UTF-8 data")]
	InvalidUtf8,
	/// The input was longer than expected. If it was expected, please use [`from_bytes_more_data`](fn@crate::from_bytes_more_data).
	#[error("data beyond end")]
	DataBeyondEnd,
	/// The value read doesn't fit into the expected integer type.
	#[error("data value too large")]
	ValueOverflow,
	/// The wire type of the value doesn't match the expected type
	#[error("unexpected wire type")]
	UnexpectedWireType,
	/// A sequence with an odd number of elements was read, which is invalid for a map.
	#[error("invalid map encoding")]
	InvalidMap,
	/// Serde framework error.
	#[error("serialization error: {0}")]
	Serialization(String),
	/// Serde framework error.
	#[error("deserialization error: {0}")]
	Deserialization(String),
	/// I/O error in writer.
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
