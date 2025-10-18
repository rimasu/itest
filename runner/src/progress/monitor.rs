use crate::{progress::{Phase, PhaseSummary, OverallSummary, TaskStatus}, tasklist::Task};

use std::{
    collections::HashMap,
    time::{Duration},
};

use tokio::{sync::mpsc, task::JoinHandle};


/// Responsible for creating `listeners` and handling shutdown.
pub struct ProgressMonitor {
    listener: ProgressListener,
    handle: JoinHandle<()>,
}

#[derive(Clone)]
pub struct ProgressListener {
    tx: mpsc::Sender<ProgressEvent>,
}


/// Events passed from listnener to worker.
#[derive(Debug, Clone, Eq, PartialEq)]
enum ProgressEvent {
    PhaseStarted {
        phase: Phase,
        num_tasks: usize,
    },
    PhaseFinished {
        summary: PhaseSummary,
    },
    UpdateTask {
        phase: Phase,
        task: Task,
        status: TaskStatus,
        duration: Option<Duration>,
        err_msg: Option<String>,
    },
    FinalStatus {
        summary: OverallSummary,
    },
    Shutdown,
}


impl ProgressMonitor {
    pub fn new(task_names: HashMap<Task, String>) -> Self {
        let max_name_len = task_names.values().map(|n| n.len()).max().unwrap_or(0);
        let worker = MonitorWorker {
            task_names,
            max_name_len,
        };

        let (tx, mut rx) = mpsc::channel(100);
        let handle = tokio::spawn(async move {
            while let Some(ev) = rx.recv().await {
                if ev == ProgressEvent::Shutdown {
                    break;
                } else {
                    worker.log_event(ev);
                }
            }
        });

        let listener = ProgressListener { tx };
        let monitor = ProgressMonitor {
            handle,
            listener: listener.clone(),
        };

        monitor
    }

    pub fn listener(&self) -> ProgressListener {
        self.listener.clone()
    }

    pub async fn shutdown(self) {
        self.listener.publish(ProgressEvent::Shutdown).await;
        self.handle.await;
    }
}

impl ProgressListener {
    pub async fn phase_started(&self, phase: Phase, num_tasks: usize) {
        self.publish(ProgressEvent::PhaseStarted { phase, num_tasks })
            .await;
    }

    pub async fn phase_finished(&self, summary: PhaseSummary) {
        self.publish(ProgressEvent::PhaseFinished { summary }).await;
    }

    pub async fn task_running(&self, phase: Phase, task: Task) {
        self.publish(ProgressEvent::UpdateTask {
            phase,
            task,
            status: TaskStatus::Running,
            duration: None,
            err_msg: None,
        })
        .await
    }

    pub async fn task_done(&self, phase: Phase, task: Task, duration: Duration) {
        self.publish(ProgressEvent::UpdateTask {
            phase,
            task,
            status: TaskStatus::Ok,
            duration: Some(duration),
            err_msg: None,
        })
        .await
    }

    pub async fn task_failed(&self, phase: Phase, task: Task, duration: Duration, err_msg: String) {
        self.publish(ProgressEvent::UpdateTask {
            phase,
            task,
            status: TaskStatus::Failed,
            duration: Some(duration),
            err_msg: Some(err_msg),
        })
        .await
    }

    pub async fn task_skipped(&self, phase: Phase, task: Task) {
        self.publish(ProgressEvent::UpdateTask {
            phase,
            task,
            status: TaskStatus::Skipped,
            duration: None,
            err_msg: None,
        })
        .await
    }

    pub async fn finished(&self, summary: OverallSummary) {
        self.publish(ProgressEvent::FinalStatus { summary }).await
    }

    async fn publish(&self, ev: ProgressEvent) {
        if let Some(err) = self.tx.send(ev).await.err() {
            println!("Failed to publish progress event {:?}", err.0);
        }
    }
}

struct MonitorWorker {
    task_names: HashMap<Task, String>,
    max_name_len: usize,
}

impl MonitorWorker {
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
            ProgressEvent::PhaseStarted { phase, num_tasks } => {
                println!("running {num_tasks} {phase} tasks");
            }
            ProgressEvent::PhaseFinished { summary } => {
                println!("\n{} {}", summary.phase, summary,);
            }
            ProgressEvent::UpdateTask {
                phase: _,
                task,
                status,
                duration,
                err_msg,
            } => {
                let name = self.task_name(task);
                print!(" {name}  {status:10}");
                if let Some(duration) = duration {
                    println!("{:8.02}s", duration.as_millis() as f64 / 1000.0);
                } else {
                    println!();
                }
                if let Some(err_msg) = err_msg {
                    println!("\t{err_msg}")
                }
            }
            ProgressEvent::FinalStatus { summary } => {
                println!("\n{}", summary)
            }
            ProgressEvent::Shutdown => panic!("Should not be logging shutdown event"),
        }
    }
}
