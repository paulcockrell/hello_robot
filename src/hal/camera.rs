use std::sync::{Arc, Mutex};

use anyhow::{Context, Ok, Result};
use opencv::{core::Vector, imgcodecs, imgproc, prelude::*, videoio};

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
    last_save: std::time::Instant,
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

        Ok(Camera {
            cap,
            last_save: std::time::Instant::now(),
        })
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

    pub fn grayscale(&mut self) -> Result<Mat> {
        let frame = self.frame_mat()?;
        let mut gray = Mat::default();
        imgproc::cvt_color(&frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

        Ok(gray)
    }

    pub fn save_frame(&mut self) -> Result<bool> {
        // Minimum frame saving frequency of 1 seconds to allow it to complete
        if self.last_save.elapsed().as_secs() >= 1 {
            let filename = "/tmp/frame.jpg";
            let frame = self.grayscale()?;
            opencv::imgcodecs::imwrite(filename, &frame, &opencv::core::Vector::<i32>::new())?;
            self.last_save = std::time::Instant::now();

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
