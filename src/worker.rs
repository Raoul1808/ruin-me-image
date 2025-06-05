use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender, TryRecvError, channel},
    thread::{JoinHandle, spawn},
};

use image::{DynamicImage, ImageError};
use thiserror::Error;

struct Worker {
    handle: JoinHandle<()>,
    sender: Sender<WorkCommand>,
    receiver: Receiver<Result<DynamicImage, WorkerError>>,
}

pub struct ImageWorker {
    worker: Option<Worker>,
}

impl Drop for ImageWorker {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            drop(worker.sender);
            drop(worker.receiver);
            worker.handle.join().unwrap();
        }
    }
}

#[derive(Debug, Error)]
pub enum WorkerError {
    #[error("an error occurred while loading image: {0}")]
    ImageError(ImageError),
}

enum WorkCommand {
    LoadImage(PathBuf),
}

fn image_worker(
    sender: Sender<Result<DynamicImage, WorkerError>>,
    receiver: Receiver<WorkCommand>,
) {
    loop {
        match receiver.recv() {
            Ok(c) => match c {
                WorkCommand::LoadImage(path) => {
                    let img = image::open(path).map_err(WorkerError::ImageError);
                    sender.send(img).unwrap();
                }
            },
            Err(_) => {
                eprintln!("Image worker thread shutdown");
                break;
            }
        }
    }
}

impl ImageWorker {
    pub fn new() -> Self {
        let (req_sender, req_receiver) = channel();
        let (res_sender, res_receiver) = channel();
        let handle = spawn(move || image_worker(res_sender, req_receiver));
        let worker = Worker {
            handle,
            sender: req_sender,
            receiver: res_receiver,
        };
        Self {
            worker: Some(worker),
        }
    }

    fn worker(&self) -> &Worker {
        self.worker
            .as_ref()
            .expect("worker thread uninitialized????")
    }

    pub fn request_image_load(&self, path: PathBuf) {
        self.worker()
            .sender
            .send(WorkCommand::LoadImage(path))
            .expect("worker thread unexpectedly down!!");
    }

    pub fn try_recv(&self) -> Option<Result<DynamicImage, WorkerError>> {
        match self.worker().receiver.try_recv() {
            Ok(res) => Some(res),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!("worker thread unexpectedly down!!"),
        }
    }
}
