use clap::Parser;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use goblin::elf::Elf;

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

fn read_needed_deps(path: &Path) -> io::Result<Vec<String>> {
    let mut buf = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut buf)?;
    let elf = Elf::parse(&buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut needed = Vec::new();
    if let Some(dynamic) = elf.dynamic {
        for dyn_entry in dynamic.dyns {
            if dyn_entry.d_tag == goblin::elf::dynamic::DT_NEEDED {
                if let Some(Ok(strtab)) = elf.dynstrtab.get(dyn_entry.d_val as usize) {
                    needed.push(strtab.to_string());
                }
            }
        }
    }
    Ok(needed)
}

fn main() {
    let args = Args::parse();
    let mut dir = Libdir::new(args.dir);
    dir.populate_files();

    for f in dir.files {
        println!("FILE: {}", f.fullpath);
        if let Ok(deps) = read_needed_deps(Path::new(&f.fullpath)) {
            for dep in deps {
                println!("  DEP: {}", dep);
            }
        } else {
            println!("  ERROR: Could not read dependencies");
        }
    }
}
