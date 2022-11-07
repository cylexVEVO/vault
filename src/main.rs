use std::{
	fs::{self, DirEntry},
	path::{Path, PathBuf}
};

use indoc::printdoc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct FileSystem {
	files: Vec<File>
}

impl FileSystem {
	fn new() -> Self {
		FileSystem { files: Vec::new() }
	}

	fn file_exists(&self, file_name: &String, file_ext: &String) -> bool {
		self.files
			.iter()
			.find(|&f| &f.name == file_name && &f.extension == file_ext)
			.is_some()
	}

	fn add_file(&mut self, file: File, overwrite: bool) -> Result<(), &str> {
		if self.file_exists(&file.name, &file.extension) && !overwrite {
			Err("file already exists")
		} else {
			if overwrite {
				self.delete_file(&file.name, &file.extension).unwrap();
			}
			self.files.push(file);
			Ok(())
		}
	}

	fn get_file(&self, file_name: &String, file_ext: &String) -> Result<&File, &str> {
		if let Some(file) = self
			.files
			.iter()
			.find(|&f| &f.name == file_name && &f.extension == file_ext)
		{
			Ok(file)
		} else {
			Err("file does not exist")
		}
	}

	fn delete_file(&mut self, file_name: &String, file_ext: &String) -> Result<(), &str> {
		if self.file_exists(file_name, file_ext) {
			self.files
				.retain(|file| &file.name != file_name && &file.extension != file_ext);
			Ok(())
		} else {
			Err("file does not exist")
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct File {
	name: String,
	extension: String,
	content: Vec<u8>
}

impl File {
	fn new(name: String, extension: String, content: Vec<u8>) -> Self {
		File {
			name,
			extension,
			content
		}
	}

	fn display(&self) -> String {
		format!(
			"{}.{} [{} bytes]",
			self.name,
			self.extension,
			self.content.len()
		)
	}
}

fn get_fs() -> Result<FileSystem, &'static str> {
	if let Ok(bytes) = fs::read("vault.vault") {
		if let Ok(fs) = rmp_serde::from_slice(&bytes) {
			Ok(fs)
		} else {
			Err("vault in current directory is invalid")
		}
	} else {
		Err("no vault in current directory")
	}
}

fn get_fs_or_exit() -> FileSystem {
	if let Ok(fs) = get_fs() {
		fs
	} else {
		println!("no vault in current directory");
		std::process::exit(1);
	}
}

fn save_fs(fs: &FileSystem) {
	let mut path = std::env::current_dir().unwrap();
	path.push("vault.vault");
	let path = path;
	if fs::write(path, rmp_serde::to_vec(fs).unwrap()).is_err() {
		println!("error writing vault");
		std::process::exit(1);
	}
}

fn pluralize(num: usize) -> &'static str {
	if num == 0 {
		"s"
	} else if num == 1 {
		""
	} else {
		"s"
	}
}

fn file_name_and_ext_from_string(string: String) -> (String, String) {
	let file = PathBuf::from(string);

	let file_name = if let Some(name) = file.file_stem() {
		name.to_string_lossy().to_string()
	} else {
		println!("invalid file");
		std::process::exit(1);
	};

	let file_ext = if let Some(ext) = file.extension() {
		ext.to_string_lossy().to_string()
	} else {
		println!("invalid file");
		std::process::exit(1);
	};

	(file_name, file_ext)
}

fn get_files_from_dir(dir: PathBuf) -> std::io::Result<Vec<DirEntry>> {
	let mut files: Vec<DirEntry> = Vec::new();

	for entry in fs::read_dir(dir)? {
		let entry = entry?;

		if entry.path().is_file() {
			files.push(entry);
		} else {
			let mut entries = if let Ok(e) = get_files_from_dir(entry.path()) {
				e
			} else {
				return Ok(files);
			};

			files.append(&mut entries);
		}
	}

	Ok(files)
}

fn main() {
	let command = if let Some(c) = std::env::args().nth(1) {
		c
	} else {
		println!("no command provided");
		std::process::exit(1);
	};

	match command.as_str() {
		"add" => {
			let force = std::env::args()
				.find(|arg| arg == &String::from("-f") || arg == &String::from("--force"))
				.is_some();
			let mut fs = get_fs_or_exit();
			let path = if let Some(p) = std::env::args().nth(2) {
				p
			} else {
				println!("no file path");
				std::process::exit(1);
			};

			let path_buf = PathBuf::from(path);

			fn add_file(path: String, fs: &mut FileSystem, force: bool) {
				let path_and_ext = file_name_and_ext_from_string(path.clone());
				let file_name = path_and_ext.0;
				let file_ext = path_and_ext.1;

				let file_content = if let Ok(content) = fs::read(path) {
					content
				} else {
					println!("error reading file");
					std::process::exit(1);
				};

				if fs
					.add_file(
						File::new(file_name.clone(), file_ext.clone(), file_content),
						force
					)
					.is_ok()
				{
					if force {
						println!("got -f or --force, overwriting existing file");
					}
					println!("added {}.{} to the vault", file_name, file_ext);
				} else {
					println!(
						"file already exists\nif you want to overwrite the file, use -f or --force"
					);
					std::process::exit(1);
				}
			}

			if path_buf.is_file() {
				add_file(path_buf.to_string_lossy().into(), &mut fs, force);
			} else if path_buf.is_dir() {
				let files = if let Ok(f) = get_files_from_dir(path_buf) {
					f
				} else {
					println!("unknown error");
					std::process::exit(1);
				};

				for file in files {
					add_file(file.path().to_string_lossy().into(), &mut fs, force);
				}
			} else {
				println!("path is not a file or directory");
				std::process::exit(1);
			}

			save_fs(&fs);
		},
		"export" => {
			let fs = get_fs_or_exit();
			let path = if let Some(p) = std::env::args().nth(2) {
				p
			} else {
				println!("no file path");
				std::process::exit(1);
			};

			let path_and_ext = file_name_and_ext_from_string(path);
			let file_name = path_and_ext.0;
			let file_ext = path_and_ext.1;

			let read_file = if let Ok(f) = fs.get_file(&file_name, &file_ext) {
				f
			} else {
				println!("file does not exist inside vault");
				std::process::exit(1);
			};

			if fs::write(
				format!("vault-{}.{}", read_file.name, read_file.extension),
				read_file.content.clone()
			)
			.is_ok()
			{
				println!(
					"exported {}.{} to ./vault-{}.{}",
					read_file.name, read_file.extension, read_file.name, read_file.extension
				);
			} else {
				println!("error writing");
				std::process::exit(1);
			}
		},
		"cat" => {
			let fs = get_fs_or_exit();
			let path = if let Some(p) = std::env::args().nth(2) {
				p
			} else {
				println!("no file path");
				std::process::exit(1);
			};

			let path_and_ext = file_name_and_ext_from_string(path);
			let file_name = path_and_ext.0;
			let file_ext = path_and_ext.1;

			let read_file = if let Ok(f) = fs.get_file(&file_name, &file_ext) {
				f
			} else {
				println!("file does not exist inside vault");
				std::process::exit(1);
			};

			println!("{}.{}:", read_file.name, read_file.extension);
			println!("{}", String::from_utf8_lossy(read_file.content.as_slice()));
		},
		"rm" => {
			let mut fs = get_fs_or_exit();
			let path = if let Some(p) = std::env::args().nth(2) {
				p
			} else {
				println!("no file path");
				std::process::exit(1);
			};

			let path_and_ext = file_name_and_ext_from_string(path);
			let file_name = path_and_ext.0;
			let file_ext = path_and_ext.1;

			if fs.delete_file(&file_name, &file_ext).is_err() {
				println!("file does not exist inside vault");
				std::process::exit(1);
			}

			save_fs(&fs);

			println!("deleted file");
		},
		"ls" => {
			let fs = get_fs_or_exit();

			let files: Vec<String> = fs.files.iter().map(|file| file.display()).collect();

			println!(
				"{} file{}:\n{}",
				files.len(),
				pluralize(files.len()),
				files.join("\n")
			);
		},
		"init" => {
			let empty = std::env::args()
				.find(|arg| arg == &String::from("-e") || arg == &String::from("--empty"))
				.is_some();
			let force = std::env::args()
				.find(|arg| arg == &String::from("-f") || arg == &String::from("--force"))
				.is_some();

			if Path::new("vault.vault").exists() && !force {
				println!("vault already exists in current directory\nif you want to overwrite the current vault, use -f or --force");
				std::process::exit(1);
			} else {
				if force {
					println!("got -f or --force, overwriting current vault");
				}
				if empty {
					println!("got -e or --empty, creating empty vault");
				}
				let mut fs = FileSystem::new();
				if !empty {
					fs.add_file(
						File::new("hello".into(),
						"txt".into(),
						"welcome to vault! vault is your private place to store sensitive documents, files, photos, and much more. get started by running `vault help` to see the available commands!".as_bytes().to_vec()),
						false
					).expect("this won't happen");
				}
				save_fs(&fs);
				println!("created a new vault in current directory");
			}
		},
		"help" => {
			printdoc! {
				"
				vault v{version}

				usage
				vault [command] <arguments>

				commands
				   add <file path> - adds a file to the vault
				       -f, --force - overwrite existing file (if any)
				export <file name> - exports a file from the vault
				   cat <file name> - prints contents of a file in the vault
				    rm <file name> - deletes a file from the vault
				    ls             - lists all files in the vault
				  init             - creates a vault in the current directory
				      -e, --empty  - create an empty vault
				      -f, --force  - overwrite existing vault (if any)
				  help             - prints available commands
				",
				version = env!("CARGO_PKG_VERSION")
			}
		},
		_ => {
			println!("invalid command");
			std::process::exit(1);
		}
	}
}
