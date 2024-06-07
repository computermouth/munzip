use std::collections::HashMap;
use std::fs::File;
use std::io::{Seek, SeekFrom};

use crate::shared::*;
use crate::types::*;

/// A queryable interface for the archive. This is for workloads where you may want
/// to hold the file handle open, and load in specific files, by their name, on demand.
///
/// If you intend to load all your files at once, it'd probably be better to just use the
/// `IterableArchive`.
///
/// # Examples
///
/// ``` no_run
/// let mut zi = munzip::SearchableArchive::new(&mut input).unwrap();
///
/// let filename = "munzip/Cargo.toml";
/// let cargo_toml = zi.by_name(filename).unwrap().unwrap();
/// write::write_file(&"Cargo.toml".to_owned(), &cargo_toml).unwrap();
/// ```
pub struct SearchableArchive<'a> {
    file: &'a mut File,
    map: HashMap<String, InternalHeader>,
    end_rec: EndRecord,
    next_gfh: u64,
}

impl<'a> SearchableArchive<'a> {
    /// Creates a new `SearchableArchive`, and scans the entire archive for file headers.
    pub fn new(file: &'a mut File) -> Result<Self, MuError> {
        let end_rec = read_end_record(file)?;

        file.seek(SeekFrom::Start(end_rec.central_directory_offset as u64))?;
        let next_gfh = file.stream_position()?;

        let mut sa = Self {
            file,
            map: HashMap::new(),
            end_rec,
            next_gfh,
        };

        sa.build_map()?;

        Ok(sa)
    }

    fn build_map(&mut self) -> Result<(), MuError> {
        for _ in 0..self.end_rec.num_entries {
            let (header, filename, new_next_gfh) = next_header(self.file, self.next_gfh)?;
            self.next_gfh = new_next_gfh;
            eprintln!("{filename}");
            self.map.insert(filename, header);
        }

        Ok(())
    }

    /// Performs a lookup based on the filenames of all entries.
    pub fn by_name(&mut self, name: &str) -> Result<Option<Vec<u8>>, MuError> {
        let ih_opt = self.map.get(&(name.to_owned())).cloned();

        match ih_opt {
            None => Ok(None),
            Some(ih) => Ok(Some(data_from_internal(&mut self.file, &ih)?)),
        }
    }
}
