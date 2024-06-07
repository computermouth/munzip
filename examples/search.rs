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

    let not_found = zi.by_name("nope");
    assert!(not_found.is_ok());
    assert!(not_found.unwrap().is_none());

    let munzip = "munzip/Cargo.toml";
    let is_found = zi.by_name(munzip).unwrap().unwrap();
    write::write_file(&"Cargo.toml".to_owned(), &is_found).unwrap();
}
