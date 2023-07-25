use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use chrono::{Timelike, Utc};
use image::{ImageOutputFormat, RgbaImage};
use rune::{Context, Diagnostics, Source, Sources, Unit, Vm, Any, Module, ContextError};
use rune::termcolor::{ColorChoice, StandardStream};

#[derive(Debug, Any)]
#[rune(name = "Wallpaper")]
pub struct WallpaperBuilder {
    current_image: RgbaImage,
}

pub struct Wallpaper {
    image: RgbaImage,
}

pub struct Generator {
    context: Context,
    unit: Unit,
    frame: u32,
    max_frames: u32,
}

fn mix_colors (a: u32, b: u32, v: f32) -> u32 {
    (v * a as f32 + (1.0 - v) * b as f32) as u32
}

impl WallpaperBuilder {
    pub fn new() -> Self {
        Self {
            current_image: RgbaImage::default(),
        }
    }

    pub fn solid(mut self, color: Vec<u32>, width: u32, height: u32) -> Self {
        let mut im = RgbaImage::new(width, height);
        for pixel in im.pixels_mut() {
            pixel.0[0] = color[0] as u8;
            pixel.0[1] = color[1] as u8;
            pixel.0[2] = color[2] as u8;
            pixel.0[3] = 255;
        }
        self.current_image = im;
        self
    }

    pub fn vertical_gradient(mut self, top: Vec<u32>, mid: Vec<u32>,
                             bottom: Vec<u32>, width: u32, height: u32) -> Self {
        let mut im = RgbaImage::new(width, height);
        for (_, y, pixel) in im.enumerate_pixels_mut() {
            let value = y as f32 / height as f32;
            let color = if value > 0.5 {
                let value = (value - 0.5) * 2.0;
                vec![
                    mix_colors(bottom[0], mid[0], value),
                    mix_colors(bottom[1], mid[1], value),
                    mix_colors(bottom[2], mid[2], value),
                ]
            } else {
                let value = value * 2.0;
                vec![
                    mix_colors(mid[0], top[0], value),
                    mix_colors(mid[1], top[1], value),
                    mix_colors(mid[2], top[2], value),
                ]
            };
            pixel.0[0] = color[0] as u8;
            pixel.0[1] = color[1] as u8;
            pixel.0[2] = color[2] as u8;
            pixel.0[3] = 255;
        }
        self.current_image = im;
        self
    }

    pub fn load_image(mut self, path: &str) -> Self {
        let im = image::open(path).expect("Can't load image");
        self.current_image = im.into_rgba8();
        self
    }

    pub fn append_image(mut self, path: &str, x: i64, y: i64) -> Self {
        let im = image::open(path).expect("Can't load image");
        let im = im.into_rgba8();
        image::imageops::overlay(&mut self.current_image, &im, x, y);
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
        let file = if path.exists() {
            File::options()
                .write(true)
                .truncate(true)
                .open(path)
                .expect("Can't open file to write")
        } else {
            File::create(path).expect("Can't create file")
        };
        let mut writer = BufWriter::new(file);
        self.image
            .write_to(&mut writer, ImageOutputFormat::Png)
            .expect("Can't write image.");
        writer.flush().unwrap();
    }
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::new();
    module.ty::<WallpaperBuilder>()?;
    module.inst_fn("solid", WallpaperBuilder::solid)?;
    module.inst_fn("load_image", WallpaperBuilder::load_image)?;
    module.inst_fn("append_image", WallpaperBuilder::append_image)?;
    module.inst_fn("vertical_gradient", WallpaperBuilder::vertical_gradient)?;

    Ok(module)
}

impl Generator {
    pub async fn new(path: &PathBuf) -> Self {
        let m = module().expect("Can't load wallpaper builder module");
        let mut context = Context::with_default_modules().expect("Can't create context");
        context.install(m).expect("Can't install module");
        let mut diagnostics = Diagnostics::new();
        let mut sources = Sources::new();
        sources.insert(Source::from_path(path).expect("Can't load script"));
        let result = rune::prepare(&mut sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .build();
        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            diagnostics.emit(&mut writer, &sources).unwrap();
        }
        let unit = result.unwrap();
        let runtime = Arc::new(context.runtime());
        let mut vm = Vm::new(runtime, Arc::new(unit.clone()));
        let output = vm.call(["init"], ())
            .expect("Error when running update");
        let max_frames: u32 = rune::FromValue::from_value(output)
            .expect("Can't convert");
        Self {
            context,
            unit,
            frame: 0,
            max_frames,
        }
    }

    pub async fn update(&mut self){
        let runtime = Arc::new(self.context.runtime());
        let mut vm = Vm::new(runtime, Arc::new(self.unit.clone()));
        let wallpaper = WallpaperBuilder::new();
        let frame = (self.frame + 1) % self.max_frames;
        let now = Utc::now();
        let hours = now.hour();
        let minutes = now.minute();
        let seconds = now.second();
        let output = vm.call(["update"], (wallpaper, frame, hours,
                                          minutes, seconds))
            .expect("Error when running update");
        let result: WallpaperBuilder = rune::FromValue::from_value(output)
            .expect("Can't convert");
        let wallpaper = result.build();
        wallpaper.save(&PathBuf::from("./test.png"));
    }
}