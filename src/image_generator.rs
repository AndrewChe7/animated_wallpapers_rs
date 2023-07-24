use std::path::PathBuf;
use std::sync::Arc;
use image::RgbaImage;
use rune::{Context, Source, Sources};
use tokio::fs::File;
use tokio::io;


pub struct WallpaperBuilder {
    current_image: RgbaImage,
}

pub struct Wallpaper {
    image: RgbaImage,
}

pub struct Generator {
    wallpaper: WallpaperBuilder,
    context: Context,
    sources: Sources,
}

impl WallpaperBuilder {
    pub fn new() -> Self {
        Self {
            current_image: RgbaImage::default(),
        }
    }

    pub fn solid(mut self, color: &[u8; 3], width: u32, height: u32) -> Self {
        let mut im = RgbaImage::new(width, height);
        for pixel in im.pixels_mut() {
            pixel.0[0] = color[0];
            pixel.0[1] = color[1];
            pixel.0[2] = color[2];
            pixel.0[3] = 255;
        }
        self.current_image = im;
        self
    }

    pub fn load_image(mut self, path: &PathBuf) -> Self {
        let im = image::open(path).expect("Can't load image");
        self.current_image = im.into_rgba8();
        self
    }

    pub fn append_image(mut self, path: &PathBuf) -> Self {
        let im = image::open(path).expect("Can't load image");
        let im = im.into_rgba8();
        image::imageops::overlay(&mut self.current_image, &im, 0, 0);
        self
    }

    pub fn build(self) -> Wallpaper {
        Wallpaper {
            image: self.current_image,
        }
    }
}

impl Wallpaper {
    pub fn save(&self, path: &PathBuf) {
        self.image.save(path).expect("Can't save image");
    }
}

impl Generator {
    pub async fn new(path: &PathBuf) -> Self {
        let wallpaper = WallpaperBuilder::new();
        let context = Context::with_default_modules().expect("Can't create context");
        let mut sources = Sources::new();
        sources.insert(Source::from_path(path).expect("Can't load script"));

        Self {
            wallpaper,
            context,
            sources,
        }
    }

    pub async fn run_script(&mut self) {
        todo!()
    }
}