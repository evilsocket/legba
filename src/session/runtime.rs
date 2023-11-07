use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use super::Error;
use crate::Credentials;

#[derive(Debug)]
pub(crate) struct Runtime {
    stop: AtomicBool,
    creds_tx: async_channel::Sender<Credentials>,
    creds_rx: async_channel::Receiver<Credentials>,
    speed: AtomicUsize,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new(1)
    }
}

impl Runtime {
    pub(crate) fn new(concurrency: usize) -> Self {
        let (creds_tx, creds_rx) = async_channel::bounded(concurrency);
        Self {
            stop: AtomicBool::new(false),
            speed: AtomicUsize::new(0),
            creds_tx,
            creds_rx,
        }
    }

    pub fn is_stop(&self) -> bool {
        self.stop.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    pub fn set_speed(&self, rps: usize) {
        self.speed.store(rps, Ordering::Relaxed);
    }

    pub fn get_speed(&self) -> usize {
        self.speed.load(Ordering::Relaxed)
    }

    pub async fn send_credentials(&self, creds: Credentials) -> Result<(), Error> {
        self.creds_tx.send(creds).await.map_err(|e| e.to_string())
    }

    pub async fn recv_credentials(&self) -> Result<Credentials, Error> {
        self.creds_rx.recv().await.map_err(|e| e.to_string())
    }
}
