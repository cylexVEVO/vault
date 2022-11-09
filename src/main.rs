use std::{
	fs::{self, DirEntry},
	path::{Path, PathBuf}
};

use clap::{Parser, Subcommand};
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

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
	#[command(subcommand)]
	command: Commands
}

#[derive(Subcommand, Debug)]
enum Commands {
	Add {
		files: Vec<PathBuf>,
		// #[arg(short, long)]
		// force: bool
	},
	Ls,
	Init {
		#[arg(short, long)]
		force: bool,
		#[arg(short, long)]
		empty: bool
	},
	Export {
		files: Vec<String>
	},
	Cat {
		file: String
	},
	Rm {
		files: Vec<String>
	}
}

fn main() {
	let cli = Cli::parse();

	match &cli.command {
		Commands::Add { files/*, force*/ } => {
			let paths = files.clone();
			let mut files: Vec<PathBuf> = Vec::new();

			for path in paths.into_iter() {
				if path.is_file() {
					files.push(path);
				} else if path.is_dir() {
					let entries = get_files_from_dir(path).unwrap_or_else(|_| {
						println!("unknown error");
						std::process::exit(1)
					});

					let mut entries: Vec<PathBuf> = entries.iter().map(|entry| entry.path()).collect();
					// TODO: support files without file extensions, for now just ignore them
					entries.retain(|entry| entry.extension().is_some());

					files.append(&mut entries);
				}
			}

			let mut fs = get_fs_or_exit();

			for file in files.iter() {
				let path_and_ext = file_name_and_ext_from_string(file.to_string_lossy().into());
				let file_name = path_and_ext.0;
				let file_ext = path_and_ext.1;

				let file_content = if let Ok(content) = fs::read(file) {
					content
				} else {
					println!("error reading file {}.{}, skipping...", file_name, file_ext);
					continue;
				};

				if fs
					.add_file(
						File::new(file_name.clone(), file_ext.clone(), file_content),
						// TODO WHICH IS VERY VERY IMPORTANT: FIX FORCE ADDING FILES
						false
					)
					.is_ok()
				{
					// if *force {
					// 	println!("got -f or --force, overwriting existing file");
					// }
					println!("added {}.{} to the vault", file_name, file_ext);
				} else {
					println!(
						"file {}.{} already exists\nif you want to overwrite the file, use -f or --force",
						file_name,
						file_ext
					);
				}
			}

			save_fs(&fs);
		},
		Commands::Ls => {
			let fs = get_fs_or_exit();

			let files: Vec<String> = fs.files.iter().map(|file| file.display()).collect();

			println!(
				"{} file{}:\n{}",
				files.len(),
				pluralize(files.len()),
				files.join("\n")
			);
		},
		Commands::Init { force, empty } => {
			if Path::new("vault.vault").exists() && !*force {
				println!("vault already exists in current directory\nif you want to overwrite the current vault, use -f or --force");
				std::process::exit(1);
			} else {
				if *force {
					println!("got -f or --force, overwriting current vault");
				}
				if *empty {
					println!("got -e or --empty, creating empty vault");
				}
				let mut fs = FileSystem::new();
				if !*empty {
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
		Commands::Export { files } => {
			let fs = get_fs_or_exit();

			for file in files.iter() {
				let path_and_ext = file_name_and_ext_from_string(file.clone());
				let file_name = path_and_ext.0;
				let file_ext = path_and_ext.1;
	
				let read_file = if let Ok(f) = fs.get_file(&file_name, &file_ext) {
					f
				} else {
					println!("file {}.{} does not exist inside vault", &file_name, &file_ext);
					continue;
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
					println!("error writing exported file, skipping...");
				}
			}
		},
		Commands::Cat { file } => {
			let fs = get_fs_or_exit();

			let path_and_ext = file_name_and_ext_from_string(file.clone());
			let file_name = path_and_ext.0;
			let file_ext = path_and_ext.1;

			let read_file = if let Ok(f) = fs.get_file(&file_name, &file_ext) {
				f
			} else {
				println!("file {}.{} does not exist inside vault", file_name, file_ext);
				std::process::exit(1);
			};

			println!("{}.{}:", read_file.name, read_file.extension);
			println!("{}", String::from_utf8_lossy(read_file.content.as_slice()));
		},
		Commands::Rm { files } => {
			let mut fs = get_fs_or_exit();

			for file in files.iter() {
				let path_and_ext = file_name_and_ext_from_string(file.clone());
				let file_name = path_and_ext.0;
				let file_ext = path_and_ext.1;
	
				if fs.delete_file(&file_name, &file_ext).is_err() {
					println!("file {}.{} does not exist inside vault", file_name, file_ext);
				} else {
					println!("deleted file {}.{} from vault", file_name, file_ext);
				}
			}

			save_fs(&fs);
		}
	}
}
