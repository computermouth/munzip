/// The munzip Error type. Currently not an enum, just a String wrapper.
#[derive(Debug)]
pub struct MuError(pub String);

impl std::fmt::Display for MuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<std::io::Error> for MuError {
    fn from(err: std::io::Error) -> MuError {
        MuError(err.to_string())
    }
}

impl From<std::str::Utf8Error> for MuError {
    fn from(err: std::str::Utf8Error) -> MuError {
        MuError(err.to_string())
    }
}

impl From<String> for MuError {
    fn from(err: String) -> MuError {
        MuError(err)
    }
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct LocalFileHeader {
    pub signature: u32,                 // 0x04034B50
    pub version_needed_to_extract: u16, // unsupported
    pub general_purpose_bit_flag: u16,  // unsupported
    pub compression_method: u16,
    pub last_mod_file_time: u16,
    pub last_mod_file_date: u16,
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub file_name_length: u16,
    pub extra_field_length: u16, // unsupported
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct GlobalFileHeader {
    pub signature: u32,                 // 0x02014B50
    pub version_made_by: u16,           // unsupported
    pub version_needed_to_extract: u16, // unsupported
    pub general_purpose_bit_flag: u16,  // unsupported
    pub compression_method: u16,
    pub last_mod_file_time: u16,
    pub last_mod_file_date: u16,
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub file_name_length: u16,
    pub extra_field_length: u16,       // unsupported
    pub file_comment_length: u16,      // unsupported
    pub disk_number_start: u16,        // unsupported
    pub internal_file_attributes: u16, // unsupported
    pub external_file_attributes: u32, // unsupported
    pub relative_offset_of_local_header: u32,
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct InternalHeader {
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub compression_method: u16,
    pub offset: u32,
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct EndRecord {
    pub signature: u32,
    pub disk_number: u16,
    pub central_directory_disk_number: u16,
    pub num_entries_this_disk: u16,
    pub num_entries: u16,
    pub central_directory_size: u32,
    pub central_directory_offset: u32,
    pub zip_comment_length: u16,
}
