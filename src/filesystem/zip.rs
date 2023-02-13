// Adapted from zip crate's write dir example:
// https://github.com/zip-rs/zip/blob/master/examples/write_dir.rs

use std::io::prelude::*;
use std::io::{Seek, Write};
use std::iter::Iterator;
use zip::result::ZipError;
use zip::write::FileOptions;
use std::fs::File;
use std::path::Path;
use walkdir::{WalkDir, DirEntry};
use super::TEMP_ZIP_LOCATION;

const IGNORE_FILE: &'static str = ".filesyncignore";

fn zip_dir_entries<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    println!("Compressing folder...");

    let all_files: Vec<DirEntry> = it.collect();
    let complete_count = all_files.len();

    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for (n, entry) in all_files.iter().enumerate() {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        let fraction_complete = n as f64 / complete_count as f64;
        print!("\r{}%\t{}", (fraction_complete*100.0).round()/100.0, path.display());
        std::io::stdout().flush().unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    // print a new line
    println!("");
    Result::Ok(())
}

fn ignored_file(ignored: &Vec<String>, file: &DirEntry) -> bool {
    // will match as ignored if the file's path contains a substring of any ignored
    for ignored_file in ignored {
        if file.path().to_str().unwrap().contains(ignored_file) {
            return true;
        }
    }
    false
}

pub fn compress_dir(
    src_dir: &str,
    dst_file: &str,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    let ignore_file = File::open(IGNORE_FILE);

    // dont add the zip file to the zip (obviously)
    let mut ignore_filenames: Vec<String> = vec![TEMP_ZIP_LOCATION.to_string()];

    match ignore_file {
        Ok(ignore_file) => {
            let mut ignore_file = ignore_file;
            let mut string_buf = String::new();
            let _ = ignore_file.read_to_string(&mut string_buf);
            ignore_filenames.extend(string_buf.split("\r\n").map(|s| s.to_string()));
        }
        Err(_) => {
            println!("Ignore file {} not found", IGNORE_FILE);
        }
    }

    let mut it_filtered = it
        .filter_map(|e| e.ok())
        
        .filter(
            |x| !ignored_file(&ignore_filenames, x)
        );

    zip_dir_entries(
        &mut it_filtered,
        src_dir,
        file,
        method
    )?;

    Ok(())
}
