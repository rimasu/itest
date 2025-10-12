use std::time::Duration;
use tokio::{sync::mpsc, task::JoinHandle};
use crate::tasklist::Task;

#[derive(Debug, Clone, Eq, PartialEq)]
enum ProgressEvent {
    SetUpsStarted,
    SetUpStarted {
        task: Task,
        name: String,
    },
    SetUpFinished {
        task: Task,
        name: String,
        duration: Duration,
    },
    SetUpFailed {
        task: Task,
        name: String,
        duration: Duration,
        message: String,
    },
    SetUpsFinished {
        success: bool,
        duration: Duration,
    },
    TearDownsStarted,
    TearDownStarted {
        task: Task,
        name: String,
    },
    TearDownFinished {
        task: Task,
        name: String,
        duration: Duration,
    },
    TearDownFailed {
        task: Task,
        name: String,
        duration: Duration,
        message: String,
    },
    TearDownsFinished {
        success: bool,
        duration: Duration,
    },
}

#[derive(Clone)]
pub struct ProgressListener {
    tx: mpsc::Sender<ProgressEvent>,
}

impl ProgressListener {
    pub async fn set_ups_started(&self) {
        self.publish(ProgressEvent::SetUpsStarted).await
    }

    pub async fn set_up_started(&self, task: Task, name: &str) {
        self.publish(ProgressEvent::SetUpStarted {
            task,
            name: name.to_owned(),
        })
        .await
    }

    pub async fn set_up_finished(&self, task: Task, name: &str, duration: Duration) {
        self.publish(ProgressEvent::SetUpFinished {
            task,
            name: name.to_owned(),
            duration,
        })
        .await
    }

    pub async fn set_up_failed(&self, task: Task, name: &str, duration: Duration, message: &str) {
        self.publish(ProgressEvent::SetUpFailed {
            task,
            name: name.to_owned(),
            duration,
            message: message.to_owned(),
        })
        .await
    }

    pub async fn set_ups_finished(&self, success: bool, duration: Duration) {
        self.publish(ProgressEvent::SetUpsFinished { success, duration })
            .await
    }

    pub async fn tear_downs_started(&self) {
        self.publish(ProgressEvent::TearDownsStarted).await
    }

    pub async fn tear_down_started(&self, task: Task, name: &str) {
        self.publish(ProgressEvent::TearDownStarted {
            task,
            name: name.to_owned(),
        })
        .await
    }

    pub async fn tear_down_finished(&self, task: Task, name: &str, duration: Duration) {
        self.publish(ProgressEvent::TearDownFinished {
            task,
            name: name.to_owned(),
            duration,
        })
        .await
    }

    pub async fn tear_down_failed(
        &self,
        task: Task,
        name: &str,
        duration: Duration,
        message: &str,
    ) {
        self.publish(ProgressEvent::TearDownFailed {
            task,
            name: name.to_owned(),
            duration,
            message: message.to_owned(),
        })
        .await
    }

    pub async fn tear_downs_finished(&self, success: bool, duration: Duration) {
        self.publish(ProgressEvent::TearDownsFinished { success, duration })
            .await
    }

    async fn publish(&self, ev: ProgressEvent) {
        self.tx.send(ev).await.unwrap()
    }
}

pub fn launch_event_monitor() -> (JoinHandle<()>, ProgressListener) {
    let (tx, mut rx) = mpsc::channel(100);
    let handle = tokio::spawn(async move {
        while let Some(ev) = rx.recv().await {
            log_event(ev);
        }
    });
    (handle, ProgressListener { tx })
}

fn log_event(event: ProgressEvent) {
    match event {
        ProgressEvent::SetUpsStarted => {
            println!("set up started");
        }
        ProgressEvent::SetUpStarted { task, name } => {
            println!("set up {} started", name);
        }
        ProgressEvent::SetUpFinished {
            task: _,
            name,
            duration,
        } => {
            println!("set up {} finished in {:?}", name, duration);
        }
        ProgressEvent::SetUpFailed {
            task: _,
            name,
            duration,
            message,
        } => {
            println!("set up {} failed in {:?} {}", name, duration, message);
        }
        ProgressEvent::SetUpsFinished { success, duration } => {
            println!("set up finished in {:?} (success={})", duration, success);
        }

        ProgressEvent::TearDownsStarted => {
            println!("tear down started");
        }
        ProgressEvent::TearDownStarted { task, name } => {
            println!("tear down {} started", name);
        }
        ProgressEvent::TearDownFinished {
            task,
            name,
            duration,
        } => {
            println!("tear down {} finished in {:?}", name, duration);
        }
        ProgressEvent::TearDownFailed {
            task,
            name,
            duration,
            message,
        } => {
            println!("tear down {} failed in {:?} {}", name, duration, message);
        }
        ProgressEvent::TearDownsFinished { success, duration } => {
            println!("tear down finished in {:?} (success={})", duration, success);
        }
    }
}
