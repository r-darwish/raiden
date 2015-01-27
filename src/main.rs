#![allow(unstable)]
use std::os;
use std::str::FromStr;
mod raiden;

fn print_usage() {
    println!("Usage:");
    println!("    raiden split [file] [disks]");
    println!("    raiden merge [file] [disks]");
}

macro_rules! try_arg_parse(
    ($arg:ident, $err:expr) => (match FromStr::from_str($arg.as_slice()) {
        None => {
            println!($err);
            return;
        },
        Some(n) => n
    })
);

macro_rules! call_action(
    ($action: expr) => (match $action {
        Err(why) => {
            println!("{}", why);
            return;
        }
        _ => ()
    })
);

fn main() {
    let args = os::args();
    match args.as_slice() {
        [_, ref command, ref filename, ref disks_str] if command.as_slice() == "split".as_slice() => {
            let disks: usize = try_arg_parse!(disks_str, "Invalid number of disks");
            call_action!(raiden::split(filename.as_slice(), disks));
        },
        [_, ref command, ref filename, ref disks_str] if command.as_slice() == "merge".as_slice() => {
            let disks: usize = try_arg_parse!(disks_str, "Invalid number of disks");
            call_action!(raiden::merge(filename.as_slice(), disks));
        }
        _ => {
            print_usage();
        }
    }
}
