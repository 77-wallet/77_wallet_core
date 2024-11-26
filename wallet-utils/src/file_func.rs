use std::{
    fs::{self, OpenOptions},
    path::Path,
};

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), crate::Error> {
    if !path.as_ref().exists() {
        fs::create_dir_all(path).map_err(crate::Error::IO)?;
    }
    Ok(())
}

pub fn recreate_dir_all<P: AsRef<Path>>(path: P) -> Result<(), crate::Error> {
    let path = path.as_ref();
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(fs::create_dir_all(path)?)
}

pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), crate::Error> {
    fs::remove_dir_all(path).map_err(crate::Error::IO)
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<(), crate::Error> {
    fs::remove_file(path).map_err(Into::into)
}

pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<(), crate::Error> {
    Ok(fs::rename(from, to)?)
}

pub fn read<P: AsRef<Path>>(buf: &mut String, path: P) -> Result<(), crate::Error> {
    fs::read_to_string(path)
        .map(|s| {
            buf.push_str(&s);
        })
        .map_err(Into::into)
}

pub fn read_dir<P: AsRef<Path>>(path: P) -> Result<fs::ReadDir, crate::Error> {
    fs::read_dir(path).map_err(Into::into)
}

pub fn write<P: AsRef<Path>>(data: &str, path: P) -> Result<(), crate::Error> {
    fs::write(path, data).map_err(Into::into)
}

pub fn metadata<P: AsRef<Path>>(path: P) -> Result<fs::Metadata, crate::Error> {
    fs::metadata(path).map_err(Into::into)
}

pub fn clear_file<P: AsRef<Path>>(path: P) -> Result<(), crate::Error> {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .and_then(|file| file.set_len(0))
        .map_err(Into::into)
}

pub fn is_file_empty<P: AsRef<Path>>(path: P) -> Result<bool, crate::Error> {
    fs::metadata(path)
        .map(|metadata| metadata.len() == 0)
        .map_err(Into::into)
}
