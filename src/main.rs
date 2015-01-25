#![allow(unstable)]
use std::os;
use std::str::FromStr;
mod raiden;

fn print_usage() {
    println!("Usage:");
    println!("    raiden split [file] [disks]");
    println!("    raiden merge [file] [disks]");
}

fn main() {
    let args = os::args();
    match args.as_slice() {
        [_, ref command, ref filename, ref disks_str] if command.as_slice() == "split".as_slice() => {
            match FromStr::from_str(disks_str.as_slice()) {
                None => println!("Invalid number of disks {}", disks_str),
                Some(n) => raiden::split(filename.as_slice(), n),
            }
        },
        [_, ref command, ref filename, ref disks_str] if command.as_slice() == "merge".as_slice() => {
            let disks = match FromStr::from_str(disks_str.as_slice()) {
                None => {
                    println!("Invalid number of disks {}", disks_str);
                    return;
                },
                Some(n) => n,
            };

            raiden::merge(filename.as_slice(), disks);
        }
        _ => {
            print_usage();
        }
    }
}
