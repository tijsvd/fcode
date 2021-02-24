use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Instant;

fn test_ser_de<T: Serialize + DeserializeOwned>(value: &T, slug: &str, mut checker: impl FnMut(&T)) {
	// warm-up
	let mut buffer = fcode::to_bytes(value).unwrap();

	const N: u64 = 1000000;

	let start = Instant::now();
	for _ in 0..N {
		buffer.clear();
		// fcode::to_vec_extend(&mut buffer, &value).unwrap();
		fcode::to_writer(&mut buffer, &value).unwrap();
		let received: T = fcode::from_bytes(&buffer).unwrap();
		checker(&received);
	}
	let elapsed = start.elapsed();
	println!(
		"{} : {} ser/de in {:?} = {} ns/roundtrip",
		slug,
		N,
		elapsed,
		elapsed.as_nanos() as u64 / N,
	);
}

fn main() {
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
}
