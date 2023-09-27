use crate::ndi_receiver::*;
use color_eyre::eyre::WrapErr;
use crossbeam_utils::atomic::AtomicCell;
use nannou::image::DynamicImage;
use nannou::prelude::*;
use nannou_egui::Egui;
use ndi::Source;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod gui;
mod ndi_receiver;

pub type VideoReceiver = Arc<AtomicCell<Option<DynamicImage>>>;
pub type VideoSender = Arc<AtomicCell<Option<DynamicImage>>>;

fn main() -> color_eyre::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber).context("tracing setup")?;

    tracing::debug!("Initializing NDI");
    ndi::initialize().context("initializing ndi")?;
    nannou::app(model).fullscreen().update(update).run();

    Ok(())
}

fn model(app: &App) -> Model {
    let video_queue = Arc::new(AtomicCell::new(None));
    let video_rx = Arc::clone(&video_queue);
    let video_tx = video_queue;
    let (source_tx, source_rx) = std::sync::mpsc::channel();
    let (selected_source_tx, selected_source_rx) = std::sync::mpsc::channel();

    std::thread::spawn(|| {
        if let Err(err) = recv_ndi(video_tx, selected_source_rx) {
            tracing::error!(error = ?err, "NDI Receiver crashed");
        }
    });

    std::thread::spawn(|| {
        if let Err(err) = discover_sources(source_tx) {
            tracing::error!(error = ?err, "NDI Source discovery crashed");
        }
    });

    let window_id = app
        .new_window()
        .title("NDI Client")
        .view(view)
        .raw_event(raw_window_event)
        .event(event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();

    let egui = Egui::from_window(&window);

    Model {
        texture: None,
        image_recv: video_rx,
        egui,
        ui_open: false,
        source_recv: source_rx,
        sources: Default::default(),
        selected_source: None,
        selected_source_send: selected_source_tx,
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

pub struct Model {
    texture: Option<wgpu::Texture>,
    image_recv: VideoReceiver,
    source_recv: Receiver<Vec<Source>>,
    selected_source_send: Sender<Source>,
    egui: Egui,
    ui_open: bool,
    sources: Vec<Source>,
    selected_source: Option<Source>,
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if let Some(image) = model.image_recv.take() {
        let texture = wgpu::Texture::from_image(app, &image);
        model.texture = Some(texture);
    }
    if let Ok(sources) = model.source_recv.try_recv() {
        model.sources = sources;
    }
    if model.selected_source.is_none() && !model.sources.is_empty() {
        let source = model.sources.first().unwrap().clone();
        model.selected_source = Some(source.clone());
        model.selected_source_send.send(source).unwrap();
    }
    gui::update(app, model, _update);
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);
    let draw = app.draw();
    if let Some(ref texture) = model.texture {
        draw.texture(texture);
    }
    draw.to_frame(app, &frame).unwrap();
    if model.ui_open {
        model.egui.draw_to_frame(&frame).unwrap();
    }
}

fn event(app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        KeyPressed(Key::Escape) => {
            app.quit();
        }
        KeyPressed(Key::F12) => {
            model.ui_open = !model.ui_open;
        }
        _ => {}
    }
}
