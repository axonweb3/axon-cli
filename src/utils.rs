use std::{
    borrow::Cow,
    fs::{read_to_string, write, File},
    io::BufReader,
    path::Path,
};

use log::info;
use serde::{de::DeserializeOwned, Serialize};

use crate::types::Result;

pub fn from_json_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
}

pub fn to_json_file<T: Serialize>(val: &T, path: impl AsRef<Path>) -> Result<()> {
    let file = File::create(path.as_ref())?;

    serde_json::to_writer_pretty(&file, val)?;

    file.sync_all()?;

    Ok(())
}

pub fn read_or_create_json_template<
    T: DeserializeOwned + ToOwned<Owned = T> + Serialize + ?Sized,
>(
    path_ref: impl AsRef<Path>,
    val: &T,
) -> Result<Cow<T>> {
    let path = path_ref.as_ref();

    let result = if path.exists() {
        Cow::Owned(from_json_file(path)?)
    } else {
        to_json_file(val, path)?;

        info!("Created template {}", path.to_str().unwrap_or(""));

        Cow::Borrowed(val)
    };

    Ok(result)
}

pub fn read_or_create_plain_template<V: ?Sized + AsRef<str>>(
    path_ref: impl AsRef<Path>,
    val: &V,
) -> Result<Cow<str>> {
    let path = path_ref.as_ref();

    let result = if path.exists() {
        Cow::Owned(read_to_string(path)?)
    } else {
        write(path, val.as_ref().as_bytes())?;

        info!("Created template {}", path.to_str().unwrap_or(""));

        Cow::Borrowed(val.as_ref())
    };

    Ok(result)
}
