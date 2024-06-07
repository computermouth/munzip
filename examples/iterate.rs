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

    let zi = munzip::IterableArchive::new(&mut input).unwrap();

    for entry in zi {
        let mut entry = entry.unwrap();

        let filename = entry.filename();
        let buffer = entry.buffer().unwrap();

        write::write_file(&filename, &buffer).unwrap();
    }
}
