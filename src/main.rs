#![allow(unused)]

use std::sync::{Arc, RwLock};
use lazy_static::lazy_static;
use tray_icon::{ClickEvent, menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem}, TrayIconBuilder};

use tray_icon::TrayEvent;
use winit::event::Event;
use winit::event::WindowEvent;

use winit::event_loop::{ControlFlow, EventLoopBuilder};
use winit::window::Window;
use animated_wallpapers_rs::image_generator::Generator;

struct State {
    generator: Option<Generator>,
    show_window: bool,
    runtime: tokio::runtime::Runtime,
}

lazy_static! {
    static ref STATE: Arc<RwLock<State>> = Arc::new(RwLock::new(State {
        generator: None,
        show_window: false,
        runtime: tokio::runtime::Runtime::new()
            .expect("Can't create tokio runtime"),
    }));
}

fn main() {
    let path = "./icon.png";
    let icon = load_icon(std::path::Path::new(path));

    // Since winit doesn't use gtk on Linux, and we need gtk for
    // the tray icon to show up, we need to spawn a thread
    // where we initialize gtk and create the tray_icon
    #[cfg(target_os = "linux")]
    std::thread::spawn(|| {
        use tray_icon::menu::Menu;

        gtk::init().unwrap();
        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(Menu::new()))
            .with_icon(icon)
            .build()
            .unwrap();

        gtk::main();
    });

    let event_loop = EventLoopBuilder::new().build();

    #[cfg(not(target_os = "linux"))]
        let mut tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(Menu::new()))
            .with_tooltip("winit - awesome windowing lib")
            .with_icon(icon)
            .build()
            .unwrap(),
    );

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayEvent::receiver();

    let mut settings_window: Option<Window> = None;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Poll;

        if let Some(window) = settings_window.as_mut() {
            match event {
                Event::WindowEvent { event, window_id } => {
                    if window_id != window.id() {
                        return;
                    }
                    match event {
                        WindowEvent::CloseRequested => {
                            settings_window = None;
                            STATE.write().unwrap().show_window = false;
                        }
                        _ => (),
                    }
                }
                Event::RedrawRequested(window_id) => {
                    if window_id != window.id() {
                        return;
                    }

                }
                _ => (),
            }
        }

        if let Ok(event) = tray_channel.try_recv() {
            let mut show_window = false;
            match event.event {
                ClickEvent::Left => {
                    show_window = true;
                }
                ClickEvent::Right => {
                    control_flow.set_exit();
                }
                _ => {

                }
            }

            if STATE.read().unwrap().show_window {
                return;
            }

            if !show_window {
                return;
            }

            STATE.write().unwrap().show_window = true;
            let _ = settings_window.insert(Window::new(event_loop).unwrap());
        }
    })
}

// struct MyApp {
//     state: Arc<RwLock<State>>,
// }
//
// impl MyApp {
//     fn new(state: &Arc<RwLock<State>>) -> Self {
//         Self {
//             state: state.clone(),
//         }
//     }
// }
//
// impl eframe::App for MyApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.horizontal(|ui| {
//                 ui.label("Script");
//                 if ui.button("Open").clicked() {
//                     let path = std::env::current_dir().unwrap();
//                     let res = rfd::FileDialog::new()
//                         .add_filter("Rune script", &["rn"])
//                         .set_directory(&path)
//                         .pick_file();
//                     if let Some(file) = res {
//                         self.state.write().unwrap().generator.insert(Generator::new(file));
//                     }
//                 }
//             });
//         });
//     }
// }


fn load_icon(path: &std::path::Path) -> tray_icon::icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::icon::Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .expect("Failed to open icon")
}