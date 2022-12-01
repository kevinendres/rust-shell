# arsh
Another Rust Shell

Developed as a final project for Corin Pitcher's CSC463 at DePaul University, Fall 2022.
Aiming for compliance with [build-your-own-shell project](https://github.com/tokenrove/build-your-own-shell) requirements. Currently passing all of Stage 1 and 2 except for Stage 2 test 5.

## Current Features
* Path searching for command execution
* Argument lists
* cd, pwd, exec, exit builtins
* Command lists separated by ';' or '&&' or '||'
* Multiline commands that end with '\' or '&&' or '||'
* Prompt color matches previous exit status
* Pipes using '|'
* Redirection with '<', '>', '>>', '<>'
* Redirection with 0, 1, 2 file descriptors
* Command substitution using '$(...)'
* Subshells

## To-do List
* Signals
* Useage of arbitrary file descriptors
* File descriptor duplication and closing

## Usage
`cargo run`

Try shell commands
