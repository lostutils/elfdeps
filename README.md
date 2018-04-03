```bash
$ elfdeps --help
elfdeps 0.2.0
Oded L. <odedlaz@gmail.com>
Extracts ELF shared library dependencies

USAGE:
    elfdeps [OPTIONS] <FILE>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <confpath>    config file (a la ldconfig) [default: /etc/ld.so.conf]
        --sysroot <sysroot>    system root directory [default: /]

ARGS:
    <FILE>    file to extract its dependencies
```

A simple run:

```bash
$ elfdeps /bin/bash
libtinfo.so.5 -> lib/x86_64-linux-gnu/libtinfo.so.5
libdl.so.2 -> lib/x86_64-linux-gnu/libdl.so.2
libc.so.6 -> lib/x86_64-linux-gnu/libc.so.6
ld-linux-x86-64.so.2 -> lib/x86_64-linux-gnu/ld-linux-x86-64.so.2
```
