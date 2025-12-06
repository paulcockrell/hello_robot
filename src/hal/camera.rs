use anyhow::{Context, Ok, Result};
use opencv::{imgproc, prelude::*, videoio};

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

    pub fn frame(&mut self) -> Result<Mat> {
        let mut frame = Mat::default();
        self.cap.read(&mut frame)?;
        if frame.size()?.width == 0 {
            anyhow::bail!("Captured empty frame");
        }

        // Save frame every 10 seconds
        if self.last_save.elapsed().as_secs() >= 10 {
            let filename = "/tmp/frame.jpg";
            opencv::imgcodecs::imwrite(&filename, &frame, &opencv::core::Vector::<i32>::new())?;
            self.last_save = std::time::Instant::now();
        }

        Ok(frame)
    }

    pub fn grayscale(&mut self) -> Result<Mat> {
        let frame = self.frame()?;
        let mut gray = Mat::default();
        imgproc::cvt_color(&frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

        Ok(gray)
    }
}
