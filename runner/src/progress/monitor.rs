use crate::{
    progress::{OverallSummary, Phase, PhaseSummary, TaskStatus},
    tasklist::Task,
};

use anstream::Stdout;
use anstyle::{AnsiColor, Color, Style};
use std::io::Write;
use std::{collections::HashMap, time::Duration};
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
        let mut worker = MonitorWorker::new(task_names);
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
    stdout: Stdout,
    bold_style: Style,
    good_style: Style,
    bad_style: Style,
    norm_style: Style,
    max_name_len: usize,
}

impl MonitorWorker {
    fn new(task_names: HashMap<Task, String>) -> Self {
        let max_name_len = task_names.values().map(|n| n.len()).max().unwrap_or(0);
        let stdout = anstream::stdout();
        let bold_style = Style::new().bold();
        let bad_style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
        let good_style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
        let norm_style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::White)));
        Self {
            task_names,
            stdout,
            bold_style,
            good_style,
            bad_style,
            norm_style,
            max_name_len,
        }
    }

    fn task_style(&self, status: TaskStatus) -> &Style {
        match status {
            TaskStatus::Running => &self.norm_style,
            TaskStatus::Failed => &self.bad_style,
            TaskStatus::Ok => &self.good_style,
            TaskStatus::Skipped => &self.norm_style,
        }
    }

    fn task_name(&self, task: Task) -> String {
        let raw = self
            .task_names
            .get(&task)
            .map(|n| n.as_str())
            .unwrap_or("?");

        format!("{:width$}", raw, width = self.max_name_len)
    }

    fn log_event(&mut self, event: ProgressEvent) {
        match event {
            ProgressEvent::PhaseStarted { phase, num_tasks } => {
                writeln!(&mut self.stdout, "running {num_tasks} {phase} tasks");
            }
            ProgressEvent::PhaseFinished { summary } => {
                let red = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
                writeln!(
                    &mut self.stdout,
                    "\n{} {} {} {}",
                    red.render(),
                    summary.phase,
                    summary,
                    red.render_reset()
                );
            }
            ProgressEvent::UpdateTask {
                phase: _,
                task,
                status,
                duration,
                err_msg,
            } => {
                let name = self.task_name(task);
                let status_style = self.task_style(status);
                print!(
                    " {}{}{}  {}{status:10}{}",
                    self.bold_style.render(),
                    name,
                    self.bold_style.render_reset(),
                    status_style.render(),
                    status_style.render_reset()
                );
                if let Some(duration) = duration {
                    writeln!(
                        &mut self.stdout,
                        "{:8.02}s",
                        duration.as_millis() as f64 / 1000.0
                    );
                } else {
                    writeln!(&mut self.stdout);
                }
                if let Some(err_msg) = err_msg {
                    writeln!(&mut self.stdout, "\t{err_msg}");
                }
            }
            ProgressEvent::FinalStatus { summary } => {
                writeln!(&mut self.stdout, "\n{}", summary);
            }
            ProgressEvent::Shutdown => panic!("Should not be logging shutdown event"),
        }
    }
}
