extern crate failure;
extern crate glob;
extern crate goblin;
extern crate regex;
extern crate scroll;
extern crate structopt;

#[macro_use]
extern crate structopt_derive;

#[macro_use]
extern crate lazy_static;

mod args;
use structopt::StructOpt;
use args::Opt;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use regex::Regex;
use failure::{err_msg, Error};
use glob::glob;
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use goblin::{Object,elf};
use goblin::elf::header::{ELFCLASS32, ELFCLASS64, EI_CLASS, ELFMAG, SELFMAG, SIZEOF_IDENT};

struct ElfDeps {
    sysroot: String,
    search_paths: Vec<String>,
    visited: HashSet<String>,
}


lazy_static! {
    static ref LD_SO_CONF_RE: Regex = Regex::new(r"^include\s+(?P<glob>[^\s]+)$").unwrap();
}


fn parse_config(path: &str) -> Result<Vec<String>, Error> {
    let mut paths = Vec::new();

    for line in BufReader::new(&File::open(path)?).lines() {
        let line = line?.trim().to_owned();
        if line.starts_with("#") || line == "" {
            continue;
        }

        if let Some(caps) = LD_SO_CONF_RE.captures(&line) {
            for entry in glob(caps["glob"].trim())? {
                let path = &entry?.to_string_lossy().into_owned();
                paths.extend(parse_config(path)?);
            }
            continue;
        }

        if Path::new(&path).exists() {
            paths.push(line.to_owned());
            continue;
        }

        return Err(err_msg(format!("error extracting lib path from: {}", line)));
    }

    Ok(paths)
}

fn get_elf_architecture(path: &str) -> Result<u8, Error> {
    let mut f = File::open(Path::new(path))?;
    let mut bytes = [0; SIZEOF_IDENT];
    f.read_exact(&mut bytes)?;

    let ident: &[u8] = &bytes[..SIZEOF_IDENT];
    if &ident[0..SELFMAG] != ELFMAG {
        use scroll::Pread;
        let magic: u64 = ident.pread_with(0, scroll::LE)?;
        return Err(err_msg(format!("bad magic: {}", magic)));
    }

    return match ident[EI_CLASS] {
        ELFCLASS32 => Ok(ELFCLASS32),
        ELFCLASS64 => Ok(ELFCLASS64),
        class => Err(err_msg(format!("invalid elf class: {}", class))),
    };
}

impl ElfDeps {
    fn get_deps(&mut self, path: &str) -> Result<Vec<String>, Error> {
        let buffer = {
            let mut bufvec = Vec::new();
            File::open(Path::new(path))?
                .read_to_end(&mut bufvec)
                .unwrap();
            bufvec
        };

        let mut deps = Vec::new();
        return match Object::parse(&buffer)? {
            Object::Elf(elf) => {
                if let &Some(elf::Dynamic { ref dyns, .. }) = &elf.dynamic {
                    for dyn in dyns {
                        if let elf::dyn::DT_RPATH = dyn.d_tag {
                            let rpath = &elf.strtab[dyn.d_val as usize];
                            self.search_paths.push(rpath.to_owned());
                        }
                        if let elf::dyn::DT_NEEDED = dyn.d_tag {
                            let lib = &elf.dynstrtab[dyn.d_val as usize];
                            deps.push(lib.to_owned());
                        }
                    }
                }
                Ok(deps)
            }
            _ => Err(err_msg("unsupported file type")),
        };
    }


    fn populate(&mut self, path: &String) -> Result<(), Error> {
        let arch = get_elf_architecture(&path)?;
        let mut deps: std::collections::VecDeque<String> =
            self.get_deps(path)?.into_iter().collect();

        while !deps.is_empty() {
            let libname = deps.pop_front().unwrap();

            for rpath in self.search_paths.clone().iter() {
                let pathbuf = Path::new(&self.sysroot).join(&rpath).join(&libname);

                if !pathbuf.exists() {
                    continue;
                }

                let path = pathbuf.to_str().unwrap();
                if get_elf_architecture(path)? != arch {
                    continue;
                }

                if !self.visited.insert(libname.to_owned()) {
                    continue;
                }

                // TODO: remove the sysroot clone. not sure how  to do that now :\
                let sysroot = self.sysroot.clone();
                let relpath = pathbuf.strip_prefix(&sysroot)?.to_str().unwrap();
                println!("{} -> {}", libname, relpath);

                if let Ok(ldeps) = self.get_deps(&path) {
                    deps.extend(ldeps);
                }
            }

            if !self.visited.contains(&libname) {
                println!("{} -> ???", libname);
            }
        }
        Ok(())
    }
}


fn run() -> Result<(), Error> {
    let options = Opt::from_args();
    let mut elfdeps = ElfDeps {
        sysroot: options.sysroot,
        search_paths: parse_config(&options.confpath)?,
        visited: HashSet::new(),
    };
    elfdeps.populate(&options.path)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
}
