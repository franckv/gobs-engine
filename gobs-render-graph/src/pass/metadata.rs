use std::collections::HashMap;

use uuid::Uuid;

use crate::pass::{Attachment, AttachmentType};

pub type PassId = Uuid;

pub struct PassMetaData {
    pub id: PassId,
    pub name: String,
    pub config: String,
    pub attachments: HashMap<String, Attachment>,
    pub input_attachments: Vec<String>,
    pub color_attachments: Vec<String>,
    pub depth_attachments: Vec<String>,
    pub image_attachments: Vec<String>,
}

impl PassMetaData {
    pub fn new(name: &str, config: &str) -> Self {
        Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            config: config.to_string(),
            attachments: HashMap::new(),
            input_attachments: Vec::new(),
            color_attachments: Vec::new(),
            depth_attachments: Vec::new(),
            image_attachments: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_attachment(&mut self, name: &str, attachment: Attachment) {
        match attachment.ty {
            AttachmentType::Input => self.input_attachments.push(name.to_string()),
            AttachmentType::Color => self.color_attachments.push(name.to_string()),
            AttachmentType::Depth => self.depth_attachments.push(name.to_string()),
            AttachmentType::ImageStorage => self.image_attachments.push(name.to_string()),
            _ => unimplemented!(),
        }

        self.attachments.insert(name.to_string(), attachment);
    }
}
