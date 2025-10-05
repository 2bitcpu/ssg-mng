#[derive(Debug, Clone, PartialEq)]
pub struct ImageResizeInfo {
    pub size: u32,
    pub quality: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageResizeConfig {
    pub default: ImageResizeInfo,
    pub thumb: ImageResizeInfo,
}
