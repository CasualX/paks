use std::ptr;
use super::*;

// fn example_dir() -> Vec<Descriptor> {
// 	vec![
// 		Descriptor::file(b"before"),
// 		Descriptor::dir(b"a", 3),
// 		Descriptor::dir(b"b", 2),
// 		Descriptor::dir(b"c", 1),
// 		Descriptor::file(b"file"),
// 	]
// }

#[test]
fn name_eq_example() {
	// Create an empty descriptor with name "test"
	let mut desc = Descriptor::default();
	desc.name.set(b"test");

	assert_eq!(name_eq(&desc, b"test"), Some(&b""[..]));
	assert_eq!(name_eq(&desc, b"test/a/b"), Some(&b"a/b"[..]));
	assert_eq!(name_eq(&desc, b"testing"), None);
	assert_eq!(name_eq(&desc, b"te"), None);
}

#[test]
fn next_sibling_example() {
	// Given the following directory structure:
	//
	// ```text
	// +--. Foo
	// |  |   Bar
	// |  `   Baz
	// |
	// +--. Sub
	// |  `-. Dir
	// |
	// `   File
	// ```
	//
	// Iterating over the top level directory:

	let dir = [
		// ...
		Descriptor::dir(b"Foo", 2),
		Descriptor::file(b"Bar"),
		Descriptor::file(b"Baz"),
		Descriptor::dir(b"Sub", 1),
		Descriptor::dir(b"Dir", 0),
		Descriptor::file(b"File"),
	];
	let results = [true, false, false, true, false, true];

	let mut i = 0;
	let end = dir.len();
	while i < end {
		let desc = &dir[i];
		let next_i = next_sibling(desc, i, end);

		// Process the descriptor
		println!("processing dir[{}] out of {}", i, end);
		assert!(results[i]);

		// Advance the iteration
		i = next_i;
	}

	// Prints the following:
	//
	// ```text
	// processing dir[0] out of 6
	// processing dir[3] out of 6
	// processing dir[5] out of 6
	// ```
}

#[test]
fn test_to_string() {
	let dir = [
		Descriptor::dir(b"Foo", 2),
		Descriptor::file(b"Bar"),
		Descriptor::file(b"Baz"),
		Descriptor::dir(b"Sub", 1),
		Descriptor::dir(b"Dir", 0),
		Descriptor::file(b"File"),
	];

	let expected = "\
./
+- Foo/
|  |  Bar
|  `  Baz
|  
+- Sub/
|  `- Dir/
|  
`  File
";

	let result = DirFmt::new(".", &dir, &TreeArt::ASCII).to_string();
	println!("\n{}", result);
	assert_eq!(expected, result);
}

#[test]
fn test_find_empty() {
	assert_eq!(find(&[], b"path"), &[]);
}

#[test]
fn test_find_desc01() {
	let mut dir = Vec::new();
	create(&mut dir, b"A/B/C");

	let result1 = find_desc(&dir, b"A/B/C");
	let result2 = find_desc(&dir, b"A/B/D");

	assert_eq!(result1.unwrap().name(), b"C");
	assert!(result2.is_none());
}

#[test]
fn test_find() {
	let dir = [
		Descriptor::file(b"before"),
		Descriptor::dir(b"a", 3),
		Descriptor::dir(b"b", 2),
		Descriptor::dir(b"c", 1),
		Descriptor::file(b"file"),
	];

	assert!(ptr::eq(find(&dir, b"before"), &dir[0..1]));
	assert!(ptr::eq(find(&dir, b"a"), &dir[1..]));

	assert!(ptr::eq(find(&dir[2..], b"b"), &dir[2..]));

	assert_eq!(find(&dir, "file".as_ref()).len(), 0);
	assert!(ptr::eq(find(&dir[4..], b"file"), &dir[4..]));

	assert_eq!(find_desc(&dir, b"a\\b\\c\\file").map(|x| x as *const _), Some(&dir[4] as *const _));
}

#[test]
fn test_create_simple() {
	let path = b"stuff.txt";

	let mut dir = Vec::new();
	create(&mut dir, path);

	assert_eq!(dir.len(), 1);
	let file = &dir[0];

	assert_eq!(file.content_type, 0);
	assert_eq!(file.content_size, 0);
	assert_eq!(file.section, Section::default());
	assert_eq!(file.name(), path);
}

#[test]
fn test_create_simple_dirs() {
	let path1 = b"A/FOO";
	let path2 = b"A/BAR";

	let mut dir = Vec::new();
	create(&mut dir, path1);
	create(&mut dir, path2);

	let result = [
		Descriptor::dir(b"A", 2),
		Descriptor::dir(b"FOO", 0),
		Descriptor::dir(b"BAR", 0),
	];
	assert_eq!(dir, result);
}

// #[test]
// fn test_find_encrypted() {
// 	let mut directory = Directory::from(example_dir());
// 	let ref key = [42, 13];
// 	let mut section = Section {
// 		offset: 0,
// 		size: directory.len() as u32,
// 		nonce: Block::default(),
// 		mac: Block::default(),
// 	};
// 	crypt2::encrypt_section(directory.as_blocks_mut(), &mut section, key);
// 	let found = find_encrypted(directory.as_ref(), b"a/b/c/file", &section.nonce, key);
// 	assert!(matches!(found, Some(_)));
// }
