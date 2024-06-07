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
    let mut zi = munzip::SearchableArchive::new(&mut input).unwrap();

    let filename = "munzip/Cargo.toml";
    let cargo_toml = zi.by_name(filename).unwrap().unwrap();
    write::write_file(&"Cargo.toml".to_owned(), &cargo_toml).unwrap();
}
