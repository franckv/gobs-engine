use std::env;
use std::path::PathBuf;

use futures::io;
use image::DynamicImage;
use thiserror::Error;

use gobs_core::logger;

pub enum AssetType {
    SHADER,
    IMAGE,
    MODEL,
    DATA,
    RESOURCES,
}

#[derive(Debug, Error)]
pub enum LoadingError {
    #[error("asset not found")]
    AssetNotFound(String),
    #[error("io error")]
    IOError(#[from] io::Error),
    #[error("cannot load image")]
    ImageLoadingError(#[from] image::ImageError),
}

pub fn get_asset_dir(file_name: &str, ty: AssetType) -> Result<PathBuf, LoadingError> {
    let current_exe = env::current_exe()?;
    let current_dir = current_exe
        .parent()
        .ok_or(LoadingError::AssetNotFound(file_name.to_owned()))?;
    let path = match ty {
        AssetType::SHADER => current_dir.join("shaders"),
        AssetType::MODEL => current_dir.join("assets/models"),
        AssetType::IMAGE => current_dir.join("assets/textures"),
        AssetType::DATA => current_dir.join("assets/data"),
        AssetType::RESOURCES => current_dir.join("resources"),
    };

    Ok(path.join(file_name))
}

pub async fn load_string(file_name: &str, ty: AssetType) -> Result<String, LoadingError> {
    load_string_sync(file_name, ty)
}

pub fn load_string_sync(file_name: &str, ty: AssetType) -> Result<String, LoadingError> {
    let path = get_asset_dir(file_name, ty)?;

    tracing::debug!(target: logger::RESOURCES, "Loading string: {:?}", path);

    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

pub async fn load_binary(file_name: &str, ty: AssetType) -> Result<Vec<u8>, LoadingError> {
    let path = get_asset_dir(file_name, ty)?;

    tracing::debug!(target: logger::RESOURCES, "Loading bin: {:?}", path);

    let data = std::fs::read(path)?;

    Ok(data)
}

pub async fn load_image(file_name: &str, ty: AssetType) -> Result<DynamicImage, LoadingError> {
    let bytes = load_binary(file_name, ty).await?;
    let img = image::load_from_memory(&bytes)?;

    Ok(img)
}
