use crate::{
	wire::{self, WireType},
	Error, Result,
};
use serde::{ser, Serialize};
use std::io::Write;

pub struct Serializer<'a, B: Write + 'a> {
	writer: &'a mut B,
}

impl<'a, B: Write + 'a> Serializer<'a, B> {
	pub fn new(writer: &'a mut B) -> Self {
		Serializer { writer }
	}
}

impl<'a, B: Write + 'a> ser::Serializer for Serializer<'a, B> {
	type Ok = ();
	type Error = Error;
	type SerializeSeq = Self;
	type SerializeMap = Self;
	type SerializeTuple = Self;
	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Self;
	type SerializeStruct = Self;
	type SerializeStructVariant = Self;

	#[inline]
	fn serialize_i8(self, v: i8) -> Result<()> {
		self.serialize_i64(v as i64)
	}

	#[inline]
	fn serialize_i16(self, v: i16) -> Result<()> {
		self.serialize_i64(v as i64)
	}

	#[inline]
	fn serialize_i32(self, v: i32) -> Result<()> {
		self.serialize_i64(v as i64)
	}

	#[inline]
	fn serialize_i64(self, v: i64) -> Result<()> {
		self.serialize_u64(wire::zigzag_encode(v))
	}

	#[inline]
	fn serialize_u8(self, v: u8) -> Result<()> {
		self.serialize_u64(v as u64)
	}

	#[inline]
	fn serialize_u16(self, v: u16) -> Result<()> {
		self.serialize_u64(v as u64)
	}

	#[inline]
	fn serialize_u32(self, v: u32) -> Result<()> {
		self.serialize_u64(v as u64)
	}

	#[inline]
	fn serialize_u64(self, v: u64) -> Result<()> {
		wire::write_varint(self.writer, WireType::Int, v)
	}

	#[inline]
	fn serialize_bool(self, v: bool) -> Result<()> {
		self.serialize_u8(if v { 1 } else { 0 })
	}

	serde::serde_if_integer128! {
		#[inline]
		fn serialize_i128(self, v: i128) -> Result<()> {
			self.serialize_u128(wire::zigzag_encode_128(v))
		}

		#[inline]
		fn serialize_u128(self, v: u128) -> Result<()> {
			wire::write_varint_128(self.writer, WireType::Int, v)
		}
	}

	#[inline]
	fn serialize_char(self, v: char) -> Result<()> {
		self.serialize_u32(v as u32)
	}

	#[inline]
	fn serialize_f32(self, v: f32) -> Result<()> {
		let mut b = [0u8; 5];
		b[0] = WireType::Fixed32 as u8;
		(&mut b[1..]).copy_from_slice(&v.to_le_bytes()[..]);
		self.writer.write_all(&b[..])?;
		Ok(())
	}

	#[inline]
	fn serialize_f64(self, v: f64) -> Result<()> {
		let mut b = [0u8; 9];
		b[0] = WireType::Fixed64 as u8;
		(&mut b[1..]).copy_from_slice(&v.to_le_bytes()[..]);
		self.writer.write_all(&b[..])?;
		Ok(())
	}

	#[inline]
	fn serialize_str(self, v: &str) -> Result<()> {
		self.serialize_bytes(v.as_bytes())
	}

	#[inline]
	fn serialize_bytes(self, v: &[u8]) -> Result<()> {
		wire::write_varint(self.writer, WireType::Bytes, v.len() as u64)?;
		self.writer.write_all(v)?;
		Ok(())
	}

	#[inline]
	fn serialize_none(self) -> Result<()> {
		self.serialize_unit_variant("Option", 0, "None")
	}

	#[inline]
	fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<()> {
		self.serialize_newtype_variant("Option", 1, "Some", value)
	}

	#[inline]
	fn serialize_unit(self) -> Result<()> {
		self.serialize_bool(false)
	}

	#[inline]
	fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
		self.serialize_unit()
	}

	#[inline]
	fn serialize_unit_variant(self, _name: &'static str, variant_index: u32, _variant: &'static str) -> Result<()> {
		wire::write_varint(self.writer, WireType::Variant, variant_index as u64)?;
		self.serialize_unit()
	}

	#[inline]
	fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _name: &'static str, value: &T) -> Result<()> {
		value.serialize(self)
	}

	#[inline]
	fn serialize_newtype_variant<T: ?Sized + Serialize>(
		self,
		_name: &'static str,
		variant_index: u32,
		_variant: &'static str,
		value: &T,
	) -> Result<()> {
		wire::write_varint(self.writer, WireType::Variant, variant_index as u64)?;
		value.serialize(self)
	}

	#[inline]
	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
		// we have a single wire type left -- could use it; but I don't think this case is very common?
		let len = len.expect("sequences with unknown length not supported");
		self.serialize_tuple(len)
	}

	#[inline]
	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
		wire::write_varint(self.writer, WireType::Sequence, len as u64)?;
		Ok(self)
	}

	#[inline]
	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
		let len = len.expect("maps with unknown length not supported");
		self.serialize_tuple(len * 2)
	}

	#[inline]
	fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct> {
		self.serialize_tuple(len)
	}

	#[inline]
	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		_variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleVariant> {
		wire::write_varint(self.writer, WireType::Variant, variant_index as u64)?;
		self.serialize_tuple(len)
	}

	#[inline]
	fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
		self.serialize_tuple(len)
	}

	#[inline]
	fn serialize_struct_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeStructVariant> {
		self.serialize_tuple_variant(name, variant_index, variant, len)
	}

	#[inline]
	fn is_human_readable(&self) -> bool {
		false
	}
}

impl<'a, B: Write + 'a> ser::SerializeSeq for Serializer<'a, B> {
	type Ok = ();
	type Error = Error;
	#[inline]
	fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
		value.serialize(Serializer { writer: self.writer })
	}
	#[inline]
	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a, B: Write + 'a> ser::SerializeMap for Serializer<'a, B> {
	type Ok = ();
	type Error = Error;
	#[inline]
	fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
		key.serialize(Serializer { writer: self.writer })
	}
	#[inline]
	fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
		value.serialize(Serializer { writer: self.writer })
	}
	#[inline]
	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a, B: Write + 'a> ser::SerializeStruct for Serializer<'a, B> {
	type Ok = ();
	type Error = Error;
	#[inline]
	fn serialize_field<T: ?Sized + Serialize>(&mut self, _key: &'static str, value: &T) -> Result<()> {
		value.serialize(Serializer { writer: self.writer })
	}
	fn skip_field(&mut self, _key: &'static str) -> Result<()> {
		panic!("optionally skipped fields are not supported")
	}
	#[inline]
	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a, B: Write + 'a> ser::SerializeStructVariant for Serializer<'a, B> {
	type Ok = ();
	type Error = Error;
	#[inline]
	fn serialize_field<T: ?Sized + Serialize>(&mut self, _key: &'static str, value: &T) -> Result<()> {
		value.serialize(Serializer { writer: self.writer })
	}
	fn skip_field(&mut self, _key: &'static str) -> Result<()> {
		panic!("optionally skipped fields are not supported")
	}
	#[inline]
	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a, B: Write + 'a> ser::SerializeTuple for Serializer<'a, B> {
	type Ok = ();
	type Error = Error;
	#[inline]
	fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
		value.serialize(Serializer { writer: self.writer })
	}
	#[inline]
	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a, B: Write + 'a> ser::SerializeTupleVariant for Serializer<'a, B> {
	type Ok = ();
	type Error = Error;
	#[inline]
	fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
		value.serialize(Serializer { writer: self.writer })
	}
	#[inline]
	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a, B: Write + 'a> ser::SerializeTupleStruct for Serializer<'a, B> {
	type Ok = ();
	type Error = Error;
	#[inline]
	fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
		value.serialize(Serializer { writer: self.writer })
	}
	#[inline]
	fn end(self) -> Result<()> {
		Ok(())
	}
}
