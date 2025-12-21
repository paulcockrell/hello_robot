use std::sync::{Arc, Mutex};

use anyhow::{Context, Ok, Result};
use opencv::{core::Vector, imgcodecs, prelude::*, videoio};

#[derive(Debug, Clone)]
pub struct CameraState {
    pub latest_frame: Arc<Mutex<Vec<u8>>>,
}

impl CameraState {
    pub fn new() -> Self {
        CameraState {
            latest_frame: Arc::new(Mutex::new(Vec::<u8>::new())),
        }
    }
}

pub struct Camera {
    cap: videoio::VideoCapture,
}

impl Camera {
    pub fn new() -> Result<Self> {
        let pipeline =
            "libcamerasrc ! video/x-raw,format=BGR,width=640,height=480 ! videoconvert ! appsink";
        let cap = videoio::VideoCapture::from_file(pipeline, videoio::CAP_GSTREAMER)
            .context("Failed to open GStreamer pipeline")?;

        if !cap.is_opened()? {
            anyhow::bail!("Camera was not opened");
        }

        Ok(Camera { cap })
    }

    pub fn frame_mat(&mut self) -> Result<Mat> {
        let mut frame = Mat::default();
        self.cap.read(&mut frame)?;

        if frame.size()?.width == 0 {
            anyhow::bail!("Captured empty frame");
        }

        Ok(frame)
    }

    pub fn frame_jpeg(&mut self) -> Result<Vec<u8>> {
        let mat = self.frame_mat()?;

        let mut buf = Vector::<u8>::new();

        imgcodecs::imencode(".jpg", &mat, &mut buf, &Vector::new())?;

        Ok(buf.to_vec())
    }
}
