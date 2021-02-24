//! The fcode crate delivers a simple binary serialization strategy for the
//! serde framework. Wire format resembles protobufs, but doesn't have field tags. Note that evolution depends
//! on lexical field ordering; you can *never* change the lexical order of fields, and fields must always
//! be added to the end of a struct/enum.
//!
//! The following evolutions are explicitly supported:
//!
//! * Add a field to the back of a struct. Deserialization of a longer struct is always possible, but
//!   in order to allow new code to deserialize an old object, added fields must be marked with
//!   `#[serde(default)]`.
//! * Extend a tuple struct, in the same way, *except* extending from a 1-field tuple (newtype) to something longer.
//! * Extend a struct enum variant or tuple enum variant in the same way.
//! * Change an anonymous tuple into a named tuple with the same field types.
//! * Change a named or anonymous tuple into a struct, as long as fields with the same type appear in the same order.
//! * Change a tuple enum variant into a struct variant, as long as fields with the same type appear in the same order.
//! * Change any value into a newtype struct (`i32` -> `Foo(i32)`).
//! * Change a struct variant or tuple variant into a newtype variant containing a struct/tuple with the same layout
//!   `Foo { x: i32, y: i32 }` -> `Foo(Foo)` (where `struct Foo { x: i32, y: i32 }`)
//! * Change the size of an integer (e.g. `i16` -> `i32`). Overly large values will cause deserialization error.
//! * Change the size of a float (`f32` -> `f64`) -- conversion back from `f64` to `f32` may silently overflow to infinity.
//! * Change a bool to an integer -- false maps to 0, true maps to anything not 0.
//! * Change a unit to bool (maps to false) or an integer (maps to 0).
//! * Change string to bytes. Non-UTF8 bytes will cause error when deserializing to string.
//! * Extend an enum with a new variant. To make this backwards compatible, the "old" code should have a unit variant
//!   marked with
//!   `#[serde(other)]`. It is therefore a good idea to always add such other / fallback variant for enums that
//!   may be extended in the future. The alternative is to always upgrade both sides before actually using the new variant.
//!
//! Explicitly not supported:
//!
//! * Change a newtype struct (`Foo(x)`) to a tuple (`Foo(x,y)`).
//! * Change the signedness of an integer (`i32` -> `u32`).
//! * Conditional skipping of fields (will panic), or skipping fields in serialization only (will cause deserialization badness).
//! * Serialization of sequences with unknown upfront length (e.g. iterators; will panic).
//!
//! Fields can be deprecated by changing them to unit in the receiver first, and then in the sender once all receivers
//! have been upgraded. Unit deserialisation blindly skips a field without actually checking the wire type. A unit field
//! takes a single byte on the wire. Vice versa, a field can be "undeprecated" (re-use of deprecated slot) by changing the
//! sender before the receiver.

mod de;
mod error;
mod ser;
mod wire;

#[cfg(test)]
mod tests;

pub use de::Deserializer;
pub use error::{Error, Result};
pub use ser::Serializer;

use serde::{Deserialize, Serialize};

/// Serialize a value into a new byte vector.
#[inline]
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
	T: Serialize + ?Sized,
{
	let mut v = Vec::new();
	to_writer(&mut v, value)?;
	Ok(v)
}

/// Serialize a value to a [`io::Write`](std::io::Write) implementation.
///
/// Use this to extend a `Vec<u8>`, or feed into some compressor.
#[inline]
pub fn to_writer<T, W>(w: &mut W, value: &T) -> Result<()>
where
	T: Serialize + ?Sized,
	W: std::io::Write,
{
	value.serialize(Serializer::new(w))
}

/// Deserialize a value from a byte slice.
pub fn from_bytes<'de, T>(data: &'de [u8]) -> Result<T>
where
	T: Deserialize<'de>,
{
	let mut de = Deserializer::from_bytes(data);
	let value = T::deserialize(&mut de)?;
	if de.remaining_len() > 0 {
		return Err(Error::DataBeyondEnd);
	}
	Ok(value)
}

/// Deserialize a value from a byte slice that may have more data.
///
/// Returns a pair of (value, size_read).
pub fn from_bytes_more_data<'de, T>(data: &'de [u8]) -> Result<(T, usize)>
where
	T: Deserialize<'de>,
{
	let mut de = Deserializer::from_bytes(data);
	let value = T::deserialize(&mut de)?;
	let consumed = data.len() - de.remaining_len();
	Ok((value, consumed))
}
