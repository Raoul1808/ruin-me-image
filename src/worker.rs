use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender, TryRecvError, channel},
    thread::{JoinHandle, spawn},
};

use image::{DynamicImage, ImageError};

use crate::commands::CommandQueue;

struct Worker {
    handle: JoinHandle<()>,
    sender: Sender<WorkCommand>,
    receiver: Receiver<WorkerResult>,
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

pub enum WorkerResult {
    Progress(usize),
    Finished(DynamicImage),
    Error(ImageError),
}

enum WorkCommand {
    LoadImage(PathBuf),
    Render {
        queue: CommandQueue,
        img: DynamicImage,
    },
}

fn image_worker(sender: Sender<WorkerResult>, receiver: Receiver<WorkCommand>) {
    loop {
        match receiver.recv() {
            Ok(c) => match c {
                WorkCommand::LoadImage(path) => {
                    let res = match image::open(path) {
                        Ok(img) => WorkerResult::Finished(img),
                        Err(e) => WorkerResult::Error(e),
                    };
                    sender.send(res).unwrap();
                }
                WorkCommand::Render { queue, img } => {
                    let mut img = img;
                    for (i, command) in queue.into_iter().enumerate() {
                        img = command.execute(img);
                        sender.send(WorkerResult::Progress(i + 1)).unwrap();
                    }
                    sender.send(WorkerResult::Finished(img)).unwrap();
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

    pub fn request_render(&self, queue: CommandQueue, img: DynamicImage) {
        self.worker()
            .sender
            .send(WorkCommand::Render { queue, img })
            .expect("worker thread unexpectedly down!!");
    }

    pub fn try_recv(&self) -> Option<WorkerResult> {
        match self.worker().receiver.try_recv() {
            Ok(res) => Some(res),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!("worker thread unexpectedly down!!"),
        }
    }
}
