This directory contains examples of glass files and of usage of the C API.

## Compiling C example

```sh
cargo build --features capi
gcc -Iinclude -Ltarget/debug -lglass -Wl,-rpath,target/debug examples/main.c
```
