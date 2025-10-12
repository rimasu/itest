use crate::tasklist::Task;
use std::{collections::HashMap, fmt, time::Duration};
use tokio::{sync::mpsc, task::JoinHandle};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Phase {
    SetUp,
    TearDown,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Phase::SetUp => "set up",
            Phase::TearDown => "tear down",
        };
        fmt::Display::fmt(s, f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum TaskStatus {
    Pending,
    Running,
    Failed { duration: Duration, err_msg: String },
    Done { duration: Duration },
    Skipped,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ProgressEvent {
    PhaseStarted {
        phase: Phase,
    },
    PhaseFinished {
        phase: Phase,
        duration: Duration,
    },
    UpdateTask {
        phase: Phase,
        task: Task,
        status: TaskStatus,
    },
}

#[derive(Clone)]
pub struct ProgressListener {
    tx: mpsc::Sender<ProgressEvent>,
}

impl ProgressListener {
    pub async fn phase_started(&self, phase: Phase) {
        self.publish(ProgressEvent::PhaseStarted { phase }).await;
    }

    pub async fn phase_finished(&self, phase: Phase, duration: Duration) {
        self.publish(ProgressEvent::PhaseFinished { phase, duration })
            .await;
    }

    pub async fn task_ready(&self, phase: Phase, task: Task) {
        self.publish(ProgressEvent::UpdateTask {
            phase,
            task,
            status: TaskStatus::Pending,
        })
        .await
    }

    pub async fn task_running(&self, phase: Phase, task: Task) {
        self.publish(ProgressEvent::UpdateTask {
            phase,
            task,
            status: TaskStatus::Running,
        })
        .await
    }

    pub async fn task_done(&self, phase: Phase, task: Task, duration: Duration) {
        self.publish(ProgressEvent::UpdateTask {
            phase,
            task,
            status: TaskStatus::Done { duration },
        })
        .await
    }

    pub async fn task_failed(&self, phase: Phase, task: Task, duration: Duration, err_msg: String) {
        self.publish(ProgressEvent::UpdateTask {
            phase,
            task,
            status: TaskStatus::Failed { duration, err_msg },
        })
        .await
    }

    pub async fn task_skipped(&self, phase: Phase, task: Task) {
        self.publish(ProgressEvent::UpdateTask {
            phase,
            task,
            status: TaskStatus::Skipped,
        })
        .await
    }

    async fn publish(&self, ev: ProgressEvent) {
        if let Some(err) = self.tx.send(ev).await.err() {
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
            ProgressEvent::PhaseStarted { phase } => {
                println!("{phase:width$}  started", width = self.max_name_len + 1);
            }
            ProgressEvent::PhaseFinished { phase, duration } => {
                println!(
                    "{phase:width$}  done     {:8.02}s",
                    duration.as_millis() as f64 / 1000.0,
                    width = self.max_name_len + 1
                );
            }
            ProgressEvent::UpdateTask {
                phase: _,
                task,
                status,
            } => {
                let name = self.task_name(task);
                match status {
                    TaskStatus::Pending => {
                        println!(" {name}  pending");
                    }
                    TaskStatus::Running => {
                        println!(" {name}  running  ");
                    }
                    TaskStatus::Failed { duration, err_msg } => {
                        println!(
                            " {name}  failed   {:8.02}s:\n\t{err_msg}",
                            duration.as_millis() as f64 / 1000.0
                        );
                    }
                    TaskStatus::Done { duration } => {
                        println!(
                            " {name}  done     {:8.02}s",
                            duration.as_millis() as f64 / 1000.0
                        );
                    }
                    TaskStatus::Skipped => {
                        println!(" {name}  skipped");
                    }
                }
            }
        }
    }
}
