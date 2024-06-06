

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Mutex, OnceLock};

mod time;
mod types;
use types::*;

const JZ_BUFFER_SIZE: usize = 65536;
const JZ_END_RECORD_SIGNATURE: u32 = 0x06054B50;

fn buffer() -> &'static Mutex<[u8;JZ_BUFFER_SIZE]> {
    static STORES: OnceLock<Mutex<[u8;JZ_BUFFER_SIZE]>> = OnceLock::new();
    STORES.get_or_init(|| std::sync::Mutex::new([0;JZ_BUFFER_SIZE]))
}

// Read ZIP file end record. Will move within file.
fn jz_read_end_record(zip: &mut File) -> Result<Option<JZEndRecord>, MZError> {
    let file_size: u64;
    let read_bytes: isize;
    let i: isize;
    let mut jz_buffer = buffer().lock().unwrap();


    zip.seek(SeekFrom::End(0))?;
    file_size = zip.seek(SeekFrom::Current(0))?;

    if file_size <= std::mem::size_of::<JZEndRecord>() as u64 {
        return Err(MZError("input file too small".to_string()));
    }

    // Determine the number of bytes to read
    let read_bytes = if file_size < JZ_BUFFER_SIZE as u64 {
        file_size
    } else {
        JZ_BUFFER_SIZE as u64
    };

    // Seek to the position to start reading from
    zip.seek(SeekFrom::Start(file_size - read_bytes))?;
    
    // Read the end of the file into a buffer
    let mut buffer_slice = &mut jz_buffer[..read_bytes as usize];
    zip.read_exact(&mut buffer_slice)?;

    let mut er: Option<&[u8]> = None;
    let record_sz = std::mem::size_of::<JZEndRecord>();
    for i in (0..=buffer_slice.len() - record_sz).rev() {
        let node = &buffer_slice[i..i+record_sz];
        // signature is the first u32
        let sig: u32 =
            (node[3] as u32) << 24 |
            (node[2] as u32) << 16 |
            (node[1] as u32) << 8  |
            (node[0] as u32);
        if sig == JZ_END_RECORD_SIGNATURE {
            er = Some(node);
            break;
        }
    }

    if er.is_none() {
        return Err(MZError("end record signature not found in zip".to_string()));
    }

    // if end_record.disk_number != 0
    //     || end_record.central_directory_disk_number != 0
    //     || end_record.num_entries != end_record.num_entries_this_disk {
    //     return Err(MZError("multifile zips not supported!".to_string()));
    // }

    Ok(None)
}

fn main() {

    let mut args = std::env::args();
    if args.len() != 2 {
        eprintln!("{} <FILE>", args.nth(0).unwrap());
        return;
    }

    let _ = jz_read_end_record(&mut File::open(args.nth(1).unwrap()).unwrap());

    println!("Hello, world!");
}
