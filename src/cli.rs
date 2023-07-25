use std::path::PathBuf;
use animated_wallpapers_rs::image_generator::Generator;

#[tokio::main]
async fn main() {
    let mut generator = Generator::new(PathBuf::from("./test.rn"));
    generator.update().await;
}
