# animated_wallpapers_rs
Animated wallpaper engine written in rust

> **Warning**\
> This project is WIP, so the performance is VERY VERY LOW. I think tokio framework captures all threads, but I don't know. If you know how to fix it, please make PR or issue.

# Scripting

Program uses [Rune](https://github.com/rune-rs/rune) scripting language. 
Over it there is some helpful functions that are used to generate wallpaper:
``` rust
solid(wallpaper, color: [u32, u32, u32], width: u32, height: u32); // same as wallpaper.solid(...)
vertical_gradient(wallpaper, top: [u32, u32, u32], mid: [u32, u32, u32], bottom: [u32, u32, u32], width: u32, height: u32);
load_image(wallpaper, path: &str);
append_image(wallpaper, path: &str, x: i64, y: i64);
```
For more information check [example](https://github.com/AndrewChe7/animated_wallpapers_rs/blob/master/test_wp/test.rn).

In scripts must be two functions:
``` rust
pub fn init(); // returns frame count for animated GIFs (not implemented yet)
pub fn update(wallpaper, frame, hours, minutes, seconds); // returns final wallpaper
```
