use super::*;

/// Defer a closure on drop.
pub struct Defer<F: FnMut()>(pub F);
impl<F: FnMut()> Drop for Defer<F> {
	fn drop(&mut self) {
		(self.0)()
	}
}
macro_rules! defer {
	($($body:tt)*) => {
		let __deferred = Defer(|| { $($body)* });
	};
}
macro_rules! temp_file {
	($file_name:expr) => {
		defer! {
			let _ = dbg!(std::fs::remove_file($file_name));
		}
	};
}

const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz";

#[test]
fn test_corrupt1() {
	let ref key = Key::default();

	temp_file!("corrupt1b");

	// Step 1: Create the empty PAKS file
	FileEditor::create_empty("corrupt1b", key).unwrap();

	// Step 2: Add example
	{
		let mut edit = FileEditor::open("corrupt1b", key).unwrap();
		edit.create_file(b"example", ALPHABET, key).unwrap();
		edit.finish(key).unwrap();
	}

	// Step 5: Read linked
	let example_text = {
		let reader = FileReader::open("corrupt1b", key).unwrap();
		reader.read(b"example", key).unwrap()
	};

	// Corruption!
	assert_eq!(example_text, ALPHABET);
}
