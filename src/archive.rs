use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tar::Archive as TarArchive;
use zip::ZipArchive;

pub fn is_archive(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    matches!(ext, "zip" | "tar" | "gz" | "tgz") || name.ends_with(".tar.gz")
}

pub fn extract_archive(path: &Path, dest: &Path) -> io::Result<Vec<PathBuf>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if ext == "zip" {
        extract_zip(path, dest)
    } else if ext == "tgz" || name.ends_with(".tar.gz") {
        extract_tar_gz(path, dest)
    } else if ext == "tar" {
        extract_tar(path, dest)
    } else if ext == "gz" {
        extract_tar_gz(path, dest)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "Unknown archive format"))
    }
}

fn extract_zip(path: &Path, dest: &Path) -> io::Result<Vec<PathBuf>> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut extracted = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest.join(path),
            None => continue,
        };

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
            extracted.push(outpath);
        }
    }

    Ok(extracted)
}

fn extract_tar(path: &Path, dest: &Path) -> io::Result<Vec<PathBuf>> {
    let file = File::open(path)?;
    extract_tar_from_reader(file, dest)
}

fn extract_tar_gz(path: &Path, dest: &Path) -> io::Result<Vec<PathBuf>> {
    let file = File::open(path)?;
    let decoder = GzDecoder::new(file);
    extract_tar_from_reader(decoder, dest)
}

fn extract_tar_from_reader<R: Read>(reader: R, dest: &Path) -> io::Result<Vec<PathBuf>> {
    let mut archive = TarArchive::new(reader);
    let mut extracted = Vec::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let outpath = dest.join(&path);

        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else if entry.header().entry_type().is_file() {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            entry.unpack(&outpath)?;
            extracted.push(outpath);
        }
    }

    Ok(extracted)
}
