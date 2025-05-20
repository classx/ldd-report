use clap::Parser;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::fs::File;
use std::io::{self};
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

struct Systemlib {
    filename: String,
    abspath: String,
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

fn get_system_libs() -> io::Result<Vec<Systemlib>> {
    let output = std::process::Command::new("ldconfig")
        .arg("-p")
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "ldconfig command failed"
        ));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();

    // Skip the first line which is typically a header
    let mut system_libs = Vec::new();
    for line in lines.iter().skip(1) {
        // Parse lines with format like: "libc.so.6 (libc6,x86-64) => /lib/x86_64-linux-gnu/libc.so.6"
        if let Some(idx) = line.find("=>") {
            let filename_part = line.trim_start();
            let filename_end = filename_part.find(' ').unwrap_or(filename_part.len());
            let filename = filename_part[..filename_end].trim().to_string();

            let abspath = line[idx + 2..].trim().to_string();

            system_libs.push(Systemlib {
                filename,
                abspath,
            });
        }
    }
    Ok(system_libs)
}

fn find_system_lib(system_libs: &[Systemlib], filename: &str) -> Option<String> {
    system_libs.iter()
        .find(|lib| lib.filename == filename)
        .map(|lib| lib.abspath.clone())
}


fn main() {
    let args = Args::parse();
    let system_libs = get_system_libs().unwrap_or_else(|err| {
        eprintln!("Error getting system libraries: {}", err);
        std::process::exit(1);
    });
    let mut dir = Libdir::new(args.dir);
    dir.populate_files();

    for f in dir.files {
        println!("{}", f.fullpath);
        if let Ok(deps) = read_needed_deps(Path::new(&f.fullpath)) {
            for dep in deps {
                if let Some(system_lib) = find_system_lib(&system_libs, &dep) {
                    println!("  {} => {}", dep, system_lib);
                } else {
                    println!("  {} => NOT FOUND", dep);
                }
            }
        } else {
            println!("  ERROR: Could not read dependencies");
        }
    }
}
