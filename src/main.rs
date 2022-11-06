use serde::{Deserialize, Serialize};
use std::{fs, path::Path, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct FileSystem {
    files: Vec<File>,
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

    fn add_file(&mut self, file: File) -> Result<(), &str> {
        if self.file_exists(&file.name, &file.extension) {
            return Err("file already exists");
        } else {
            self.files.push(file);
            return Ok(());
        }
    }

    fn get_file(&self, file_name: &String, file_ext: &String) -> Result<&File, &str> {
		if let Some(file) = self.files.iter().find(|&f| &f.name == file_name && &f.extension == file_ext) {
			Ok(file)
		} else {
            Err("file does not exist")
		}
    }

    fn delete_file(&mut self, file_name: &String, file_ext: &String) -> Result<(), &str> {
        if self.file_exists(file_name, file_ext) {
            self.files
                .retain(|file| &file.name != file_name && &file.extension != file_ext);
            return Ok(());
        } else {
            return Err("file does not exist");
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct File {
    name: String,
    extension: String,
    content: Vec<u8>,
}

impl File {
    fn new(name: String, extension: String, content: Vec<u8>) -> Self {
        File {
            name,
            extension,
            content,
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
        if let Ok(fs) =  rmp_serde::from_slice(&bytes) {
            return Ok(fs);
        } else {
            return Err("vault in current directory is invalid");
        }
    } else {
        return Err("no vault in current directory");
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

fn main() {
    let command = if let Some(c) = std::env::args().nth(1) {
		c
	} else {
		println!("no command provided");
		std::process::exit(1);
	};

    match command.as_str() {
        "add" => {
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

            let file_content = if let Ok(content) = fs::read(path) {
				content
			} else {
				println!("error reading file");
                std::process::exit(1);
			};

			if fs.add_file(File::new(file_name, file_ext, file_content)).is_ok() {
				println!("added {}.{} to the vault", file_name, file_ext);
			} else {
				println!("file already exists");
                std::process::exit(1);
			}

            save_fs(&fs);
        }
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

            if fs::write(format!("vault-{}.{}", read_file.name, read_file.extension), read_file.content.clone()).is_ok() {
                println!(
                    "exported to ./vault-{}.{}",
                    read_file.name, read_file.extension
                );
                std::process::exit(0);
            } else {
                println!("error writing");
                std::process::exit(1);
            }
        }
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
        }
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
            std::process::exit(0);
        }
        "ls" => {
            let fs = get_fs_or_exit();

            let files: Vec<String> = fs.files.iter().map(|file| file.display()).collect();

            println!(
                "{} file{}:\n{}",
                files.len(),
                pluralize(files.len()),
                files.join("\n")
            );
        }
        "init" => {
            if Path::new("vault.vault").exists() {
                println!("vault already exists in current directory");
                std::process::exit(0);
            } else {
                let mut fs = FileSystem::new();
                fs.add_file(
					File::new("hello".into(),
					"txt".into(),
					"welcome to vault! vault is your private place to store sensitive documents, files, photos, and much more. get started by running `vault help` to see the available commands!".as_bytes().to_vec())
				).expect("this won't happen");
                save_fs(&fs);
                println!("created a new vault in current directory");
                std::process::exit(0);
            }
        }
        "help" => {
            println!(
                "vault v0.1.0

usage
vault [command] <arguments>

commands
add <file path> - adds file to the vault
export <file name> - exports a file from the vault
cat <file name> - prints contents of a file in the vault
rm <file name> - deletes a file from the vault
ls - lists all files in the vault
init - creates a vault in the current directory
help - prints available commands"
            );
        }
        _ => {
            println!("invalid command");
            std::process::exit(1);
        }
    }
}