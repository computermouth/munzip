use std::cell::RefCell;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::rc::Rc;

use crate::shared::*;
use crate::types::*;

pub struct IterableArchive<'a> {
    file: Rc<RefCell<&'a mut File>>,
    end_rec: EndRecord,
    next_gfh: u64,
    next_entry: u16,
    did_error: bool,
}

impl<'a> IterableArchive<'a> {
    pub fn new(file: &'a mut File) -> Result<Self, MuError> {
        let end_rec = read_end_record(file)?;
        let next_entry = 0;
        let did_error = false;

        file.seek(SeekFrom::Start(end_rec.central_directory_offset as u64))?;
        let next_gfh = file.stream_position()?;

        Ok(Self {
            file: Rc::new(RefCell::new(file)),
            end_rec,
            next_gfh,
            next_entry,
            did_error,
        })
    }
}

impl<'a> Iterator for IterableArchive<'a> {
    type Item = Result<Entry<'a>, MuError>;

    fn next(&mut self) -> Option<Self::Item> {
        // shouldn't be possible
        if self.next_entry > self.end_rec.num_entries {
            panic!("wtf");
        }
        // the end
        if self.next_entry == self.end_rec.num_entries || self.did_error {
            return None;
        }

        let nh = next_header(&mut *self.file.borrow_mut(), self.next_gfh);
        if let Err(e) = nh {
            self.did_error = true;
            return Some(Err(e));
        }

        let (header, filename, new_next_gfh) = nh.unwrap();
        self.next_gfh = new_next_gfh;
        self.next_entry += 1;

        Some(Ok(Entry {
            file: Rc::clone(&self.file),
            header,
            filename,
        }))
    }
}

pub struct Entry<'a> {
    file: Rc<RefCell<&'a mut File>>,
    header: InternalHeader,
    filename: String,
}

impl<'a> Entry<'a> {
    pub fn buffer(&mut self) -> Result<Vec<u8>, MuError> {
        data_from_internal(&mut *self.file.borrow_mut(), &self.header)
    }

    pub fn filename(&self) -> String {
        self.filename.clone()
    }

    pub fn compressed_size(&self) -> usize {
        self.header.compressed_size as usize
    }

    pub fn uncompressed_size(&self) -> usize {
        self.header.uncompressed_size as usize
    }
}
