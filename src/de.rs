use crate::{
	wire::{self, WireType},
	Error, Result,
};
use serde::de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use std::convert::TryInto;

pub struct Deserializer<'de> {
	input: &'de [u8],
}

impl<'de> Deserializer<'de> {
	#[inline]
	pub fn from_bytes(input: &'de [u8]) -> Self {
		Deserializer { input }
	}

	#[inline]
	pub fn remaining_len(&self) -> usize {
		self.input.len()
	}

	#[inline]
	fn check(&self, n: usize) -> Result<()> {
		if n > self.input.len() {
			Err(Error::UnexpectedEndOfInput)
		} else {
			Ok(())
		}
	}

	#[inline]
	fn read(&mut self, n: usize) -> Result<&'de [u8]> {
		self.check(n)?;
		let (value, remainder) = self.input.split_at(n);
		self.input = remainder;
		Ok(value)
	}

	#[inline]
	fn read_32(&mut self) -> Result<[u8; 4]> {
		Ok(self.read(4)?.try_into().unwrap())
	}

	#[inline]
	fn read_64(&mut self) -> Result<[u8; 8]> {
		Ok(self.read(8)?.try_into().unwrap())
	}

	#[inline]
	fn read_byte(&mut self) -> Result<u8> {
		let &b = self.input.first().ok_or(Error::UnexpectedEndOfInput)?;
		self.input = &self.input[1..];
		Ok(b)
	}

	#[inline]
	fn consume(&mut self, len: usize) {
		self.input = &self.input[len..];
	}

	#[inline]
	fn read_varint(&mut self, tagbyte: u8) -> Result<u64> {
		let (value, len) = wire::read_varint(tagbyte, self.input)?;
		self.consume(len);
		Ok(value)
	}

	serde::serde_if_integer128! {
		fn read_varint_128(&mut self, tagbyte: u8) -> Result<u128> {
			let (value, len) = wire::read_varint_128(tagbyte, self.input)?;
			self.consume(len);
			Ok(value)
		}
	}

	#[inline]
	fn skip(&mut self) -> Result<()> {
		let tagbyte = self.read_byte()?;
		match wire::read_wiretype(tagbyte) {
			WireType::Int => {
				let len = wire::skip_varint(tagbyte, self.input)?;
				self.consume(len);
			}
			WireType::Fixed32 => {
				self.read_32()?;
			}
			WireType::Fixed64 => {
				self.read_64()?;
			}
			WireType::Sequence => {
				let len = self.read_varint(tagbyte)?;
				for _ in 0..len {
					self.skip()?;
				}
			}
			WireType::Bytes => {
				let len = self.read_varint(tagbyte)?;
				self.read(len as usize)?;
			}
			WireType::Variant => {
				self.read_varint(tagbyte)?;
				self.skip()?;
			}
			_ => {
				return Err(Error::UnexpectedWireType);
			}
		}
		Ok(())
	}
}

impl<'de> de::Deserializer<'de> for &'_ mut Deserializer<'de> {
	type Error = Error;

	fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
		unimplemented!()
	}

	fn is_human_readable(&self) -> bool {
		false
	}

	#[inline]
	fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		if wire::read_wiretype(tagbyte) != WireType::Int {
			return Err(Error::UnexpectedWireType);
		}
		let v: i8 = wire::zigzag_decode(self.read_varint(tagbyte)?).try_into()?;
		visitor.visit_i8(v)
	}

	#[inline]
	fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		if wire::read_wiretype(tagbyte) != WireType::Int {
			return Err(Error::UnexpectedWireType);
		}
		let v: i16 = wire::zigzag_decode(self.read_varint(tagbyte)?).try_into()?;
		visitor.visit_i16(v)
	}

	#[inline]
	fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		// for 32-bit and 64-bit ints, we allow the Fixed32/Fixed64 wire type, for the
		// case where perhaps someday we can tell serde that a value is not suitable
		// as a varint (e.g. a hash value or other semi-random ID).
		let tagbyte = self.read_byte()?;
		let v: i32 = match wire::read_wiretype(tagbyte) {
			WireType::Int => wire::zigzag_decode(self.read_varint(tagbyte)?).try_into()?,
			WireType::Fixed32 => i32::from_le_bytes(self.read_32()?),
			_ => return Err(Error::UnexpectedWireType),
		};
		visitor.visit_i32(v)
	}

	#[inline]
	fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		let v: i64 = match wire::read_wiretype(tagbyte) {
			WireType::Int => wire::zigzag_decode(self.read_varint(tagbyte)?),
			WireType::Fixed64 => i64::from_le_bytes(self.read_64()?),
			_ => return Err(Error::UnexpectedWireType),
		};
		visitor.visit_i64(v)
	}

	#[inline]
	fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		if wire::read_wiretype(tagbyte) != WireType::Int {
			return Err(Error::UnexpectedWireType);
		}
		let v: u8 = self.read_varint(tagbyte)?.try_into()?;
		visitor.visit_u8(v)
	}

	#[inline]
	fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		if wire::read_wiretype(tagbyte) != WireType::Int {
			return Err(Error::UnexpectedWireType);
		}
		let v: u16 = self.read_varint(tagbyte)?.try_into()?;
		visitor.visit_u16(v)
	}

	#[inline]
	fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		let v: u32 = match wire::read_wiretype(tagbyte) {
			WireType::Int => self.read_varint(tagbyte)?.try_into()?,
			WireType::Fixed32 => u32::from_le_bytes(self.read_32()?),
			_ => return Err(Error::UnexpectedWireType),
		};
		visitor.visit_u32(v)
	}

	#[inline]
	fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		let v: u64 = match wire::read_wiretype(tagbyte) {
			WireType::Int => self.read_varint(tagbyte)?,
			WireType::Fixed64 => u64::from_le_bytes(self.read_64()?),
			_ => return Err(Error::UnexpectedWireType),
		};
		visitor.visit_u64(v)
	}

	#[inline]
	fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		let v = match wire::read_wiretype(tagbyte) {
			WireType::Fixed32 => f32::from_le_bytes(self.read_32()?),
			WireType::Fixed64 => f64::from_le_bytes(self.read_64()?) as f32, // truncate silently
			_ => return Err(Error::UnexpectedWireType),
		};
		visitor.visit_f32(v)
	}

	#[inline]
	fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		let v = match wire::read_wiretype(tagbyte) {
			WireType::Fixed32 => f32::from_le_bytes(self.read_32()?) as f64,
			WireType::Fixed64 => f64::from_le_bytes(self.read_64()?),
			_ => return Err(Error::UnexpectedWireType),
		};
		visitor.visit_f64(v)
	}

	#[inline]
	fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let v: u64 = de::Deserialize::deserialize(self)?;
		visitor.visit_bool(v != 0)
	}

	serde::serde_if_integer128! {
		#[inline]
		fn deserialize_i128<V:Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
			let tagbyte = self.read_byte()?;
			if wire::read_wiretype(tagbyte) != WireType::Int {
				return Err(Error::UnexpectedWireType);
			}
			let v  = wire::zigzag_decode_128(self.read_varint_128(tagbyte)?);
			visitor.visit_i128(v)
		}

		#[inline]
		fn deserialize_u128<V:Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
			let tagbyte = self.read_byte()?;
			if wire::read_wiretype(tagbyte) != WireType::Int {
				return Err(Error::UnexpectedWireType);
			}
			let v  = self.read_varint_128(tagbyte)?;
			visitor.visit_u128(v)
		}
	}

	#[inline]
	fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		use std::convert::TryFrom;
		let v: u32 = de::Deserialize::deserialize(self)?;
		let c = char::try_from(v)?;
		visitor.visit_char(c)
	}

	#[inline]
	fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let bytes: &'de [u8] = de::Deserialize::deserialize(self)?;
		let s = std::str::from_utf8(bytes)?;
		visitor.visit_borrowed_str(s)
	}

	#[inline]
	fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		self.deserialize_str(visitor)
	}

	#[inline]
	fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		if wire::read_wiretype(tagbyte) != WireType::Bytes {
			return Err(Error::UnexpectedWireType);
		}
		let len = self.read_varint(tagbyte)?;
		let bytes = self.read(len as usize)?;
		visitor.visit_borrowed_bytes(bytes)
	}

	#[inline]
	fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		self.deserialize_bytes(visitor)
	}

	#[inline]
	fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		if wire::read_wiretype(tagbyte) != WireType::Variant {
			return Err(Error::UnexpectedWireType);
		}
		let b = self.read_varint(tagbyte)?;
		if b == 0 {
			self.skip()?;
			visitor.visit_none()
		} else {
			visitor.visit_some(self)
		}
	}

	#[inline]
	fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		self.skip()?;
		visitor.visit_unit()
	}

	#[inline]
	fn deserialize_unit_struct<V: Visitor<'de>>(self, _name: &'static str, visitor: V) -> Result<V::Value> {
		self.skip()?;
		visitor.visit_unit()
	}

	#[inline]
	fn deserialize_newtype_struct<V: Visitor<'de>>(self, _name: &'static str, visitor: V) -> Result<V::Value> {
		visitor.visit_newtype_struct(self)
	}

	#[inline]
	fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		if wire::read_wiretype(tagbyte) != WireType::Sequence {
			return Err(Error::UnexpectedWireType);
		}
		let n = self.read_varint(tagbyte)? as usize;
		visitor.visit_seq(SeqRead {
			d: self,
			nread: n,
			nreturn: n,
		})
	}

	#[inline]
	fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		if wire::read_wiretype(tagbyte) != WireType::Sequence {
			return Err(Error::UnexpectedWireType);
		}
		let n = self.read_varint(tagbyte)? as usize;
		visitor.visit_seq(SeqRead {
			d: self,
			nread: n,
			nreturn: std::cmp::min(n, len),
		})
	}

	#[inline]
	fn deserialize_tuple_struct<V: Visitor<'de>>(
		self,
		_name: &'static str,
		len: usize,
		visitor: V,
	) -> Result<V::Value> {
		self.deserialize_tuple(len, visitor)
	}

	#[inline]
	fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		let tagbyte = self.read_byte()?;
		if wire::read_wiretype(tagbyte) != WireType::Sequence {
			return Err(Error::UnexpectedWireType);
		}
		let n = self.read_varint(tagbyte)? as usize;
		if n % 2 != 0 {
			return Err(Error::InvalidMap);
		}
		visitor.visit_map(SeqRead {
			d: self,
			nread: n,
			nreturn: n / 2,
		})
	}

	#[inline]
	fn deserialize_struct<V: Visitor<'de>>(
		self,
		_name: &'static str,
		fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value> {
		self.deserialize_tuple(fields.len(), visitor)
	}

	#[inline]
	fn deserialize_enum<V: Visitor<'de>>(
		self,
		_name: &'static str,
		_variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value> {
		visitor.visit_enum(self)
	}

	#[inline]
	fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		self.deserialize_u32(visitor)
	}

	fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
		self.skip()?;
		visitor.visit_unit()
	}
}

impl<'de, 'a> EnumAccess<'de> for &'a mut Deserializer<'de> {
	type Error = Error;
	type Variant = SeqRead<'de, 'a>;

	#[inline]
	fn variant_seed<V: de::DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
		// we want to read a u32, but with a different wire type, so can't simply use
		// deserializer -- read the discriminant then force it into a deserializer
		let tagbyte = self.read_byte()?;
		if wire::read_wiretype(tagbyte) != WireType::Variant {
			return Err(Error::UnexpectedWireType)?;
		}
		let discr: u32 = self.read_varint(tagbyte)?.try_into()?;
		use de::IntoDeserializer;
		let d: de::value::U32Deserializer<Error> = discr.into_deserializer();
		let val = seed.deserialize(d)?;
		Ok((
			val,
			SeqRead {
				d: self,
				nread: 1,
				nreturn: 1,
			},
		))
	}
}

pub struct SeqRead<'de, 'a> {
	d: &'a mut Deserializer<'de>,
	nread: usize,
	nreturn: usize,
}

// this is for the case when an overly long struct or tuple is received, or not the entire sequence is read for another
// reason, or the variant is not accessed (in #[serde(other)])
impl Drop for SeqRead<'_, '_> {
	#[inline]
	fn drop(&mut self) {
		while self.nread > 0 {
			if self.d.skip().is_err() {
				break;
			}
			self.nread -= 1;
		}
	}
}

impl<'de> SeqAccess<'de> for SeqRead<'de, '_> {
	type Error = Error;
	#[inline]
	fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
		if self.nreturn == 0 {
			return Ok(None);
		}
		self.nreturn -= 1;
		debug_assert!(self.nread > 0);
		self.nread -= 1;
		Ok(Some(seed.deserialize(&mut *self.d)?))
	}
	#[inline]
	fn size_hint(&self) -> Option<usize> {
		Some(self.nreturn)
	}
}

impl<'de> VariantAccess<'de> for SeqRead<'de, '_> {
	type Error = Error;

	#[inline]
	fn unit_variant(mut self) -> Result<()> {
		self.nread -= 1;
		self.d.skip()
	}
	#[inline]
	fn newtype_variant_seed<V: de::DeserializeSeed<'de>>(mut self, seed: V) -> Result<V::Value> {
		self.nread -= 1;
		seed.deserialize(&mut *self.d)
	}
	#[inline]
	fn tuple_variant<V: Visitor<'de>>(mut self, len: usize, visitor: V) -> Result<V::Value> {
		self.nread -= 1;
		use de::Deserializer;
		self.d.deserialize_tuple(len, visitor)
	}
	#[inline]
	fn struct_variant<V: Visitor<'de>>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value> {
		self.tuple_variant(fields.len(), visitor)
	}
}

impl<'de> MapAccess<'de> for SeqRead<'de, '_> {
	type Error = Error;
	#[inline]
	fn next_key_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
		if self.nreturn == 0 {
			return Ok(None);
		}
		self.nreturn -= 1;
		debug_assert!(self.nread > 0);
		self.nread -= 1;
		Ok(Some(seed.deserialize(&mut *self.d)?))
	}
	#[inline]
	fn next_value_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<T::Value> {
		debug_assert!(self.nread > 0);
		self.nread -= 1;
		seed.deserialize(&mut *self.d)
	}
	#[inline]
	fn size_hint(&self) -> Option<usize> {
		Some(self.nreturn)
	}
}
