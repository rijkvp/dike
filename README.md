# dike

dike is a parallel code tester which runs your tests blazingly fast using multithreading.
dike is named after the Greek Goddess of justice Dike or Dice, daughter of Themis.

## Example

```sh
dike -i "./tests/*.in" "./myprogram"
```
This will run `./myprogram` with all testcases from `.in` and check the results against the corresponding `.out` files

## Installation

Using Nix Flakes:
```sh
nix profile install github:rijkvp/dike
```

Or build manually using Cargo.

## Usage

```
dike --help
Parallel code tester

Usage: dike [OPTIONS] --inputs <INPUTS> <CMD>

Arguments:
  <CMD>  Command that runs the program to test

Options:
  -i, --inputs <INPUTS>    Glob pattern for test files
  -l, --timeout <TIMEOUT>  Time limit for each run
  -t, --threads <THREADS>  Sets a custom amount of threads, defaults to number of CPUs
  -h, --help               Print help
  -V, --version            Print version
```
