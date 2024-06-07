use std::fs::File;

use munzip;
mod write;

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        eprintln!("{} <FILE>", args.nth(0).unwrap());
        return;
    }

    let mut input = File::open(args.nth(1).unwrap()).unwrap();
    let mut zi = munzip::Archive::new(&mut input).unwrap();

    zi.by_name("farts").unwrap();
}
