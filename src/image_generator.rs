use std::path::PathBuf;
use image::{ImageBuffer, RgbaImage};

pub struct WallpaperBuilder {
    current_image: RgbaImage,
}

pub struct Wallpaper {
    image: RgbaImage,
}

impl WallpaperBuilder {
    pub fn new() -> Self {
        Self {
            current_image: RgbaImage::default(),
        }
    }

    pub fn load_image(mut self, path: &PathBuf) -> Self {
        let im = image::open(path).expect("Can't load image");
        self.current_image = im.into_rgba8();
        self
    }

    pub fn build(self) -> Wallpaper {
        Wallpaper {
            image: self.current_image,
        }
    }
}

impl Wallpaper {
    pub fn save(path: &PathBuf) {
        todo!()
    }
}