use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use actix_web::rt::{spawn, task::JoinHandle, time::sleep};

use crate::utsjekk;

#[derive(Debug)]
pub struct JobState {
    pub state: Mutex<State>,
    pub sleep_ms: Mutex<u64>,
}

impl Default for JobState {
    fn default() -> Self {
        JobState {
            state: Mutex::new(State::Stopped),
            sleep_ms: Mutex::new(1_000),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum State {
    #[default]
    Stopped,
    Started,
}

pub fn init_job() -> (Arc<JobState>, JoinHandle<()>) {
    let state = Arc::new(JobState::default());
    let handle = spawn(background_job(state.clone()));
    (state.clone(), handle)
}

async fn background_job(job_state: Arc<JobState>) {
    loop {
        let state = *job_state.state.lock().unwrap();
        if let State::Started = state {
            utsjekk::iverksett().await;
        }

        let sleep_ms = *job_state.sleep_ms.lock().unwrap();
        sleep(Duration::from_millis(sleep_ms)).await;
    }
}
