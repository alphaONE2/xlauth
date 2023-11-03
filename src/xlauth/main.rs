#![windows_subsystem = "windows"]

use std::{
    env::{args, current_exe},
    ffi::OsStr,
    os::windows::process::CommandExt,
    process::{exit, Command},
};

const DETACHED_PROCESS: u32 = 0x00000008;

fn main() {
    let mut path = current_exe().unwrap().canonicalize().unwrap();
    let extension = path
        .extension()
        .unwrap_or_else(|| OsStr::new(""))
        .to_owned();
    path.pop();
    path.push("xlauth-cli");
    path.set_extension(extension);

    let mut child = Command::new(path)
        .args(args().skip(1))
        .creation_flags(DETACHED_PROCESS)
        .spawn()
        .expect("xlauth-cli was started");
    let exit_code = child
        .wait()
        .expect("xlauth-cli has exited")
        .code()
        .unwrap_or(-1);
    exit(exit_code);
}

/*
MIT License

Copyright © alphaONE2

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
