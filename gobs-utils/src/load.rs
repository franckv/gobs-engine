use std::env;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use anyhow::Result;
use log::info;

pub enum AssetType {
    SHADER,
    IMAGE,
    MODEL,
}

pub fn get_asset_dir(file_name: &str, ty: AssetType) -> Result<PathBuf> {
    let current_exe = env::current_exe()?;
    let current_dir = current_exe
        .parent()
        .ok_or(Error::from(ErrorKind::NotFound))?;
    let path = match ty {
        AssetType::SHADER => current_dir.join("shaders"),
        AssetType::MODEL => current_dir.join("assets"),
        AssetType::IMAGE => current_dir.join("assets"),
    };

    Ok(path.join(file_name))
}

pub async fn load_string(file_name: &str, ty: AssetType) -> Result<String> {
    let path = get_asset_dir(file_name, ty)?;

    info!("Loading string: {:?}", path);

    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

pub async fn load_binary(file_name: &str, ty: AssetType) -> Result<Vec<u8>> {
    let path = get_asset_dir(file_name, ty)?;

    info!("Loading bin: {:?}", path);

    let data = std::fs::read(path)?;

    Ok(data)
}
