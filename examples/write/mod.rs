use std::io::Write;
use std::path::Path;

use munzip::MZError;

pub fn write_file(filename: &String, data: &Vec<u8>) -> Result<(), MZError> {
    let path = Path::new(&filename);

    if filename.ends_with("/") {
        if !path.exists() {
            std::fs::create_dir_all(path)
                .map_err(|_| MZError(format!("failed to create dir '{:?}'", path).to_string()))?;
        }
        return Ok(());
    }

    // is a dir, or empty filename
    if path.ends_with("/") || path == Path::new("") {
        return Ok(());
    }

    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(&data).unwrap();

    Ok(())
}
