use serde::Serialize;

#[derive(Clone, Debug)]
pub struct ImageUploadDto {
    pub data: Vec<u8>,
}

impl ImageUploadDto {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct ImageUploadResponseDto {
    pub default: String,
    pub thumb: String,
}

impl ImageUploadResponseDto {
    pub fn new(default: String, thumb: String) -> Self {
        Self { default, thumb }
    }
}
