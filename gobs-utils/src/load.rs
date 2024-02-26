use std::env;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use anyhow::Result;
use image::DynamicImage;

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
        AssetType::MODEL => current_dir.join("assets/models"),
        AssetType::IMAGE => current_dir.join("assets/textures"),
    };

    Ok(path.join(file_name))
}

pub async fn load_string(file_name: &str, ty: AssetType) -> Result<String> {
    let path = get_asset_dir(file_name, ty)?;

    log::debug!("Loading string: {:?}", path);

    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

pub async fn load_binary(file_name: &str, ty: AssetType) -> Result<Vec<u8>> {
    let path = get_asset_dir(file_name, ty)?;

    log::info!("Loading bin: {:?}", path);

    let data = std::fs::read(path)?;

    Ok(data)
}

pub async fn load_image(file_name: &str, ty: AssetType) -> Result<DynamicImage> {
    let bytes = load_binary(file_name, ty).await?;
    let img = image::load_from_memory(&bytes)?;

    Ok(img)
}
