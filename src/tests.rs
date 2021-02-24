use super::*;
use serde::{de::DeserializeOwned, Serialize};

fn ser_de_r<T: Serialize + DeserializeOwned>(value: &T) -> Result<T> {
	from_bytes(&to_bytes(value)?)
}

macro_rules! ser_de {
	($v:expr) => {
		ser_de_r(&$v).unwrap()
	};
}

#[test]
fn test_basic_types() {
	assert_eq!(ser_de!(true), true);
	assert_eq!(ser_de!(false), false);
	assert_eq!(ser_de!(42i8), 42);
	assert_eq!(ser_de!(42i16), 42);
	assert_eq!(ser_de!(42i32), 42);
	assert_eq!(ser_de!(42i64), 42);
	assert_eq!(ser_de!(42u8), 42);
	assert_eq!(ser_de!(42u16), 42);
	assert_eq!(ser_de!(42u32), 42);
	assert_eq!(ser_de!(42u64), 42);
	assert_eq!(ser_de!(42.0f32), 42.0);
	assert_eq!(ser_de!(42.0f64), 42.0);
	assert_eq!(ser_de!('a'), 'a');
	assert_eq!(ser_de!('愛'), '愛');

	assert_eq!(ser_de!("foobar".to_string()), "foobar");

	assert_eq!(ser_de!(Some(42i32)), Some(42));
	assert_eq!(ser_de!(None::<i32>), None);

	assert_eq!(ser_de!(()), ());

	assert_eq!(ser_de!(vec![1, 2, 3]), vec![1, 2, 3]);
	assert_eq!(ser_de!(vec![1u8, 2u8, 3u8]), vec![1u8, 2u8, 3u8]);
	assert_eq!(ser_de!((1, 2, 3)), (1, 2, 3));
	assert_eq!(ser_de!([1, 2, 3]), [1, 2, 3]);
}

serde::serde_if_integer128! {
	#[test]
	fn test_128() {
		assert_eq!(ser_de!(42i128), 42);
		assert_eq!(ser_de!(42u128), 42);
		assert_eq!(ser_de!(i128::MAX), i128::MAX);
		assert_eq!(ser_de!(i128::MIN), i128::MIN);
		assert_eq!(ser_de!(u128::MAX), u128::MAX);
	}
}

#[test]
fn test_minmax() {
	assert_eq!(ser_de!(i8::MAX), i8::MAX);
	assert_eq!(ser_de!(i8::MIN), i8::MIN);
	assert_eq!(ser_de!(i16::MAX), i16::MAX);
	assert_eq!(ser_de!(i16::MIN), i16::MIN);
	assert_eq!(ser_de!(i32::MAX), i32::MAX);
	assert_eq!(ser_de!(i32::MIN), i32::MIN);
	assert_eq!(ser_de!(i64::MAX), i64::MAX);
	assert_eq!(ser_de!(i64::MIN), i64::MIN);

	assert_eq!(ser_de!(u8::MAX), u8::MAX);
	assert_eq!(ser_de!(u16::MAX), u16::MAX);
	assert_eq!(ser_de!(u32::MAX), u32::MAX);
	assert_eq!(ser_de!(u64::MAX), u64::MAX);
}

#[test]
fn test_borrowed() {
	let buf = to_bytes("foobar").unwrap();
	let s: &str = from_bytes(&buf).unwrap();
	assert_eq!(s, "foobar");

	// serde::ser::Serialize is not specifically implemented for &[u8]
	let buf = to_bytes(&serde_bytes::Bytes::new("foobar".as_bytes())).unwrap();
	let v: &[u8] = from_bytes(&buf).unwrap();
	assert_eq!(std::str::from_utf8(v).unwrap(), "foobar");

	// other slices can only be serialized
	let stuff = [1i32, 2i32, 3i32];
	let buf = to_bytes(&stuff[..]).unwrap();
	let v: Vec<i32> = from_bytes(&buf).unwrap();
	assert_eq!(v, vec![1, 2, 3]);

	// embedded in struct
	#[derive(Debug, Serialize, Deserialize)]
	struct Foo<'a> {
		i: i32,
		s: &'a str,
		#[serde(with = "serde_bytes")]
		b: &'a [u8],
		j: i32,
	}
	let f_in = Foo {
		i: 42,
		s: "foobar",
		b: "barfoo".as_bytes(),
		j: 43,
	};
	let buf = to_bytes(&f_in).unwrap();
	let f_out: Foo = from_bytes(&buf).unwrap();
	assert_eq!(f_out.i, 42);
	assert_eq!(f_out.j, 43);
	assert_eq!(f_out.s, "foobar");
	assert_eq!(std::str::from_utf8(f_out.b).unwrap(), "barfoo");
}

#[test]
fn test_struct() {
	#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
	struct Inner {
		x: i64,
	}
	#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
	struct Foo {
		x: i32,
		y: String,
		z: Vec<i32>,
		i: Inner,
	};

	let value = Foo {
		x: 42,
		y: "foobar".into(),
		z: vec![1, 2, 3],
		i: Inner { x: 43 },
	};
	assert_eq!(ser_de!(value.clone()), value);

	#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
	struct Bar(i32);
	assert_eq!(ser_de!(Bar(42)), Bar(42));

	#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
	struct Bar2(i32, String);
	assert_eq!(ser_de!(Bar2(42, "foobar".into())), Bar2(42, "foobar".into()));

	#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
	struct FooBar;
	assert_eq!(ser_de!(FooBar), FooBar);
}

#[test]
fn test_struct_vec() {
	#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
	struct Inner {
		x: i64,
		y: i64,
	}
	#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
	struct Outer {
		items: Vec<Inner>,
	}

	let value = Outer {
		items: vec![Inner { x: 1, y: 2 }, Inner { x: 3, y: 4 }],
	};
	assert_eq!(ser_de!(value.clone()), value);
}

#[test]
fn test_map() {
	use std::collections::HashMap;

	let value: HashMap<String, String> = vec![
		("foo".to_string(), "bar".to_string()),
		("aap".to_string(), "noot".to_string()),
	]
	.into_iter()
	.collect();
	assert_eq!(ser_de!(value.clone()), value);
}

#[test]
fn test_enum() {
	#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
	enum E {
		Unit,
		Newtype(i32),
		Tuple(i32, i32),
		Struct { x: i32, y: i32 },
	}

	assert_eq!(ser_de!(E::Unit), E::Unit);
	assert_eq!(ser_de!(E::Newtype(42)), E::Newtype(42));
	assert_eq!(ser_de!(E::Tuple(42, 43)), E::Tuple(42, 43));
	assert_eq!(ser_de!(E::Struct { x: 42, y: 43 }), E::Struct { x: 42, y: 43 });
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
struct LongStruct {
	x: i32,
	y: i32,
	#[serde(default)]
	z: i32,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct ShortStruct {
	x: i32,
	y: i32,
}
impl Into<ShortStruct> for LongStruct {
	fn into(self: LongStruct) -> ShortStruct {
		ShortStruct { x: self.x, y: self.y }
	}
}

#[test]
fn test_long_struct_to_short() {
	let src = vec![
		LongStruct { x: 1, y: 2, z: 3 },
		LongStruct { x: 4, y: 5, z: 6 },
		LongStruct { x: 7, y: 8, z: 9 },
	];
	let dest: Vec<ShortStruct> = from_bytes(&to_bytes(&src).unwrap()).unwrap();
	let expected: Vec<ShortStruct> = src.into_iter().map(Into::into).collect();
	assert_eq!(dest, expected);
}

#[test]
fn test_short_struct_to_long() {
	let expected = vec![
		LongStruct {
			x: 1,
			y: 2,
			..Default::default()
		},
		LongStruct {
			x: 4,
			y: 5,
			..Default::default()
		},
		LongStruct {
			x: 7,
			y: 8,
			..Default::default()
		},
	];
	let src: Vec<ShortStruct> = expected.iter().cloned().map(Into::into).collect();
	let dest: Vec<LongStruct> = from_bytes(&to_bytes(&src).unwrap()).unwrap();
	assert_eq!(dest, expected);
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
struct LongTuple(i32, i32, #[serde(default)] i32);
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct ShortTuple(i32, i32);
impl Into<ShortTuple> for LongTuple {
	fn into(self: LongTuple) -> ShortTuple {
		ShortTuple(self.0, self.1)
	}
}

#[test]
fn test_long_tuple_to_short() {
	let src = vec![LongTuple(1, 2, 3), LongTuple(4, 5, 6), LongTuple(7, 8, 9)];
	let dest: Vec<ShortTuple> = from_bytes(&to_bytes(&src).unwrap()).unwrap();
	let expected: Vec<ShortTuple> = src.into_iter().map(Into::into).collect();
	assert_eq!(dest, expected);
}

#[test]
fn test_short_tuple_to_long() {
	let expected = vec![LongTuple(1, 2, 0), LongTuple(4, 5, 0), LongTuple(7, 8, 0)];
	let src: Vec<ShortTuple> = expected.iter().cloned().map(Into::into).collect();
	let dest: Vec<LongTuple> = from_bytes(&to_bytes(&src).unwrap()).unwrap();
	assert_eq!(dest, expected);
}

#[test]
fn anonymous_tuple_to_named() {
	let expected = vec![LongTuple(1, 2, 0), LongTuple(4, 5, 0), LongTuple(7, 8, 0)];
	let src: Vec<(i32, i32)> = expected.iter().cloned().map(|LongTuple(x, y, _)| (x, y)).collect();
	let dest: Vec<LongTuple> = from_bytes(&to_bytes(&src).unwrap()).unwrap();
	assert_eq!(dest, expected);
}

#[test]
fn tuple_to_struct() {
	let expected = vec![
		LongStruct {
			x: 1,
			y: 2,
			..Default::default()
		},
		LongStruct {
			x: 4,
			y: 5,
			..Default::default()
		},
		LongStruct {
			x: 7,
			y: 8,
			..Default::default()
		},
	];
	let src: Vec<ShortTuple> = expected.iter().cloned().map(|v| ShortTuple(v.x, v.y)).collect();
	let dest: Vec<LongStruct> = from_bytes(&to_bytes(&src).unwrap()).unwrap();
	assert_eq!(dest, expected);
}

#[test]
fn type_to_newtype() {
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	struct Foo(i32, i32);
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	struct Bar(Foo);

	let src = vec![Foo(1, 2), Foo(3, 4), Foo(5, 6)];
	let dest: Vec<Bar> = from_bytes(&to_bytes(&src).unwrap()).unwrap();
	let expected: Vec<Bar> = src.iter().cloned().map(|f| Bar(f)).collect();
	assert_eq!(dest, expected);
}

#[test]
fn extend_struct_variant() {
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	enum Short {
		Foo { x: i32, y: i32 },
	}
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	enum Long {
		Foo {
			x: i32,
			y: i32,
			#[serde(default)]
			z: i32,
		},
	}

	let short = vec![
		Short::Foo { x: 1, y: 2 },
		Short::Foo { x: 4, y: 5 },
		Short::Foo { x: 7, y: 8 },
	];
	let long: Vec<Long> = short
		.iter()
		.map(|&Short::Foo { x, y }| Long::Foo { x, y, z: 0 })
		.collect();

	let dest: Vec<Short> = from_bytes(&to_bytes(&long).unwrap()).unwrap();
	assert_eq!(dest, short);

	let dest: Vec<Long> = from_bytes(&to_bytes(&short).unwrap()).unwrap();
	assert_eq!(dest, long);
}

#[test]
fn tuple_variant_to_struct() {
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	enum Tuple {
		Foo(i32, i32),
	}
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	enum Struct {
		Foo { x: i32, y: i32 },
	}

	let src = vec![Tuple::Foo(1, 2), Tuple::Foo(3, 4), Tuple::Foo(5, 6)];
	let dest: Vec<Struct> = from_bytes(&to_bytes(&src).unwrap()).unwrap();
	let expected: Vec<Struct> = src.iter().map(|&Tuple::Foo(x, y)| Struct::Foo { x, y }).collect();
	assert_eq!(dest, expected);
}

#[test]
fn struct_variant_to_newtype_struct() {
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	enum E1 {
		Foo { x: i32, y: i32 },
	}
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	struct Foo {
		x: i32,
		y: i32,
	}
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	enum E2 {
		Foo(Foo),
	}

	let src = vec![E1::Foo { x: 1, y: 2 }, E1::Foo { x: 3, y: 4 }, E1::Foo { x: 5, y: 6 }];
	let dest: Vec<E2> = from_bytes(&to_bytes(&src).unwrap()).unwrap();
	let expected: Vec<E2> = src.iter().map(|&E1::Foo { x, y }| E2::Foo(Foo { x, y })).collect();
	assert_eq!(dest, expected);
}

#[test]
fn tuple_variant_to_newtype_tuple() {
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	enum E1 {
		Foo(i32, i32),
	}
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	enum E2 {
		Foo((i32, i32)),
	}

	let src = vec![E1::Foo(1, 2), E1::Foo(3, 4), E1::Foo(5, 6)];
	let dest: Vec<E2> = from_bytes(&to_bytes(&src).unwrap()).unwrap();
	let expected: Vec<E2> = src.iter().map(|&E1::Foo(x, y)| E2::Foo((x, y))).collect();
	assert_eq!(dest, expected);
}

#[test]
fn extend_enum() {
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	enum E1 {
		X(i32),
		Y(i64),
	}
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	enum E2 {
		X(i32),
		Y(i64),
		Z(String),
	}

	let src = vec![E1::X(42), E1::Y(43)];
	let dest: Vec<E2> = from_bytes(&to_bytes(&src).unwrap()).unwrap();
	assert_eq!(dest, vec![E2::X(42), E2::Y(43)]);

	// but vice versa should throw
	let src = E2::Z("foobar".into());
	let maybe_dest: std::result::Result<E1, _> = from_bytes(&to_bytes(&src).unwrap());
	assert!(maybe_dest.is_err());
}

#[test]
fn extend_enum_with_other() {
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	enum E1 {
		X(i32),
		Y(i64),
		#[serde(other)]
		Other,
	}
	#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	enum E2 {
		X(i32),
		Y(i64),
		Z(String),
	}

	let src = vec![E2::X(42), E2::Y(43), E2::Z("foobar".into())];
	let dest: Vec<E1> = from_bytes(&to_bytes(&src).unwrap()).unwrap();
	assert_eq!(dest, vec![E1::X(42), E1::Y(43), E1::Other,]);
}

#[test]
fn skip_field() {
	#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
	struct Foo {
		x: i32,
		#[serde(skip)]
		y: i32,
		z: i32,
	}

	assert_eq!(ser_de!(Foo { x: 42, y: 43, z: 44 }), Foo { x: 42, y: 0, z: 44 });
}
