use clap::Parser;
use std::fs;
use std::io::Read;
use std::path::Path;

/// Library relation report utility
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Libraries path
    #[arg(short, long)]
    dir: String,

    /// Return exit code only
    #[arg(short, long, default_value_t = false)]
    silent: bool,
}

enum Filetype {
    ELF,
    CODE,
}

struct Libdir {
    dir: String,
    files: Vec<Files>,
}

struct Files {
    fullpath: String,
    filetype: Filetype,
}

impl Libdir {
    fn new(dirname: String) -> Self {
        Self {
            dir: dirname,
            files: Vec::new(),
        }
    }

    fn populate_files(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(metadata) = fs::metadata(&path) {
                            //let a = metadata.file_type();
                            println!("type: {:#?}", path);
                            if metadata.is_file() && is_elf(&path) {
                                self.files.push(Files {
                                    fullpath: path.to_string_lossy().to_string(),
                                    filetype: Filetype::ELF,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

fn is_elf(path: &Path) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        let mut magic = [0u8; 4];
        if let Ok(_) = file.read_exact(&mut magic) {
            return magic == [0x7f, b'E', b'L', b'F'];
        }
    }
    false
}

fn main() {
    let args = Args::parse();
    let mut dir = Libdir::new(args.dir);
    dir.populate_files();

    for f in dir.files {
        print!("FILE: {}", f.fullpath);
    }

    println!("DONE!");
}
