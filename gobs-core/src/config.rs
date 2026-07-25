use std::collections::HashMap;

use crate::{ImageFormat, logger};

pub trait ConfigDefault {
    fn register_defaults(config: &mut Config);
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

impl Config {
    pub fn register<T: ConfigDefault>(&mut self) {
        T::register_defaults(self);
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<&ConfigValue> {
        self.values.get(key.as_ref())
    }

    pub fn get_string(&self, key: impl AsRef<str>) -> &str {
        match self.values.get(key.as_ref()) {
            Some(ConfigValue::StringValue(val)) => val,
            _ => {
                tracing::warn!(target: logger::CONFIG, "Invalid config option: {}", key.as_ref());
                ""
            }
        }
    }

    pub fn get_int(&self, key: impl AsRef<str>) -> u32 {
        match self.values.get(key.as_ref()) {
            Some(ConfigValue::IntValue(val)) => *val,
            _ => {
                tracing::warn!(target: logger::CONFIG, "Invalid config option: {}", key.as_ref());
                0
            }
        }
    }

    pub fn get_bool(&self, key: impl AsRef<str>) -> bool {
        match self.values.get(key.as_ref()) {
            Some(ConfigValue::BoolValue(val)) => *val,
            _ => {
                tracing::warn!(target: logger::CONFIG, "Invalid config option: {}", key.as_ref());
                false
            }
        }
    }

    pub fn get_image_format(&self, key: impl AsRef<str>) -> ImageFormat {
        match self.values.get(key.as_ref()) {
            Some(ConfigValue::ImageFormatValue(val)) => *val,
            _ => {
                tracing::warn!(target: logger::CONFIG, "Invalid config option: {}", key.as_ref());
                ImageFormat::R8g8b8a8Unorm
            }
        }
    }

    pub fn set_string(&mut self, key: impl AsRef<str>, value: &str) {
        self.values.insert(
            key.as_ref().to_string(),
            ConfigValue::StringValue(value.to_string()),
        );
    }

    pub fn set_int(&mut self, key: impl AsRef<str>, value: u32) {
        self.values
            .insert(key.as_ref().to_string(), ConfigValue::IntValue(value));
    }

    pub fn set_bool(&mut self, key: impl AsRef<str>, value: bool) {
        self.values
            .insert(key.as_ref().to_string(), ConfigValue::BoolValue(value));
    }

    pub fn set_image_format(&mut self, key: impl AsRef<str>, value: ImageFormat) {
        self.values.insert(
            key.as_ref().to_string(),
            ConfigValue::ImageFormatValue(value),
        );
    }
}
