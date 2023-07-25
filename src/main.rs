#![allow(unused)]

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use egui::epaint::stats;
use egui::FontDefinitions;
use egui_wgpu_backend::ScreenDescriptor;
use egui_winit_platform::{Platform, PlatformDescriptor};
use lazy_static::lazy_static;
use tokio::time::sleep;
use tray_icon::{ClickEvent, menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem}, TrayIconBuilder};

use tray_icon::TrayEvent;
use wgpu::InstanceDescriptor;
use winit::dpi::PhysicalSize;
use winit::event::Event;
use winit::event::WindowEvent;

use winit::event_loop::{ControlFlow, EventLoopBuilder};
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::window::{Window, WindowBuilder};
use animated_wallpapers_rs::image_generator::Generator;

struct SettingWindowState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Window,
    platform: Platform,
    egui_rpass: egui_wgpu_backend::RenderPass,
}

struct State {
    generator: Option<Generator>,
    show_window: bool,
}

lazy_static! {
    static ref STATE: Arc<RwLock<State>> = Arc::new(RwLock::new(State {
        generator: None,
        show_window: false,
    }));
}

async fn worker() {
    loop {
        let mut state = STATE.write().await;
        if let Some(generator) = state.generator.as_mut() {
            generator.update().await;
            let mut path = std::env::current_dir().unwrap();
            path.push("test.png");
            wallpaper::set_from_path(path.to_str().unwrap()).unwrap();
        }
        sleep(Duration::from_millis(500)).await;
    }
}

fn main() {
    let path = "./icon.png";
    let icon = load_icon(std::path::Path::new(path));
    let runtime = tokio::runtime::Runtime::new().expect("Can't create tokio runtime");

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

    let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

    #[cfg(not(target_os = "linux"))]
        let mut tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(Menu::new()))
            .with_tooltip("Animated wallpaper")
            .with_icon(icon)
            .build()
            .unwrap(),
    );

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayEvent::receiver();

    let mut settings_window: Option<SettingWindowState> = None;

    runtime.spawn(async {
        worker().await;
    });

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Poll;
        if let Some(settings_window_state) = settings_window.as_mut() {
            settings_window_state.platform.handle_event(&event);
            match event {
                Event::WindowEvent { event, window_id } => {
                    if window_id != settings_window_state.window.id() {
                        return;
                    }
                    match event {
                        WindowEvent::CloseRequested => {
                            settings_window = None;
                            let mut state = runtime.block_on(STATE.write());
                            state.show_window = false;
                        }
                        _ => (),
                    }
                }
                Event::RedrawRequested(window_id) => {
                    if window_id != settings_window_state.window.id() {
                        return;
                    }
                    let output = settings_window_state.surface.get_current_texture().unwrap();
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = settings_window_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });
                    {
                        settings_window_state.platform.begin_frame();
                        let ctx = &settings_window_state.platform.context();
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Script");
                                if ui.button("Open").clicked() {
                                    let path = std::env::current_dir().unwrap();
                                    let res = rfd::FileDialog::new()
                                        .add_filter("Rune script", &["rn"])
                                        .set_directory(&path)
                                        .pick_file();
                                    if let Some(file) = res {
                                        let mut state = runtime.block_on(STATE.write());
                                        let mut dir = file.clone();
                                        dir.pop();
                                        std::env::set_current_dir(dir);
                                        state.generator.insert(Generator::new(file));
                                        wallpaper::set_mode(wallpaper::Mode::Fit).unwrap();
                                    }
                                }
                            });
                        });
                        let full_output = settings_window_state.platform
                            .end_frame(Some(&settings_window_state.window));
                        let paint_jobs = settings_window_state.platform
                            .context()
                            .tessellate(full_output.shapes);
                        // Upload all resources for the GPU.
                        let screen_descriptor = ScreenDescriptor {
                            physical_width: settings_window_state.config.width,
                            physical_height: settings_window_state.config.height,
                            scale_factor: settings_window_state.window.scale_factor() as f32,
                        };
                        let tdelta: egui::TexturesDelta = full_output.textures_delta;
                        settings_window_state.egui_rpass
                            .add_textures(&settings_window_state.device,
                                          &settings_window_state.queue,
                                          &tdelta)
                            .expect("Something went wrong");
                        settings_window_state.egui_rpass.update_buffers(
                            &settings_window_state.device,
                            &settings_window_state.queue,
                            &paint_jobs, &screen_descriptor);

                        // Record all render passes.
                        settings_window_state.egui_rpass
                            .execute(
                                &mut encoder,
                                &view,
                                &paint_jobs,
                                &screen_descriptor,
                                Some(
                                wgpu::Color {
                                    r: 0.01,
                                    g: 0.01,
                                    b: 0.02,
                                    a: 1.0,
                                }),
                            )
                            .unwrap();
                    }
                    settings_window_state.queue.submit(
                        std::iter::once(encoder.finish()));
                    output.present();
                }
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    settings_window_state.window.request_redraw();
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

            let mut state = runtime.block_on(STATE.write());
            if state.show_window {
                return;
            }

            if !show_window {
                return;
            }
            state.show_window = true;
            let window = WindowBuilder::new()
                .with_title("Settings")
                .with_inner_size(PhysicalSize::new(500, 250))
                .with_resizable(false)
                .with_active(true)
                .build(&event_loop).unwrap();

            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                dx12_shader_compiler: Default::default(),
            });
            let surface = unsafe { instance.create_surface(&window).unwrap() };

            // WGPU 0.11+ support force fallback (if HW implementation not supported), set it to true or false (optional).
            let adapter = runtime.block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }))
                .unwrap();

            let (device, queue) = runtime.block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            ))
                .unwrap();

            let size = window.inner_size();
            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps.formats.iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]);
            let mut config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
            };
            surface.configure(&device, &config);

            // We use the egui_winit_platform crate as the platform.
            let mut platform = Platform::new(PlatformDescriptor {
                physical_width: size.width,
                physical_height: size.height,
                scale_factor: window.scale_factor(),
                font_definitions: FontDefinitions::default(),
                style: Default::default(),
            });

            let egui_rpass = egui_wgpu_backend::RenderPass::new(&device, surface_format, 1);

            let _ = settings_window.insert(
                SettingWindowState {
                    surface,
                    device,
                    queue,
                    config,
                    size,
                    window,
                    platform,
                    egui_rpass,
                }
            );
        }
    })
}

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