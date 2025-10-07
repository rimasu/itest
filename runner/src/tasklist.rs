use std::collections::{HashSet, VecDeque};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Status {
    Waiting,
    Ready,
    Running,
    Finished,
    Skipped,
    Failed,
}

impl Status {
    pub fn not_started(&self) -> bool {
        match self {
            Status::Waiting => true,
            Status::Ready => true,
            Status::Running => false,
            Status::Finished => false,
            Status::Skipped => false,
            Status::Failed => false,
        }
    }
}

#[derive(Clone)]
struct Task {
    status: Status,
    unblocks: Vec<usize>,
    blocked_by: HashSet<usize>,
}

#[derive(Clone)]
pub struct TaskList {
    ready: VecDeque<usize>,
    tasks: Vec<Task>,
}

impl TaskList {
    pub fn new(deps: &[Vec<usize>]) -> Self {
        let mut tasks = Vec::with_capacity(deps.len());
        let mut ready = VecDeque::new();
        for (task_id, deps) in deps.iter().enumerate() {
            let status = if deps.is_empty() {
                Status::Ready
            } else {
                Status::Waiting
            };

            tasks.push(Task {
                status: Status::Ready,
                unblocks: Vec::new(),
                blocked_by: deps.iter().map(|t| *t).collect(),
            });

            if status == Status::Ready {
                ready.push_back(task_id)
            }
        }

        for (task_id, deps) in deps.iter().enumerate() {
            for dep in deps {
                tasks[*dep].unblocks.push(task_id);
            }
        }

        Self { ready, tasks }
    }

    pub fn set_status(&mut self, id: usize, next: Status) {
        let current = self.tasks[id].status;
        match (current, next) {
            (Status::Ready, Status::Running) => self.start_task(id),
            (Status::Running, Status::Finished) => self.finish_task(id),
            (Status::Running, Status::Failed) => self.fail_task(id),
            _ => panic!(
                "Invalid status change for task {} ({:?} -> {:?})",
                id, current, next
            ),
        }
    }

    fn start_task(&mut self, id: usize) {
        self.tasks[id].status = Status::Running;
    }

    fn finish_task(&mut self, finished_task_id: usize) {
        while let Some(unblocked_id) = self.tasks[finished_task_id].unblocks.pop() {
            let blocked = &mut self.tasks[unblocked_id];
            assert!(blocked.blocked_by.remove(&finished_task_id));
            if blocked.blocked_by.is_empty() {
                blocked.status = Status::Ready;
                self.ready.push_back(unblocked_id);
            }
        }
        self.tasks[finished_task_id].status = Status::Finished;
    }

    fn fail_task(&mut self, failed_task_id: usize) {
        self.tasks[failed_task_id].status = Status::Failed;
        self.ready.clear();
        for task in &mut self.tasks {
            if task.status.not_started() {
                task.status = Status::Skipped
            }
        }
    }

    pub fn pop_ready(&mut self) -> Option<Vec<usize>> {
        // could be simpler.
        let mut ready = Vec::new();
        while let Some(idx) = self.ready.pop_front() {
            ready.push(idx)
        }
        if ready.is_empty() { None } else { Some(ready) }
    }

    pub fn all_finished(&self) -> bool {
        self.tasks.iter().all(|t| t.status == Status::Finished)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tasks_with_no_deps_are_ready() {
        let mut tasks = TaskList::new(&[vec![], vec![0], vec![]]);
        assert_eq!(Some(vec![0, 2]), tasks.pop_ready());
        assert_eq!(None, tasks.pop_ready());
    }

    #[test]
    fn tasks_become_ready_when_there_dependencies_are_finished() {
        let mut tasks = TaskList::new(&[vec![], vec![0], vec![]]);
        assert_eq!(Some(vec![0, 2]), tasks.pop_ready());
        assert_eq!(None, tasks.pop_ready());

        tasks.set_status(0, Status::Running);
        assert_eq!(None, tasks.pop_ready());

        tasks.set_status(0, Status::Finished);
        assert_eq!(Some(vec![1]), tasks.pop_ready());
        assert_eq!(None, tasks.pop_ready());
    }

    #[test]
    fn check_all_finished() {
        let mut tasks = TaskList::new(&[vec![], vec![0], vec![]]);
        assert_eq!(false, tasks.all_finished());

        tasks.set_status(0, Status::Running);
  
        tasks.set_status(1, Status::Finished);
        tasks.set_status(2, Status::Finished);
        assert_eq!(false, tasks.all_finished());
    }
}
