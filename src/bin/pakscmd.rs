/*!
Implements pakscmd's command-line interface.
*/

#![allow(non_snake_case)]

use std::{env, fs, io, io::prelude::*, path, str};

fn main() {
	let args: Vec<_> = env::args().collect();
	let args: Vec<_> = args.iter().map(|s| &**s).collect();

	match &args[1..] {
		&[] => print!("{}", HELP_GENERAL),
		&["help"] => print!("{}", HELP_GENERAL),
		&[_] => eprintln!("Error invalid syntax, see `pakscmd help`."),
		&["help", cmd] => help(&[cmd]),
		&[_, _] => eprintln!("Error invalid syntax, see `pakscmd help`."),
		&[_pak, _key, "help", ref args @ ..] => help(args),
		&[paks, key, "new", ref args @ ..] => new(paks, key, args),
		&[paks, key, "tree", ref args @ ..] => tree(paks, key, args),
		&[paks, key, "add", ref args @ ..] => add(paks, key, args),
		&[paks, key, "copy", ref args @ ..] => copy(paks, key, args),
		&[paks, key, "link", ref args @ ..] => link(paks, key, args),
		&[paks, key, "cat", ref args @ ..] => cat(paks, key, args),
		&[paks, key, "rm", ref args @ ..] => rm(paks, key, args),
		&[paks, key, "mv", ref args @ ..] => mv(paks, key, args),
		&[paks, key, "fsck", ref args @ ..] => fsck(paks, key, args),
		&[paks, key, "gc", ref args @ ..] => gc(paks, key, args),
		&[paks, key, "dbg", ref args @ ..] => dbg(paks, key, args),
		&[_pak, _key, cmd, ..] => eprintln!("Error unknown subcommand: {}", cmd),
	}
}

fn parse_key(s: &str) -> Option<paks::Key> {
	match u128::from_str_radix(s, 16) {
		Ok(val) => {
			Some([(val & 0xffffffffffffffff) as u64, (val >> 64) as u64])
		},
		Err(err) => {
			eprintln!("Error parsing key argument: {}", err);
			None
		},
	}
}

//----------------------------------------------------------------

const HELP_GENERAL: &str = "\
pakscmd - Copyright (c) 2020-2025 Casper <CasualX@users.noreply.github.com>

USAGE
    pakscmd help <COMMAND>
    pakscmd <PAKFILE> <KEY> <COMMAND> [..]

ARGUMENTS
    PAKFILE  Path to a PAKS archive to create or edit.
    KEY      The 128-bit encryption key encoded in hex.
    COMMAND  The subcommand to invoke.

Commands are:
    new      Creates a new empty PAKS archive.
    tree     Displays the directory of the PAKS archive.
    add      Adds a file to the PAKS archive.
    copy     Copies files to the PAKS archive.
    link     Links the file from alternative paths.
    cat      Reads files from the PAKS archive and writes to stdout.
    rm       Removes paths from the PAKS archive.
    mv       Moves files in the PAKS archive.
    fsck     File system consistency check.
    gc       Collects garbage left behind by removed files.

    See `pakscmd help <COMMAND>` for more information on a specific command.

EXAMPLES
    pakscmd example.paks 0 new
    pakscmd example.paks 0 add a/b/example < tests/data/example.txt
    pakscmd example.paks 0 link a/b/example aa/bb/example
    pakscmd example.paks 0 tree -u
    pakscmd example.paks 0 rm a/b/example
    pakscmd example.paks 0 cat aa/bb/example
";

fn help(args: &[&str]) {
	let text = match args.first().cloned() {
		None => HELP_GENERAL,
		Some("new") => HELP_NEW,
		Some("tree") => HELP_TREE,
		Some("add") => HELP_ADD,
		Some("copy") => HELP_COPY,
		Some("link") => HELP_LINK,
		Some("cat") => HELP_CAT,
		Some("rm") => HELP_RM,
		Some("mv") => HELP_MV,
		Some("fsck") => HELP_FSCK,
		Some("gc") => HELP_GC,
		Some(cmd) => return eprintln!("Error unknown subcommand: {}", cmd),
	};
	print!("{}", text);
}

//----------------------------------------------------------------

const HELP_NEW: &str = "\
NAME
    pakscmd-new - Creates a new empty PAKS archive.

DESCRIPTION
    Creates a new empty PAKS archive with the given file name and encryption key.
    If a file with this name already exists it will be overwritten.
";

fn new(file: &str, key: &str, _args: &[&str]) {
	let ref key = match parse_key(key) {
		Some(key) => key,
		None => return,
	};

	if let Err(err) = paks::FileEditor::create_empty(file, key) {
		eprintln!("Error writing {}: {}", file, err);
	}
}

//----------------------------------------------------------------

const HELP_TREE: &str = "\
NAME
    pakscmd-tree - Displays the directory of the PAKS archive.

SYNOPSIS
    pakscmd [..] tree [-au] [PATH]

DESCRIPTION
    Displays the directory of the PAKS archive.

ARGUMENTS
    -a       Display using ASCII art.
    -u       Display using UNICODE art.
    PATH     Optional subdirectory to start at.
";

fn tree(file: &str, key: &str, mut args: &[&str]) {
	let ref key = match parse_key(key) {
		Some(key) => key,
		None => return,
	};

	let mut art = &paks::TreeArt::UNICODE;
	while let Some(head) = args.first().cloned() {
		if head.starts_with("-") {
			args = &args[1..];
			match head {
				"-a" => art = &paks::TreeArt::ASCII,
				"-u" => art = &paks::TreeArt::UNICODE,
				_ => eprintln!("Unknown argument: {}", head),
			}
		}
		else {
			break;
		}
	}

	let path = match args {
		&[path] => Some(path),
		[..] => None,
	};

	let reader = match paks::FileReader::open(file, key) {
		Ok(reader) => reader,
		Err(err) => return eprintln!("Error opening {}: {}", file, err),
	};

	let display = match reader.display_children(path, art) {
		Some(display) => display,
		None => return eprintln!("Error directory not found or is a file: {}", path.unwrap_or("")),
	};

	println!("{}", display);
}

//----------------------------------------------------------------

const HELP_ADD: &str = "\
NAME
    pakscmd-add - Adds a file to the PAKS archive.

SYNOPSIS
    pakscmd [..] add <PATH> < <CONTENT>

DESCRIPTION
    Adds a file to the PAKS archive.

ARGUMENTS
    PATH     The destination path in the PAKS archive to put the file.
    CONTENT  The file data to write in the PAKS archive passed via stdin.
";

fn add(file: &str, key: &str, args: &[&str]) {
	let ref key = match parse_key(key) {
		Some(key) => key,
		None => return,
	};

	let path = match args {
		[path] => path,
		_ => return eprintln!("Error invalid path: expected exactly 1 argument."),
	};

	let mut data = Vec::new();
	match io::stdin().read_to_end(&mut data) {
		Ok(_) => (),
		Err(err) => return eprintln!("Error reading stdin: {}", err),
	};

	let mut edit = match paks::FileEditor::open(file, key) {
		Ok(edit) => edit,
		Err(err) => return eprintln!("Error opening {}: {}", file, err),
	};

	if let Err(err) = edit.create_file(path.as_bytes(), &data, key) {
		eprintln!("Error creating {}: {}", path, err);
	}

	if let Err(err) = edit.finish(key) {
		eprintln!("Error writing {}: {}", file, err);
	}
}

//----------------------------------------------------------------

const HELP_COPY: &str = "\
NAME
    pakscmd-copy - Copies files to the PAKS archive.

SYNOPSIS
    pakscmd [..] copy <PATH> [FILE]..

DESCRIPTION
    Copies files to the PAKS archive.
";

fn copy(file: &str, key: &str, args: &[&str]) {
	let ref key = match parse_key(key) {
		Some(key) => key,
		None => return,
	};

	if args.len() < 1 {
		return eprintln!("Error invalid syntax: expecting one path followed by many filenames.");
	}
	else if args.len() == 1 {
		return;
	}
	let base_path = args[0];

	let mut edit = match paks::FileEditor::open(file, key) {
		Ok(edit) => edit,
		Err(err) => return eprintln!("Error opening {}: {}", file, err),
	};

	let mut dest_path = String::from(base_path);

	for src_path in &args[1..] {
		let src_path = path::Path::new(src_path);

		let dest_len = dest_path.len();
		copy_rec(&mut edit, src_path, &mut dest_path, true, key);
		dest_path.truncate(dest_len);
	}

	if let Err(err) = edit.finish(key) {
		eprintln!("Error writing {}: {}", file, err);
	}
}

fn copy_rec(edit: &mut paks::FileEditor, src_path: &path::Path, dest_path: &mut String, root: bool, key: &paks::Key) {
	if dest_path.len() > 0 && !dest_path.ends_with("/") {
		dest_path.push_str("/");
	}

	if src_path.is_file() {
		// Read the file contents
		let data = match fs::read(src_path) {
			Ok(data) => data,
			Err(err) => {
				eprintln!("Error reading {}: {}", src_path.display(), err);
				return;
			},
		};

		// Extract the file name
		let file_name = match src_path.file_name().and_then(|s| s.to_str()) {
			Some(file_name) => file_name,
			None => {
				eprintln!("Error invalid file name: {}", src_path.display());
				return;
			},
		};

		// Construct destination path
		dest_path.push_str(file_name);

		// Write its contents to the PAKS archive
		if let Err(err) = edit.create_file(dest_path.as_bytes(), &data, key) {
			eprintln!("Error creating {}: {}", dest_path, err);
		}
	}
	else if src_path.is_dir() {
		if !root {
			// Extract the directory name
			let dir_name = match src_path.file_name().and_then(|s| s.to_str()) {
				Some(dir_name) => dir_name,
				None => {
					eprintln!("Error invalid directory name: {}", src_path.display());
					return;
				},
			};

			// Create the directory in the PAKS archive
			dest_path.push_str(dir_name);
			edit.create_dir(dest_path.as_bytes());
		}

		// Recurse into the directory
		let read_dir = match fs::read_dir(src_path) {
			Ok(read_dir) => read_dir,
			Err(err) => {
				eprintln!("Error reading {}: {}", src_path.display(), err);
				return;
			},
		};

		for entry in read_dir {
			let entry = match entry {
				Ok(entry) => entry,
				Err(err) => {
					eprintln!("Error reading {}: {}", src_path.display(), err);
					continue;
				},
			};

			let dest_len = dest_path.len();
			copy_rec(edit, &entry.path(), dest_path, false, key);
			dest_path.truncate(dest_len);
		}
	}
	else {
		eprintln!("Warning skipping {}: not a file or directory", src_path.display());
	}
}

//----------------------------------------------------------------

const HELP_LINK: &str = "\
NAME
    pakscmd-link - Links the file from alternative paths.

SYNOPSIS
    pakscmd [..] link <SRC> [DEST]..

DESCRIPTION
    Links the source file to alternative destination paths.
    Returns file not found error if the SRC path does not exist.

ARGUMENTS
    SRC      Path to the source file to link.
    DEST     One or more destination paths where to link the SRC.
";

fn link(file: &str, key: &str, args: &[&str]) {
	let ref key = match parse_key(key) {
		Some(key) => key,
		None => return,
	};

	let (src_path, dest_paths) = match args {
		&[src, ref dest @ ..] => (src, dest),
		_ => return eprintln!("Error invalid syntax: expecting a source file"),
	};

	let mut edit = match paks::FileEditor::open(file, key) {
		Ok(edit) => edit,
		Err(err) => return eprintln!("Error opening {}: {}", file, err),
	};

	let src_desc = match edit.find_desc(src_path.as_bytes()) {
		Some(desc) if desc.is_dir() => return eprintln!("Error file not found: {}", src_path),
		Some(desc) => *desc,
		None => return eprintln!("Error file not found: {}", src_path),
	};

	for &dest_path in dest_paths {
		edit.create_link(dest_path.as_bytes(), &src_desc);
	}

	if let Err(err) = edit.finish(key) {
		eprintln!("Error writing {}: {}", file, err);
	}
}

//----------------------------------------------------------------

const HELP_CAT: &str = "\
NAME
    pakscmd-cat - Reads files from the PAKS archive and writes to stdout.

SYNOPSIS
    pakscmd [..] cat [PATH]..

DESCRIPTION
    Reads files from the PAKS archive and writes to stdout.
    Each file is read in the order specified and written to stdout one after another.
    If an error happens it is printed and continues to write the rest of the files.

ARGUMENTS
    PATH     Path to the file in the PAKS archive to output.
";

fn cat(file: &str, key: &str, args: &[&str]) {
	let ref key = match parse_key(key) {
		Some(key) => key,
		None => return,
	};

	let reader = match paks::FileReader::open(file, key) {
		Ok(reader) => reader,
		Err(err) => return eprintln!("Error opening {}: {}", file, err),
	};

	for &path in args {
		match reader.find_file(path.as_bytes()) {
			Some(file_desc) => {
				match reader.read_data(&file_desc, key) {
					Ok(data) => {
						if let Err(err) = io::stdout().write_all(&data) {
							eprintln!("Error writing {} to stdout: {}", path, err);
						}
					},
					Err(err) => eprintln!("Error reading {}: {}", path, err),
				}
			},
			None => eprintln!("Error file not found: {}", path),
		}
	}
}

//----------------------------------------------------------------

const HELP_RM: &str = "\
NAME
    pakscmd-rm - Removes files from the PAKS archive.

SYNOPSIS
    pakscmd [..] rm [PATH]..

DESCRIPTION
    Removes files from the PAKS archive.

ARGUMENTS
    PATH     Path to the file in the PAKS archive to remove.
";

fn rm(file: &str, key: &str, args: &[&str]) {
	let ref key = match parse_key(key) {
		Some(key) => key,
		None => return,
	};

	let mut edit = match paks::FileEditor::open(file, key) {
		Ok(edit) => edit,
		Err(err) => return eprintln!("Error opening {}: {}", file, err),
	};

	for &path in args {
		if edit.remove(path.as_bytes()).is_none() {
			eprintln!("Unable to remove {}: file not found?", path);
		}
	}

	if let Err(err) = edit.finish(key) {
		eprintln!("Error writing {}: {}", file, err);
	}
}

//----------------------------------------------------------------

const HELP_MV: &str = "\
NAME
    pakscmd-mv - Moves files in the PAKS archive.

SYNOPSIS
    pakscmd [..] mv <SRC> <DEST>

DESCRIPTION
    Moves files in the PAKS archive.

ARGUMENTS
    SRC      Path to the source file.
    DEST     Path to the destination file.
";

fn mv(file: &str, key: &str, args: &[&str]) {
	let ref key = match parse_key(key) {
		Some(key) => key,
		None => return,
	};

	let (src_path, dest_path) = match args {
		&[src_path, dest_path] => (src_path, dest_path),
		[..] => return eprintln!("Error invalid syntax: expecting exactly two path arguments."),
	};

	let mut edit = match paks::FileEditor::open(file, key) {
		Ok(edit) => edit,
		Err(err) => return eprintln!("Error opening {}: {}", file, err),
	};

	edit.move_file(src_path.as_bytes(), dest_path.as_bytes());

	if let Err(err) = edit.finish(key) {
		eprintln!("Error writing {}: {}", file, err);
	}
}

//----------------------------------------------------------------

const HELP_FSCK: &str = "\
NAME
    pakscmd-fsck - File system consistency check.

SYNOPSIS
    pakscmd [..] fsck

DESCRIPTION
    Checks the PAKS file's directory for errors.
";

fn fsck(file: &str, key: &str, _args: &[&str]) {
	let ref key = match parse_key(key) {
		Some(key) => key,
		None => return,
	};

	let reader = match paks::FileReader::open(file, key) {
		Ok(reader) => reader,
		Err(err) => return eprintln!("Error opening {}: {}", file, err),
	};

	let mut log = String::new();
	let msg = if !reader.fsck(reader.high_mark(), &mut log) {
		"PAKS file contains errors:\n"
	}
	else {
		"No errors found!\n"
	};

	print!("{}{}", msg, log);
}

//----------------------------------------------------------------

const HELP_GC: &str = "\
NAME
    pakscmd-gc - Collects garbage left behind by removed files.

SYNOPSIS
    pakscmd [..] gc

DESCRIPTION
    Collects garbage left behind by removed files.
    When files are removed their data is left behind.
    These files are unreadable because their cryptographic nonce is forgotten.
";

fn gc(file: &str, key: &str, _args: &[&str]) {
	let ref key = match parse_key(key) {
		Some(key) => key,
		None => return,
	};

	let f = match fs::File::open(file) {
		Ok(f) => f,
		Err(err) => return eprintln!("Error opening {}: {}", file, err),
	};

	let blocks = match paks::read(f, key) {
		Ok(blocks) => blocks,
		Err(err) => return eprintln!("Error reading {}: {}", file, err),
	};

	let mut edit = match paks::MemoryEditor::from_blocks(blocks, key) {
		Ok(edit) => edit,
		Err(_) => return eprintln!("Error invalid {}: not a PAKS file", file),
	};

	edit.gc();

	let (data, _) = edit.finish(key);
	if let Err(err) = fs::write(file, dataview::bytes(data.as_slice())) {
		eprintln!("Error writing {}: {}", file, err);
	}
}

//----------------------------------------------------------------

fn dbg(file: &str, key: &str, _args: &[&str]) {
	let ref key = match parse_key(key) {
		Some(key) => key,
		None => return,
	};

	let reader = match paks::FileReader::open(file, key) {
		Ok(reader) => reader,
		Err(err) => return eprintln!("Error opening {}: {}", file, err),
	};

	print!("{:#?}", reader.as_ref());
}
