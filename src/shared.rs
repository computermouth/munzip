use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::types::*;

pub const BUFFER_SIZE: usize = 65536;
pub const END_RECORD_SIGNATURE: u32 = 0x06054B50;
pub const GLOBAL_FILE_HEADER_SIGNATURE: u32 = 0x02014B50;
pub const LOCAL_FILE_HEADER_SIGNATURE: u32 = 0x04034B50;

// Read ZIP file end record. Will move within file.
pub fn read_end_record(zip: &mut File) -> Result<EndRecord, MuError> {
    zip.seek(SeekFrom::End(0))?;
    let file_size = zip.stream_position()?;

    if file_size <= std::mem::size_of::<EndRecord>() as u64 {
        return Err(MuError("input file too small".to_string()));
    }

    // Determine the number of bytes to read
    let read_bytes = if file_size < BUFFER_SIZE as u64 {
        file_size
    } else {
        BUFFER_SIZE as u64
    };

    // Seek to the position to start reading from
    zip.seek(SeekFrom::Start(file_size - read_bytes))?;

    // Read the end of the file into a buffer
    let mut buf = vec![0; read_bytes as usize];
    zip.read_exact(&mut buf)?;

    let mut er: Option<&[u8]> = None;
    let record_sz = std::mem::size_of::<EndRecord>();
    for i in (0..=buf.len() - record_sz).rev() {
        let node = &buf[i..i + record_sz];
        // signature is the first u32
        let sig: u32 = (node[3] as u32) << 24
            | (node[2] as u32) << 16
            | (node[1] as u32) << 8
            | (node[0] as u32);
        if sig == END_RECORD_SIGNATURE {
            er = Some(node);
            break;
        }
    }

    if er.is_none() {
        return Err(MuError("end record signature not found in zip".to_string()));
    }

    let end_record: EndRecord = unsafe { std::ptr::read(er.unwrap().as_ptr() as *const _) };

    if end_record.disk_number != 0
        || end_record.central_directory_disk_number != 0
        || end_record.num_entries != end_record.num_entries_this_disk
    {
        return Err(MuError("multifile zips not supported!".to_string()));
    }

    Ok(end_record)
}

pub fn get_global_file_header(buf: &[u8]) -> Result<GlobalFileHeader, MuError> {
    let file_header: GlobalFileHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _) };

    if file_header.signature != GLOBAL_FILE_HEADER_SIGNATURE {
        return Err(MuError("invalid global file header signature".to_string()));
    }

    if file_header.file_name_length as usize + 1 >= BUFFER_SIZE {
        return Err(MuError("file name too long".to_string()));
    }

    Ok(file_header)
}

pub fn get_internal_file_header(buf: &[u8]) -> Result<LocalFileHeader, MuError> {
    let file_header: LocalFileHeader = unsafe { std::ptr::read(buf.as_ptr() as *const _) };

    if file_header.signature != LOCAL_FILE_HEADER_SIGNATURE {
        return Err(MuError("invalid local file header signature".to_string()));
    }

    if file_header.file_name_length as usize + 1 >= BUFFER_SIZE {
        return Err(MuError("file name too long".to_string()));
    }

    if file_header.compression_method == 0
        && file_header.compressed_size != file_header.uncompressed_size
    {
        return Err(MuError("invalid local file header signature".to_string()));
    }

    Ok(file_header)
}

pub fn next_header(
    file: &mut File,
    next_gfh: u64,
) -> Result<(InternalHeader, String, u64), MuError> {
    file.seek(SeekFrom::Start(next_gfh))?;

    const GFH_SIZE: usize = std::mem::size_of::<GlobalFileHeader>();
    let mut fh_buff: [u8; GFH_SIZE] = [0; GFH_SIZE];
    file.read_exact(&mut fh_buff)?;

    let gfh = get_global_file_header(&fh_buff)?;
    let push_pos = file.stream_position()?;

    // seek to local
    file.seek(SeekFrom::Start(gfh.relative_offset_of_local_header as u64))?;

    const LFH_SIZE: usize = std::mem::size_of::<LocalFileHeader>();
    let mut fh_buff: [u8; LFH_SIZE] = [0; LFH_SIZE];
    file.read_exact(&mut fh_buff)?;

    let lfh = get_internal_file_header(&fh_buff)?;

    let mut filename_buf = vec![0; lfh.file_name_length as usize];
    file.read_exact(&mut filename_buf)?;
    let filename = std::str::from_utf8(&filename_buf)?.to_string();

    if lfh.extra_field_length != 0 {
        file.seek(SeekFrom::Current(lfh.extra_field_length as i64))?;
    }

    let ih: InternalHeader = InternalHeader {
        compressed_size: lfh.compressed_size,
        uncompressed_size: lfh.uncompressed_size,
        compression_method: lfh.compression_method,
        offset: file.stream_position()? as u32,
    };

    // rewind to GFH
    file.seek(SeekFrom::Start(push_pos))?;

    // skip filename and comments
    let skip_len: i64 = gfh.file_name_length as i64
        + gfh.extra_field_length as i64
        + gfh.file_comment_length as i64;

    file.seek(SeekFrom::Current(skip_len))?;

    Ok((ih, filename, file.stream_position()?))
}

pub fn data_from_internal(file: &mut File, header: &InternalHeader) -> Result<Vec<u8>, MuError> {
    let dst_len = header.uncompressed_size;
    let src_len = header.compressed_size;

    file.seek(SeekFrom::Start(header.offset as u64))?;

    if header.compression_method == 0 {
        // Store - just read it
        let mut data = vec![0; dst_len as usize];
        file.read_exact(&mut data)?;
        Ok(data)
    } else if header.compression_method == 8 {
        // DEFLATE
        let mut compressed_data = vec![0; src_len as usize];
        file.read_exact(&mut compressed_data)?;
        let data = inflate::inflate_bytes(&compressed_data)?;
        Ok(data)
    } else {
        let method = header.compression_method;
        Err(MuError(
            format!("compression method {method} not supported").to_string(),
        ))
    }
}
