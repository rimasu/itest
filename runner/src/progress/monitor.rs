use crate::{
    progress::{styles::Styles, OverallSummary, Phase, PhaseSummary, TaskStatus},
    tasklist::Task,
};

use anstream::Stdout;
use std::io::{self, Write};
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
                    worker.log_event(ev).unwrap();
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
    styles: Styles,
    max_name_len: usize,
}



impl MonitorWorker {
    fn new(task_names: HashMap<Task, String>) -> Self {
        let max_name_len = task_names.values().map(|n| n.len()).max().unwrap_or(0);
        let stdout = anstream::stdout();
        let styles = Styles::default();
        Self {
            task_names,
            stdout,
            styles,
            max_name_len,
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

    fn log_event(&mut self, event: ProgressEvent) -> Result<(), io::Error> {
        match event {
            ProgressEvent::PhaseStarted { phase, num_tasks } => {
                writeln!(&mut self.stdout, "\nrunning {num_tasks} {phase} tasks")
            }
            ProgressEvent::PhaseFinished { summary } => self.log_phase_finished(summary),
            ProgressEvent::UpdateTask {
                phase: _,
                task,
                status,
                duration,
                err_msg,
            } => self.log_update_task(task, status, duration, err_msg),

            ProgressEvent::FinalStatus { summary } => self.log_final_status(summary),
            ProgressEvent::Shutdown => panic!("Should not be logging shutdown event"),
        }
    }

    fn log_update_task(
        &mut self,
        task: Task,
        status: TaskStatus,
        duration: Option<Duration>,
        err_msg: Option<String>,
    ) -> Result<(), io::Error> {
        let name = self.task_name(task);
        let status_style = self.styles.task_status(status);
        let bold = self.styles.bold;

        write!(
            &mut self.stdout,
            " {}{}{}  {}{status:10}{}",
            bold.render(),
            name,
            bold.render_reset(),
            status_style.render(),
            status_style.render_reset()
        )?;

        if let Some(duration) = duration {
            writeln!(
                &mut self.stdout,
                "{:8.02}s",
                duration.as_millis() as f64 / 1000.0
            )?;
        } else {
            writeln!(&mut self.stdout)?;
        }

        if let Some(err_msg) = err_msg {
            writeln!(
                &mut self.stdout,
                "\n\t{}{err_msg}{}",
                self.styles.bad.render(),
                self.styles.bad.render_reset()
            )?;
        }

        Ok(())
    }

    fn log_phase_finished(&mut self, summary: PhaseSummary) -> Result<(), io::Error> {
        write!(&mut self.stdout, "\n{} ", summary.phase,)?;

        self.log_phase_details(summary)
    }

    fn log_phase_details(&mut self, summary: PhaseSummary) -> Result<(), io::Error> {
        let result_style = self.styles.phase_result(summary.result);

        write!(
            &mut self.stdout,
            "result: {}{}{}. ",
            result_style.render(),
            summary.result,
            result_style.render_reset()
        )?;

        for status in &[TaskStatus::Ok, TaskStatus::Failed, TaskStatus::Skipped] {
            if let Some(count) = summary.counts.get(status) {
                write!(&mut self.stdout, "{} {}; ", *count, status)?;
            }
        }

        writeln!(
            &mut self.stdout,
            "finished in {:.02}s",
            summary.duration.as_millis() as f64 / 1000.0
        )
    }

    fn log_final_status(&mut self, summary: OverallSummary) -> Result<(), io::Error> {
        let result_style = self.styles.overall_result(summary.result);

        let width = summary
            .phases
            .iter()
            .map(|p| p.phase.to_string().len())
            .max()
            .unwrap_or(0);

        writeln!(&mut self.stdout, "\nsummary\n",)?;

        for summary in summary.phases {
            write!(
                &mut self.stdout,
                " {:>width$} ",
                summary.phase,
                width = width
            )?;
            self.log_phase_details(summary)?;
        }

        writeln!(
            &mut self.stdout,
            "\noverall result: {}{}{}. finished in {:.02}s\n",
            result_style.render(),
            summary.result,
            result_style.render_reset(),
            summary.duration.as_millis() as f64 / 1000.0
        )
    }
}
