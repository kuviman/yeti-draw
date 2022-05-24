use super::*;

use serde::de::DeserializeOwned;
use std::sync::{Mutex, MutexGuard};

pub struct AutoSaved<T: Default + Serialize + DeserializeOwned> {
    state: Arc<Mutex<State<T>>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl<T: Default + Serialize + DeserializeOwned> Drop for AutoSaved<T> {
    fn drop(&mut self) {
        {
            let mut state = self.state.lock().unwrap();
            state.dropped = true;
            state.save_if_needed();
        }
        let thread = self.thread.take().unwrap();
        thread.thread().unpark();
        thread.join().unwrap();
    }
}

struct State<T> {
    dropped: bool,
    mutated: bool,
    path: std::path::PathBuf,
    last_touch: std::time::Instant,
    last_save: std::time::Instant,
    value: Option<T>,
}

impl<T: Default + Serialize + DeserializeOwned> State<T> {
    fn new(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            last_touch: std::time::Instant::now(),
            last_save: std::time::Instant::now(),
            path: path.as_ref().to_owned(),
            mutated: false,
            dropped: false,
            value: None,
        }
    }
    fn touch(&mut self) {
        if self.value.is_none() {
            let value: T;
            if self.path.is_file() {
                let file = std::fs::File::open(&self.path).expect("Failed to open file");
                let reader = std::io::BufReader::new(file);
                let reader = flate2::read::GzDecoder::new(reader);
                value = bincode::deserialize_from(reader).expect("Failed to deserialize");
            } else {
                value = default();
            }
            self.value = Some(value);
        }
        self.last_touch = std::time::Instant::now();
    }
    fn save_if_needed(&mut self) {
        self.last_save = std::time::Instant::now();
        if !self.mutated {
            return;
        }
        debug!("Saving");
        let value = self.value.as_mut().expect("Mutated but not loaded wtf?");
        let file = std::fs::File::create(&self.path).expect("Failed to create file");
        let writer = std::io::BufWriter::new(file);
        let writer = flate2::write::GzEncoder::new(writer, flate2::Compression::best());
        bincode::serialize_into(writer, value).expect("Failed to serialize");
        self.mutated = false;
    }
    fn periodic_check(&mut self) {
        if self.last_save.elapsed() > std::time::Duration::from_secs(10) {
            self.save_if_needed();
        }
        if self.last_touch.elapsed() > std::time::Duration::from_secs(10) {
            self.save_if_needed();
            self.value = None;
        }
    }
}

fn thread<T: Default + Serialize + DeserializeOwned>(state: Arc<Mutex<State<T>>>) {
    loop {
        std::thread::park_timeout(std::time::Duration::from_secs(1));
        let mut state = state.lock().unwrap();
        state.periodic_check();
        if state.dropped {
            return;
        }
    }
}

pub struct ReadGuard<'a, T> {
    guard: MutexGuard<'a, State<T>>,
}

impl<'a, T> Drop for ReadGuard<'a, T> {
    fn drop(&mut self) {}
}

impl<'a, T> Deref for ReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.guard.value.as_ref().unwrap()
    }
}

pub struct WriteGuard<'a, T> {
    guard: MutexGuard<'a, State<T>>,
}

impl<'a, T> Drop for WriteGuard<'a, T> {
    fn drop(&mut self) {}
}

impl<'a, T> Deref for WriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.guard.value.as_ref().unwrap()
    }
}

impl<'a, T> DerefMut for WriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard.value.as_mut().unwrap()
    }
}

impl<T: Default + Serialize + DeserializeOwned + Send + 'static> AutoSaved<T> {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        let state = Arc::new(Mutex::new(State::new(path.as_ref().to_owned())));
        Self {
            thread: Some(std::thread::spawn({
                let state = state.clone();
                move || thread(state)
            })),
            state,
        }
    }
    fn lock(&self) -> MutexGuard<State<T>> {
        let mut guard = self.state.lock().unwrap();
        self.thread.as_ref().unwrap().thread().unpark();
        guard.touch();
        guard
    }
    pub fn read(&self) -> ReadGuard<T> {
        ReadGuard { guard: self.lock() }
    }
    pub fn write(&self) -> WriteGuard<T> {
        let mut guard = self.lock();
        guard.mutated = true;
        WriteGuard { guard }
    }
}
