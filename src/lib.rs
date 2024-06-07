use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Mutex, OnceLock};

use inflate;

mod types;
use types::*;
pub use types::MZError;

const JZ_BUFFER_SIZE: usize = 65536;
const JZ_END_RECORD_SIGNATURE: u32 = 0x06054B50;
const JZ_GLOBAL_FILE_HEADER_SIGNATURE: u32 = 0x02014B50;
const JZ_LOCAL_FILE_HEADER_SIGNATURE: u32 = 0x04034B50;

fn buffer() -> &'static Mutex<[u8; JZ_BUFFER_SIZE]> {
    static STORES: OnceLock<Mutex<[u8; JZ_BUFFER_SIZE]>> = OnceLock::new();
    STORES.get_or_init(|| std::sync::Mutex::new([0; JZ_BUFFER_SIZE]))
}

// Read ZIP file end record. Will move within file.
fn jz_read_end_record(zip: &mut File) -> Result<JZEndRecord, MZError> {
    let file_size: u64;
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
        let node = &buffer_slice[i..i + record_sz];
        // signature is the first u32
        let sig: u32 = (node[3] as u32) << 24
            | (node[2] as u32) << 16
            | (node[1] as u32) << 8
            | (node[0] as u32);
        if sig == JZ_END_RECORD_SIGNATURE {
            er = Some(node);
            break;
        }
    }

    if er.is_none() {
        return Err(MZError("end record signature not found in zip".to_string()));
    }

    let end_record: JZEndRecord = unsafe { std::ptr::read(er.unwrap().as_ptr() as *const _) };

    if end_record.disk_number != 0
        || end_record.central_directory_disk_number != 0
        || end_record.num_entries != end_record.num_entries_this_disk
    {
        return Err(MZError("multifile zips not supported!".to_string()));
    }

    Ok(end_record)
}

fn jz_read_data(zip: &mut File, header: &JZFileHeader) -> Result<Vec<u8>, MZError> {
    let dst_len = header.uncompressed_size;
    let src_len = header.compressed_size;

    if header.compression_method == 0 {
        // Store - just read it
        let mut data = vec![0; dst_len as usize];
        zip.read_exact(&mut data)?;
        Ok(data)
    } else if header.compression_method == 8 {
        // DEFLATE
        let mut compressed_data = vec![0; src_len as usize];
        zip.read_exact(&mut compressed_data)?;
        let data = inflate::inflate_bytes(&compressed_data)?;
        Ok(data)
    } else {
        let method = header.compression_method;
        Err(MZError(
            format!("compression method {method} not supported").to_string(),
        ))
    }
}

fn jz_read_local_file_header(zip: &mut File) -> Result<(JZFileHeader, String), MZError> {
    let (local_header, filename) = jz_read_local_file_header_raw(zip)?;

    let header = JZFileHeader {
        compression_method: local_header.compression_method,
        last_mod_file_time: local_header.last_mod_file_time,
        last_mod_file_date: local_header.last_mod_file_date,
        crc32: local_header.crc32,
        compressed_size: local_header.compressed_size,
        uncompressed_size: local_header.uncompressed_size,
        offset: 0, // not used in local context
    };

    Ok((header, filename))
}

fn jz_read_local_file_header_raw(zip: &mut File) -> Result<(JZLocalFileHeader, String), MZError> {
    let fh_size = std::mem::size_of::<JZLocalFileHeader>();
    let mut buf = vec![0; fh_size];
    zip.read_exact(&mut buf)?;

    let header: JZLocalFileHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _) };

    if header.signature != JZ_LOCAL_FILE_HEADER_SIGNATURE {
        return Err(MZError("invalid local file header signature".to_string()));
    }

    let mut filename_buf = vec![0; header.file_name_length as usize];
    zip.read_exact(&mut filename_buf)?;
    let filename = std::str::from_utf8(&filename_buf)?.to_string();

    if header.extra_field_length != 0 {
        zip.seek(SeekFrom::Current(header.extra_field_length as i64))?;
    }

    if header.compression_method == 0 && header.compressed_size != header.uncompressed_size {
        return Err(MZError("invalid local file header signature".to_string()));
    }

    Ok((header, filename))
}

pub struct ZipIterator<'a> {
    file: &'a mut File,
    filename: Option<String>,
    end_rec: JZEndRecord,
    next_entry: u16,
}

impl<'a> ZipIterator<'a> {
    pub fn new(file: &'a mut File) -> Result<Self, MZError> {
        let end_rec = jz_read_end_record(file)?;
        let next_entry = 0;

        file.seek(SeekFrom::Start(end_rec.central_directory_offset as u64))?;

        Ok(Self {
            file,
            filename: None,
            end_rec,
            next_entry,
        })
    }

    fn record_callback(&mut self, header: &JZFileHeader) -> Result<Vec<u8>, MZError> {
        let offset = self.file.seek(SeekFrom::Current(0))?;
        self.file.seek(SeekFrom::Start(header.offset as u64))?;

        // process_file
        let ret = self.process_file()?;

        self.file.seek(SeekFrom::Start(offset))?;

        Ok(ret)
    }

    fn process_file(&mut self) -> Result<Vec<u8>, MZError> {
        let (header, filename) = jz_read_local_file_header(self.file)?;

        let cs = header.compressed_size;
        let us = header.uncompressed_size;
        let of = header.offset;
        eprintln!("{}, {} / {} bytes at offset {:x}", filename, cs, us, of);

        let jzr = jz_read_data(self.file, &header)?;
        self.filename = Some(filename);

        Ok(jzr)
    }
}

impl<'a> Iterator for ZipIterator<'a> {
    type Item = Result<ZipEntry, MZError>;

    fn next(&mut self) -> Option<Self::Item> {
        // self.file.seek(SeekFrom::Start(self.end_rec.central_directory_offset as u64)).unwrap();
        if self.next_entry > self.end_rec.num_entries {
            panic!("wtf");
        }
        // the end
        if self.next_entry == self.end_rec.num_entries {
            return None;
        }

        const FH_SIZE: usize = std::mem::size_of::<JZGlobalFileHeader>();
        let mut jz_buffer = buffer().lock().unwrap();

        let mut fh_buff: [u8; FH_SIZE] = [0; FH_SIZE];

        if let Err(e) = self.file.read_exact(&mut fh_buff) {
            return Some(Err(e.into()));
        }
        let file_header: JZGlobalFileHeader =
            unsafe { std::ptr::read(fh_buff.as_ptr() as *const _) };

        if file_header.signature != JZ_GLOBAL_FILE_HEADER_SIGNATURE {
            return Some(Err(MZError(
                "invalid global file header signature".to_string(),
            )));
        }

        if file_header.file_name_length as usize + 1 >= JZ_BUFFER_SIZE {
            return Some(Err(MZError("file name too long".to_string())));
        }

        let mut buf = vec![0; file_header.file_name_length as usize];
        if let Err(e) = self.file.read(&mut buf) {
            return Some(Err(e.into()));
        }

        jz_buffer[..buf.len()].clone_from_slice(&buf);
        // null terminator, probably not necessary
        jz_buffer[buf.len()] = 0;

        // skip comments
        if let Err(e) = self
            .file
            .seek(SeekFrom::Current(file_header.extra_field_length as i64))
        {
            return Some(Err(e.into()));
        }
        if let Err(e) = self
            .file
            .seek(SeekFrom::Current(file_header.file_comment_length as i64))
        {
            return Some(Err(e.into()));
        }

        let header = JZFileHeader {
            compression_method: file_header.compression_method,
            last_mod_file_time: file_header.last_mod_file_time,
            last_mod_file_date: file_header.last_mod_file_date,
            crc32: file_header.crc32,
            compressed_size: file_header.compressed_size,
            uncompressed_size: file_header.uncompressed_size,
            offset: file_header.relative_offset_of_local_header,
        };

        match self.record_callback(&header) {
            Ok(buffer) => {
                // yo wtf is this
                let filename = <Option<String> as Clone>::clone(&self.filename).unwrap();
                self.next_entry += 1;
                Some(Ok(ZipEntry {
                    header,
                    buffer,
                    filename,
                }))
            }
            Err(e) => Some(Err(e.into())),
        }
    }
}

pub struct ZipEntry {
    header: JZFileHeader,
    buffer: Vec<u8>,
    filename: String,
}

impl ZipEntry {
    pub fn header(&self) -> () {
        _ = self.header;
        ()
    }
    pub fn buffer(&self) -> &Vec<u8> {
        &self.buffer
    }
    pub fn filename(&self) -> &String {
        &self.filename
    }
}