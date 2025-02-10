# ozora
Tools for automatic generation of hypervisor modules and decoder automation.

## Requirements
libsail is required.
```
opam show libsail

<><> libsail: information on all versions <><><><><><><><><><><><><><><><><><><>
name                   libsail
all-installed-versions 0.17.1 [4.14.0]
```

## Usage
Edit files.json to specify target sail files and run a command.
```
cargo r -- --help
Ozora: Tools for automatic generation of hypervisor modules and decoder automation

Usage: ozora [OPTIONS] --ext-name <EXT_NAME> <TARGET> <OUTPUT>

Arguments:
  <TARGET>  Target file
  <OUTPUT>  Output path

Options:
      --loglv <LOGLV>            Logging level as string (e.g. "debug" or "trace")
      --ext-name <EXT_NAME>      Extension name
      --input-json <INPUT_JSON>  Json file path contains target sail files [default: files.json]
  -h, --help                     Print help
  -V, --version                  Print version
```

## Example
```
cargo r riscv_zicfiss.sail target/zicfiss.rs --ext-name Zicfiss
```
