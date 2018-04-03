#[derive(StructOpt)]
#[structopt(name = "elfdeps", about = "Extracts ELF shared library dependencies",
            author = "Oded L. <odedlaz@github.com>")]
pub struct Opt {
    #[structopt(short = "c", long = "config", help = "config file (a la ldconfig)",
                default_value = "/etc/ld.so.conf")]
    pub confpath: String,
    #[structopt(long = "sysroot", help = "system root directory", default_value = "/")]
    pub sysroot: String,
    #[structopt(name = "FILE", help = "file to extract its dependencies")] pub path: String,
}
