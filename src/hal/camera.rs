use anyhow::{Context, Result};
use opencv::{core, imgproc, prelude::*, videoio};

pub struct Camera {
    cap: videoio::VideoCapture,
}
