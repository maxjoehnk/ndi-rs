use crate::VideoSender;
use image::{self, DynamicImage, ImageBuffer, Pixel};
use ndi::{FourCCVideoType, FrameType, Recv, RecvColorFormat, Source, VideoData};
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone)]
pub struct NdiSource {
    pub name: String,
    pub source: Source,
}

pub fn recv_ndi(video_tx: VideoSender, rx: Receiver<Source>) -> color_eyre::Result<()> {
    tracing::debug!("Setting up NDI receiver");
    let mut recv = ndi::recv::RecvBuilder::new()
        .color_format(RecvColorFormat::RGBX_RGBA)
        .build()?;

    let source = rx.recv()?;
    tracing::debug!("Connecting to source: {source:?}");
    recv.connect(&source);
    tracing::info!("Connected to source {source:?}");
    loop {
        tracing::debug!("Waiting for frame");
        match recv_ndi_frame(&recv) {
            Ok(image) => video_tx.store(Some(image)),
            Err(err) => tracing::error!(error = ?err, "Error receiving frame"),
        }

        if let Ok(new_source) = rx.try_recv() {
            tracing::info!("Received new source: {new_source:?}");
            recv.disconnect();
            recv.connect(&new_source);
            tracing::info!("Connected to source {new_source:?}");
        }
    }
}

pub fn discover_sources(sender: Sender<Vec<NdiSource>>) -> color_eyre::Result<()> {
    tracing::debug!("Setting up NDI finder");
    let find = ndi::find::FindBuilder::new().build()?;

    loop {
        tracing::debug!("Finding NDI sources");
        let sources = find.current_sources(u128::MAX)?;

        tracing::info!("Found NDI sources: {sources:?}");

        let sources = sources
            .into_iter()
            .map(|source| NdiSource {
                name: source.get_name(),
                source,
            })
            .collect::<Vec<_>>();

        sender.send(sources)?;

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

fn recv_ndi_frame(recv: &Recv) -> color_eyre::Result<DynamicImage> {
    let mut video_data = None;
    let frame_type = recv.capture_video(&mut video_data, u32::MAX);
    match frame_type {
        FrameType::Video => {
            if let Some(video_data) = video_data {
                tracing::debug!("Received video frame: {video_data:?}");
                let size =
                    video_data.height() * video_data.line_stride_in_bytes().unwrap_or_default();
                color_eyre::eyre::ensure!(!video_data.p_data().is_null(), "Video data was null");
                let buffer =
                    unsafe { std::slice::from_raw_parts(video_data.p_data(), size as usize) };
                let frame = Vec::from_iter(buffer.to_owned());
                let image = match video_data.four_cc() {
                    FourCCVideoType::RGBA | FourCCVideoType::RGBX => {
                        decode::<image::Rgba<u8>>(frame, video_data)
                            .map(DynamicImage::ImageRgba8)?
                    }
                    // FourCCVideoType::BGRA | FourCCVideoType::BGRX => {
                    //     decode::<image::Bgra<u8>>(frame, video_data)
                    //         .map(DynamicImage::ImageBgra8)?
                    // }
                    video_type => color_eyre::eyre::bail!("Unsupported video type: {video_type:?}"),
                };

                Ok(image)
            } else {
                Err(color_eyre::eyre::eyre!("Video data was null"))
            }
        }
        frame_type => {
            tracing::warn!("Received non-video frame: {frame_type:?}");
            Err(color_eyre::eyre::eyre!("Frame type was not video"))
        }
    }
}

fn decode<T: Pixel<Subpixel = u8> + 'static>(
    frame: Vec<u8>,
    video_data: VideoData,
) -> color_eyre::Result<ImageBuffer<T, Vec<u8>>> {
    let image =
        image::ImageBuffer::<T, _>::from_vec(video_data.width(), video_data.height(), frame)
            .ok_or_else(|| color_eyre::eyre::eyre!("Failed to create image"))?;

    Ok(image)
}
