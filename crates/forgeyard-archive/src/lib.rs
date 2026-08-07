use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

pub fn create_tar_gz(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), ArchiveError> {
    let tar_gz = File::create(dst)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(".", src)?;
    tar.finish()?;
    Ok(())
}

pub fn create_zip(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), ArchiveError> {
    let file = File::create(dst)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Simplistic zip logic for directories
    let src_path = src.as_ref();
    if src_path.is_dir() {
        for entry in walkdir::WalkDir::new(src_path) {
            let entry = entry.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let path = entry.path();
            let name = path.strip_prefix(src_path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            if path.is_file() {
                zip.start_file(name.to_string_lossy(), options)?;
                let mut f = File::open(path)?;
                std::io::copy(&mut f, &mut zip)?;
            } else if !name.as_os_str().is_empty() {
                zip.add_directory(name.to_string_lossy(), options)?;
            }
        }
    } else {
        let file_name = src_path.file_name().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Invalid file name"))?;
        zip.start_file(file_name.to_string_lossy(), options)?;
        let mut f = File::open(src_path)?;
        std::io::copy(&mut f, &mut zip)?;
    }

    zip.finish()?;
    Ok(())
}
