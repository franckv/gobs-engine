use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;

use crate::{ImageFormat, logger};

pub trait ConfigDefault {
    fn register_defaults(config: &mut GobsConfig);
}

#[derive(Clone, Debug)]
pub enum ConfigValue {
    StringValue(String),
    IntValue(u32),
    BoolValue(bool),
    ImageFormatValue(ImageFormat),
}

#[derive(Clone, Debug, Default)]
pub struct Config {
    values: HashMap<String, ConfigValue>,
}

pub trait ConfigReader {
    fn get(&self, key: impl AsRef<str>) -> Option<ConfigValue>;
    fn get_string(&self, key: impl AsRef<str>) -> String;
    fn get_int(&self, key: impl AsRef<str>) -> u32;
    fn get_bool(&self, key: impl AsRef<str>) -> bool;
    fn get_image_format(&self, key: impl AsRef<str>) -> ImageFormat;
}

pub trait ConfigWriter {
    fn register<T: ConfigDefault>(&mut self);
    fn set_string(&mut self, key: impl AsRef<str>, value: &str);
    fn set_int(&mut self, key: impl AsRef<str>, value: u32);
    fn set_bool(&mut self, key: impl AsRef<str>, value: bool);
    fn set_image_format(&mut self, key: impl AsRef<str>, value: ImageFormat);
}

pub type GobsConfig = Arc<RwLock<Config>>;

impl ConfigReader for GobsConfig {
    fn get(&self, key: impl AsRef<str>) -> Option<ConfigValue> {
        self.read().values.get(key.as_ref()).cloned()
    }

    fn get_string(&self, key: impl AsRef<str>) -> String {
        match self.read().values.get(key.as_ref()) {
            Some(ConfigValue::StringValue(val)) => val.clone(),
            _ => {
                tracing::warn!(target: logger::CONFIG, "Invalid config option: {}", key.as_ref());
                "".to_string()
            }
        }
    }

    fn get_int(&self, key: impl AsRef<str>) -> u32 {
        match self.read().values.get(key.as_ref()) {
            Some(ConfigValue::IntValue(val)) => *val,
            _ => {
                tracing::warn!(target: logger::CONFIG, "Invalid config option: {}", key.as_ref());
                0
            }
        }
    }

    fn get_bool(&self, key: impl AsRef<str>) -> bool {
        match self.read().values.get(key.as_ref()) {
            Some(ConfigValue::BoolValue(val)) => *val,
            _ => {
                tracing::warn!(target: logger::CONFIG, "Invalid config option: {}", key.as_ref());
                false
            }
        }
    }

    fn get_image_format(&self, key: impl AsRef<str>) -> ImageFormat {
        match self.read().values.get(key.as_ref()) {
            Some(ConfigValue::ImageFormatValue(val)) => *val,
            _ => {
                tracing::warn!(target: logger::CONFIG, "Invalid config option: {}", key.as_ref());
                ImageFormat::R8g8b8a8Unorm
            }
        }
    }
}

impl ConfigWriter for GobsConfig {
    fn register<T: ConfigDefault>(&mut self) {
        T::register_defaults(self);
    }

    fn set_string(&mut self, key: impl AsRef<str>, value: &str) {
        self.write().values.insert(
            key.as_ref().to_string(),
            ConfigValue::StringValue(value.to_string()),
        );
    }

    fn set_int(&mut self, key: impl AsRef<str>, value: u32) {
        self.write()
            .values
            .insert(key.as_ref().to_string(), ConfigValue::IntValue(value));
    }

    fn set_bool(&mut self, key: impl AsRef<str>, value: bool) {
        self.write()
            .values
            .insert(key.as_ref().to_string(), ConfigValue::BoolValue(value));
    }

    fn set_image_format(&mut self, key: impl AsRef<str>, value: ImageFormat) {
        self.write().values.insert(
            key.as_ref().to_string(),
            ConfigValue::ImageFormatValue(value),
        );
    }
}
