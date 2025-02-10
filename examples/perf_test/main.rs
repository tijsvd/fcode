use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Instant;

fn test_ser_de<T: Serialize + DeserializeOwned>(value: &T, slug: &str, mut checker: impl FnMut(&T)) {
	println!("** testing: {} **", slug);
	test_ser_de_detail(
		value,
		&mut checker,
		|buf, val| fcode::to_writer(buf, val).unwrap(),
		|buf| fcode::from_bytes(buf).unwrap(),
		"fcode",
	);
	test_ser_de_detail(
		value,
		&mut checker,
		|buf, val| bincode::serialize_into(buf, val).unwrap(),
		|buf| bincode::deserialize(buf).unwrap(),
		"bincode",
	);
	test_ser_de_detail(
		value,
		&mut checker,
		|buf, val| serde_json::to_writer(buf, val).unwrap(),
		|buf| serde_json::from_slice(buf).unwrap(),
		"json",
	);
}

fn test_ser_de_detail<T>(
	value: &T,
	checker: &mut impl FnMut(&T),
	mut encode: impl FnMut(&mut Vec<u8>, &T),
	mut decode: impl FnMut(&[u8]) -> T,
	detail_name: &str,
) {
	// warm-up and allocate
	let mut buffer = Vec::new();
	encode(&mut buffer, value);
	checker(&decode(&buffer));

	const N: u64 = 1000000;

	let start = Instant::now();
	for _ in 0..N {
		buffer.clear();
		encode(&mut buffer, value);
		let received: T = decode(&buffer);
		checker(&received);
	}
	let elapsed = start.elapsed();
	println!(
		"{} sz={} bytes; time={} ns/roundtrip",
		detail_name,
		buffer.len(),
		elapsed.as_nanos() as u64 / N,
	);
}

mod benchfb {
	use serde::{Deserialize, Serialize};
	#[derive(Serialize, Deserialize)]
	pub enum Enum {
		Apples,
		Pears,
		Bananas,
	}
	#[derive(Serialize, Deserialize)]
	pub struct Foo {
		pub id: u64,
		pub count: i16,
		pub prefix: i8,
		pub length: u32,
	}
	#[derive(Serialize, Deserialize)]
	pub struct Bar {
		pub parent: Foo,
		pub time: i32,
		pub ratio: f32,
		pub size: u16,
	}
	#[derive(Serialize, Deserialize)]
	pub struct FooBar {
		pub sibling: Bar,
		pub name: String,
		pub rating: f64,
		pub postfix: u8,
	}
	#[derive(Serialize, Deserialize)]
	pub struct FooBarContainer {
		pub list: Vec<FooBar>,
		pub initialized: bool,
		pub fruit: Enum,
		pub location: String,
	}
}

mod protobench {
	include!("monster.rs");
}

fn main() {
	// // prost_build::compile_protos(&["monster.proto"], &["."]).unwrap();

	test_ser_de(&42i32, "the simplest int", |v| assert_eq!(*v, 42));

	#[derive(Serialize, Deserialize)]
	struct SimpleStructOfScalars {
		x: i32,
		y: f64,
		z: i64,
		a1: i32,
		a2: i32,
		a3: i32,
	}
	test_ser_de(
		&SimpleStructOfScalars {
			x: 42,
			y: 684.0,
			z: 84,
			a1: 1,
			a2: 2,
			a3: 3,
		},
		"a simple struct of scalars",
		|v| assert_eq!(v.x, 42),
	);

	#[derive(Serialize, Deserialize)]
	struct SimpleStructWithNestedArray {
		x: i32,
		y: f64,
		z: i64,
		a: [i32; 3],
	}
	test_ser_de(
		&SimpleStructWithNestedArray {
			x: 42,
			y: 684.0,
			z: 84,
			a: [1, 2, 3],
		},
		"a simple struct of scalars with fixed array",
		|v| assert_eq!(v.x, 42),
	);

	#[derive(Serialize, Deserialize)]
	struct NestedStruct {
		x: i32,
		y: i32,
		s: SimpleStructOfScalars,
	}
	test_ser_de(
		&NestedStruct {
			x: 42,
			y: 42,
			s: SimpleStructOfScalars {
				x: 42,
				y: 684.0,
				z: 84,
				a1: 1,
				a2: 2,
				a3: 3,
			},
		},
		"simple nested struct",
		|v| assert_eq!(v.x, 42),
	);

	#[derive(Serialize, Deserialize)]
	struct StructWithString {
		x: i32,
		y: i32,
		s: String,
	}
	test_ser_de(
		&StructWithString {
			x: 42,
			y: 43,
			s: "to be or not to be".to_string(),
		},
		"struct with string",
		|v| assert_eq!(v.x, 42),
	);

	#[derive(Serialize, Deserialize)]
	struct StructWithLargeVector {
		x: i32,
		y: i32,
		v: Vec<i32>,
	}
	test_ser_de(
		&StructWithLargeVector {
			x: 42,
			y: 43,
			v: vec![42; 100],
		},
		"struct with largish vector",
		|v| assert_eq!(v.x, 42),
	);
	test_ser_de(
		&benchfb::FooBarContainer {
			list: (0i32..3)
				.map(|i| benchfb::FooBar {
					sibling: benchfb::Bar {
						parent: benchfb::Foo {
							id: 0xABADCAFEABADCAFE + i as u64,
							count: 10000 + i as i16,
							prefix: '@' as i8 + i as i8,
							length: 1000000 + i as u32,
						},
						time: 123456 + i,
						ratio: 3.141519 + i as f32,
						size: 10000 + i as u16,
					},
					name: "Hello, World!".into(),
					rating: 3.1415432432445543 + i as f64,
					postfix: b'!' + i as u8,
				})
				.collect(),
			initialized: true,
			fruit: benchfb::Enum::Bananas,
			location: "http://google.com/flatbuffers/".into(),
		},
		"google's monster benchmark object",
		|v| assert!(v.initialized),
	);

	test_ser_de_detail(
		&protobench::FooBarContainer {
			list: (0i32..3)
				.map(|i| protobench::FooBar {
					sibling: Some(protobench::Bar {
						parent: Some(protobench::Foo {
							id: 0xABADCAFEABADCAFE + i as u64,
							count: 10000 + i,
							prefix: '@' as i32 + i,
							length: 1000000 + i as u32,
						}),
						time: 123456 + i,
						ratio: 3.141519 + i as f32,
						size: 10000 + i as u32,
					}),
					name: "Hello, World!".into(),
					rating: 3.141543243244554 + i as f64,
					postfix: '!' as u32 + i as u32,
				})
				.collect(),
			initialized: true,
			fruit: protobench::Enum::Bananas as i32,
			location: "http://google.com/flatbuffers/".into(),
		},
		&mut |v| assert!(v.initialized),
		|buf, val| prost::Message::encode(val, buf).unwrap(),
		|buf| prost::Message::decode(buf).unwrap(),
		"prost",
	);
}
