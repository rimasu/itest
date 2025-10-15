use crate::tasklist::Task;
use std::{
    collections::HashMap,
    fmt,
    time::{Duration, Instant},
};
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TaskStatus {
    Running,
    Failed,
    Ok,
    Skipped,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TaskStatus::Running => "running",
            TaskStatus::Failed => "failed",
            TaskStatus::Ok => "ok",
            TaskStatus::Skipped => "skipped",
        };
        fmt::Display::fmt(s, f)
    }
}

pub struct SummaryBuilder {
    start: Instant,
    phases: Vec<(Phase, PhaseSummary)>,
}

impl SummaryBuilder {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            phases: Vec::new(),
        }
    }

    pub fn add_phase(&mut self, phase: Phase, summary: PhaseSummary) {
        self.phases.push((phase, summary));
    }

    pub fn build(self) -> Summary {
        Summary {
            duration: self.start.elapsed(),
            phases: self.phases,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Summary {
    duration: Duration,
    phases: Vec<(Phase, PhaseSummary)>,
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (phase, summary) in &self.phases {
            let result = if summary.all_ok() { "ok" } else { "failed" };
            writeln!(f, "{phase} result: {result}. {}", summary,)?;
        }
        writeln!(
            f,
            "finished in {:.02}s",
            self.duration.as_millis() as f64 / 1000.0
        )
    }
}

pub struct PhaseSummaryBuilder {
    start: Instant,
    counts: HashMap<TaskStatus, usize>,
}

impl PhaseSummaryBuilder {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            counts: HashMap::new(),
        }
    }

    pub fn inc(&mut self, status: TaskStatus) {
        *(self.counts.entry(status).or_default()) += 1;
    }

    pub fn build(self) -> PhaseSummary {
        PhaseSummary {
            duration: self.start.elapsed(),
            counts: self.counts,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PhaseSummary {
    duration: Duration,
    counts: HashMap<TaskStatus, usize>,
}

impl fmt::Display for PhaseSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for status in &[TaskStatus::Ok, TaskStatus::Failed, TaskStatus::Skipped] {
            if let Some(count) = self.counts.get(status) {
                write!(f, "{} {}; ", *count, status)?;
            }
        }
        write!(
            f,
            "finished in {:.02}s",
            self.duration.as_millis() as f64 / 1000.0
        )
    }
}

impl PhaseSummary {
    pub fn all_ok(&self) -> bool {
        let total: usize = self.counts.values().sum();
        let okay = *self.counts.get(&TaskStatus::Ok).unwrap_or(&0);
        okay == total
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ProgressEvent {
    PhaseStarted {
        phase: Phase,
        num_tasks: usize,
    },
    PhaseFinished {
        phase: Phase,
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
        summary: Summary,
    },
    Shutdown,
}

#[derive(Clone)]
pub struct ProgressListener {
    tx: mpsc::Sender<ProgressEvent>,
}

pub struct ProgressMonitor {
    listener: ProgressListener,
    handle: JoinHandle<()>,
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

    pub async fn phase_finished(&self, phase: Phase, summary: PhaseSummary) {
        self.publish(ProgressEvent::PhaseFinished { phase, summary })
            .await;
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

    pub async fn finished(&self, summary: Summary) {
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
            ProgressEvent::PhaseFinished { phase, summary } => {
                let result = if summary.all_ok() { "ok" } else { "failed" };
                println!("\n{phase} result: {result}. {}", summary,);
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
                println!("\nSummary\n{}", summary)
            }
            ProgressEvent::Shutdown => panic!("Should not be logging shutdown event"),
        }
    }
}
