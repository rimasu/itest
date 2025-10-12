use crate::tasklist::Task;
use std::{collections::HashMap, time::Duration};
use tokio::{sync::mpsc, task::JoinHandle};

#[derive(Debug, Clone, Eq, PartialEq)]
enum ProgressEvent {
    SetUpsStarted,
    SetUpStarted {
        task: Task,
    },
    SetUpReady {
        task: Task,
    },
    SetUpFinished {
        task: Task,
        duration: Duration,
    },
    SetUpFailed {
        task: Task,
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
    },
    TearDownFinished {
        task: Task,
        duration: Duration,
    },
    TearDownFailed {
        task: Task,
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

    pub async fn set_up_ready(&self, task: Task) {
        self.publish(ProgressEvent::SetUpReady { task }).await
    }

    pub async fn set_up_started(&self, task: Task) {
        self.publish(ProgressEvent::SetUpStarted { task }).await
    }

    pub async fn set_up_finished(&self, task: Task, duration: Duration) {
        self.publish(ProgressEvent::SetUpFinished { task, duration })
            .await
    }

    pub async fn set_up_failed(&self, task: Task, duration: Duration, message: &str) {
        self.publish(ProgressEvent::SetUpFailed {
            task,
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

    pub async fn tear_down_started(&self, task: Task) {
        self.publish(ProgressEvent::TearDownStarted { task }).await
    }

    pub async fn tear_down_finished(&self, task: Task, duration: Duration) {
        self.publish(ProgressEvent::TearDownFinished { task, duration })
            .await
    }

    pub async fn tear_down_failed(&self, task: Task, duration: Duration, message: &str) {
        self.publish(ProgressEvent::TearDownFailed {
            task,
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
        if let Some(err)  =self.tx.send(ev).await.err() {
            println!("Failed to publish progress event {:?}", err.0);
        }
    }
}

pub fn launch_progress_monitor(
    task_names: HashMap<Task, String>,
) -> (JoinHandle<()>, ProgressListener) {
    let max_name_len = task_names.values().map(|n| n.len()).max().unwrap_or(0);
    let monitor = Monitor {
        task_names,
        max_name_len,
    };

    let (tx, mut rx) = mpsc::channel(100);
    let handle = tokio::spawn(async move {
        while let Some(ev) = rx.recv().await {
            monitor.log_event(ev);
        }
    });
    (handle, ProgressListener { tx })
}

struct Monitor {
    task_names: HashMap<Task, String>,
    max_name_len: usize,
}

impl Monitor {
    fn task_name(&self, task: Task) -> String {
        let raw = self
            .task_names
            .get(&task)
            .map(|n| n.as_str())
            .unwrap_or("?");

        format!("{:width$}", raw, width = self.max_name_len)
    }

    fn log_event(&self, event: ProgressEvent) {
        match event {
            ProgressEvent::SetUpsStarted => {
                println!("set up started");
            }
            ProgressEvent::SetUpStarted { task } => {
                let name = self.task_name(task);
                println!("  {name} started");
            }
            ProgressEvent::SetUpReady { task } => {
                let name = self.task_name(task);
                println!("  {name} ready");
            }
            ProgressEvent::SetUpFinished { task, duration } => {
                let name = self.task_name(task);
                println!("  {name} finished in {:?}", duration);
            }
            ProgressEvent::SetUpFailed {
                task,
                duration,
                message,
            } => {
                let name = self.task_name(task);
                println!("  {name} failed in {:?} {}", duration, message);
            }
            ProgressEvent::SetUpsFinished { success, duration } => {
                println!("set up finished in {:?} (success={})", duration, success);
            }

            ProgressEvent::TearDownsStarted => {
                println!("tear down started");
            }
            ProgressEvent::TearDownStarted { task } => {
                let name = self.task_name(task);
                println!("tear down {name} started");
            }
            ProgressEvent::TearDownFinished { task, duration } => {
                let name = self.task_name(task);
                println!("tear down {name} finished in {:?}", duration);
            }
            ProgressEvent::TearDownFailed {
                task,
                duration,
                message,
            } => {
                let name = self.task_name(task);
                println!("tear down {name} failed in {:?} {}", duration, message);
            }
            ProgressEvent::TearDownsFinished { success, duration } => {
                println!("tear down finished in {:?} (success={})", duration, success);
            }
        }
    }
}
