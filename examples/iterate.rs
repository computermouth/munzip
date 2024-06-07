
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
    let zi = munzip::ZipIterator::new(&mut input).unwrap();

    for item in zi {
        let item = item.unwrap();

        write::write_file(item.filename(), item.buffer()).unwrap();
    }
}