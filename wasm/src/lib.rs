use std::{ptr, slice};

extern "C" {
	fn random_bytes(ptr: *mut u8, len: usize);
	fn result_json(ptr: *const u8, len: usize);
	fn result_data(ptr: *const u8, len: usize);
	fn result_error(ptr: *const u8, len: usize);
}

fn custom_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
	unsafe { random_bytes(buf.as_mut_ptr(), buf.len()) };
	Ok(())
}

getrandom::register_custom_getrandom!(custom_getrandom);

#[no_mangle]
pub fn alloc(size: usize) -> *mut u8 {
	let mut buf = Vec::with_capacity(size);
	let ptr = buf.as_mut_ptr();
	std::mem::forget(buf);
	ptr
}

#[no_mangle]
pub fn dealloc(ptr: *mut u8, size: usize) {
	drop(unsafe { Vec::from_raw_parts(ptr, 0, size) })
}

#[no_mangle]
pub fn key_parse(key_ptr: *const u8, key_len: usize) -> *mut paks::Key {
	let key = unsafe { slice::from_raw_parts(key_ptr, key_len) };
	let key = std::str::from_utf8(key).unwrap_or("");
	let key: paks::Key = match u128::from_str_radix(key, 16) {
		Ok(val) => {
			[(val & 0xffffffffffffffff) as u64, (val >> 64) as u64]
		},
		Err(err) => {
			let err = serde_json::json!({ "error": err.to_string() }).to_string();
			unsafe { result_error(err.as_ptr(), err.to_string().len()) };
			return ptr::null_mut();
		},
	};
	let key = Box::new(key);
	Box::into_raw(key)
}

#[no_mangle]
pub fn key_free(key_ptr: *mut paks::Key) {
	if key_ptr.is_null() {
		return;
	}
	drop(unsafe { Box::from_raw(key_ptr) })
}

#[no_mangle]
pub fn paks_open(data_ptr: *const u8, data_len: usize, key: *const paks::Key) -> *mut paks::MemoryEditor {
	let data = unsafe { slice::from_raw_parts(data_ptr, data_len) };
	let key = unsafe { &*key };
	match paks::MemoryEditor::from_bytes(data, key) {
		Ok(paks) => {
			let paks = Box::new(paks);
			Box::into_raw(paks)
		},
		Err(err) => {
			let err = serde_json::json!({ "error": err.to_string() }).to_string();
			unsafe { result_error(err.as_ptr(), err.to_string().len()) };
			std::ptr::null_mut()
		},
	}
}

#[no_mangle]
pub fn paks_close(paks_ptr: *mut paks::MemoryEditor) {
	if paks_ptr.is_null() {
		return;
	}
	drop(unsafe { Box::from_raw(paks_ptr) })
}

#[derive(serde::Serialize)]
#[serde(tag = "ty")]
enum LsEntry {
	File(LsFile),
	Dir(LsDir),
}

#[derive(serde::Serialize)]
struct LsFile {
	name: String,
	size: usize,
}

#[derive(serde::Serialize)]
struct LsDir {
	name: String,
	children: Vec<LsEntry>,
}

#[no_mangle]
pub fn paks_ls(paks_ptr: *mut paks::MemoryEditor) {
	if paks_ptr.is_null() {
		return;
	}
	let paks = unsafe { &mut *paks_ptr };
	fn build_entry(dir: &[paks::Descriptor]) -> Vec<LsEntry> {
		let mut i = 0;
		let mut entries = Vec::new();
		while i < dir.len() {
			let entry = &dir[i];
			i += 1;

			if entry.is_dir() {
				let name = String::from_utf8_lossy(entry.name()).to_string();

				let children = &dir[i..i + entry.content_size as usize];
				let children = build_entry(children);
				entries.push(LsEntry::Dir(LsDir { name, children }));
				i += entry.content_size as usize;
			}
			else if entry.is_file() {
				let name = String::from_utf8_lossy(entry.name()).to_string();
				let size = entry.content_size as usize;
				entries.push(LsEntry::File(LsFile { name, size }));
			}
			else {
				unreachable!();
			}
		}
		// Sort directories first, then files, both alphabetically.
		entries.sort_by(|a, b| {
			match (a, b) {
				(LsEntry::Dir(a), LsEntry::Dir(b)) => a.name.cmp(&b.name),
				(LsEntry::Dir(_), LsEntry::File(_)) => std::cmp::Ordering::Less,
				(LsEntry::File(_), LsEntry::Dir(_)) => std::cmp::Ordering::Greater,
				(LsEntry::File(a), LsEntry::File(b)) => a.name.cmp(&b.name),
			}
		});
		return entries;
	}
	let tree = build_entry(&*paks);
	let tree_json = serde_json::to_string(&tree).unwrap();
	unsafe { result_json(tree_json.as_ptr(), tree_json.len()) };
}

#[no_mangle]
pub fn paks_read(paks_ptr: *mut paks::MemoryEditor, path_ptr: *const u8, path_len: usize, key: *const paks::Key) {
	if paks_ptr.is_null() {
		return;
	}
	let paks = unsafe { &mut *paks_ptr };
	let path = unsafe { slice::from_raw_parts(path_ptr, path_len) };
	let key = unsafe { &*key };
	match paks.read(path, key) {
		Ok(data) => {
			unsafe { result_data(data.as_ptr(), data.len()) };
		},
		Err(err) => {
			let err = serde_json::json!({ "error": err.to_string() }).to_string();
			unsafe { result_error(err.as_ptr(), err.to_string().len()) };
		},
	}
}
