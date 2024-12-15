use super::*;

/// File reader.
pub struct FileReader {
	file: fs::File,
	directory: Directory,
	info: InfoHeader,
}

impl FileReader {
	/// Opens a PAKS file for reading.
	///
	/// If the file at the given path is not a PAKS file or the encryption key is incorrect, [`io::ErrorKind::InvalidData`] is returned.
	#[inline]
	pub fn open<P: ?Sized + AsRef<Path>>(path: &P, key: &Key) -> io::Result<FileReader> {
		open(path.as_ref(), key)
	}
}

#[inline(never)]
fn open(path: &Path, key: &Key) -> io::Result<FileReader> {
	let mut file = fs::File::open(path)?;

	let (info, directory) = read_header(&mut file, key)?;

	Ok(FileReader { file, directory, info })
}

impl ops::Deref for FileReader {
	type Target = Directory;
	#[inline]
	fn deref(&self) -> &Directory {
		&self.directory
	}
}

impl FileReader {
	/// Returns the info header.
	#[inline]
	pub fn info(&self) -> &InfoHeader {
		&self.info
	}

	/// Highest block index containing file data.
	#[inline]
	pub fn high_mark(&self) -> u32 {
		self.info.directory.offset
	}

	/// Reads the contents of a file from the PAKS archive.
	pub fn read(&self, path: &[u8], key: &Key) -> io::Result<Vec<u8>> {
		let desc = match self.find_file(path) {
			Some(desc) => desc,
			None => Err(io::ErrorKind::NotFound)?,
		};

		self.read_data(desc, key)
	}

	/// Reads the contents of a file from the PAKS archive into a string.
	pub fn read_to_string(&self, path: &[u8], key: &Key) -> io::Result<String> {
		let desc = match self.find_file(path) {
			Some(desc) => desc,
			None => Err(io::ErrorKind::NotFound)?,
		};

		let data = self.read_data(desc, key)?;
		String::from_utf8(data).map_err(|_| io::ErrorKind::InvalidData.into())
	}

	/// Decrypts the section.
	///
	/// The key is not required to be the same as used to open the PAKS file.
	///
	/// # Errors
	///
	/// * [`io::ErrorKind::InvalidInput`]: The the descriptor is not a file descriptor.
	/// * [`io::ErrorKind::InvalidData`]: The file's MAC is incorrect, the file is corrupted.
	/// * [`io::Error`]: An error encountered reading the underlying PAKS file.
	#[inline]
	pub fn read_section(&self, section: &Section, key: &Key) -> io::Result<Vec<Block>> {
		read_section(&self.file, section, key)
	}

	/// Decrypts the contents of the given file descriptor.
	///
	/// See [`read_section`](Self::read_section) for more information.
	#[inline]
	pub fn read_data(&self, desc: &Descriptor, key: &Key) -> io::Result<Vec<u8>> {
		read_data(&self.file, desc, key)
	}

	/// Decrypts the contents of the given file descriptor into the dest buffer.
	///
	/// See [`read_section`](Self::read_section) for more information.
	#[inline]
	pub fn read_data_into(&self, desc: &Descriptor, key: &Key, byte_offset: usize, dest: &mut [u8]) -> io::Result<()> {
		read_data_into(&self.file, desc, key, byte_offset, dest)
	}
}
