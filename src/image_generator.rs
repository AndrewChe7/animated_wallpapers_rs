use std::path::PathBuf;
use std::sync::Arc;
use image::RgbaImage;
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
        self.image.save(path).expect("Can't save image");
    }
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::new();
    module.ty::<WallpaperBuilder>()?;
    module.inst_fn("solid", WallpaperBuilder::solid)?;
    module.inst_fn("load_image", WallpaperBuilder::load_image)?;
    module.inst_fn("append_image", WallpaperBuilder::append_image)?;

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
        Self {
            context,
            unit,
        }
    }

    pub async fn update(&mut self){
        let runtime = Arc::new(self.context.runtime());
        let mut vm = Vm::new(runtime, Arc::new(self.unit.clone()));
        let wallpaper = WallpaperBuilder::new();
        let output = vm.call(["update"], (wallpaper, ))
            .expect("Error when running update");
        let result: WallpaperBuilder = rune::FromValue::from_value(output)
            .expect("Can't convert");
        let wallpaper = result.build();
        wallpaper.save(&PathBuf::from("./test.png"));
    }
}