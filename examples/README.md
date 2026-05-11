# Examples

This directory contains examples of glass files and of usage of the C API.

## Compiling C example on linux

```sh
cargo build --features capi
gcc -oglass_example -Iinclude -Ltarget/debug -lglass -Wl,-rpath,target/debug examples/main.c
```
