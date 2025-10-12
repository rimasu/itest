use std::collections::{HashSet, VecDeque};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Status {
    Waiting,
    Success,
    Skipped,
    Failed,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Task(pub(crate) usize);

#[derive(Clone)]
struct TaskState {
    status: Status,
    unblocks: Vec<Task>,
    blocked_by: HashSet<Task>,
}

#[derive(Clone)]
pub struct TaskList {
    ready: VecDeque<Task>,
    tasks: Vec<TaskState>,
}

impl TaskList {
    pub fn new(deps: &[Vec<usize>]) -> Self {
        let mut tasks = Vec::with_capacity(deps.len());
        let mut ready = VecDeque::new();
        for (task_id, deps) in deps.iter().enumerate() {
            tasks.push(TaskState {
                status: Status::Waiting,
                unblocks: Vec::new(),
                blocked_by: deps.iter().map(|t| Task(*t)).collect(),
            });

            if deps.is_empty() {
                ready.push_back(Task(task_id))
            }
        }

        for (task_id, deps) in deps.iter().enumerate() {
            for dep in deps {
                tasks[*dep].unblocks.push(Task(task_id));
            }
        }

        Self { ready, tasks }
    }

    pub fn set_status(&mut self, task: Task, next: Status) {
        let current = self.tasks[task.0].status;
        match (current, next) {
            (Status::Waiting, Status::Success) => self.finish_task(task),
            (Status::Waiting, Status::Failed) => self.fail_task(task),
            _ => panic!(
                "Invalid status change for {:?} ({:?} -> {:?})",
                task, current, next
            ),
        }
    }

    fn finish_task(&mut self, finished: Task) {
        while let Some(unblocked_id) = self.tasks[finished.0].unblocks.pop() {
            let blocked = &mut self.tasks[unblocked_id.0];
            assert!(blocked.blocked_by.remove(&finished));
            if blocked.blocked_by.is_empty() {
                self.ready.push_back(unblocked_id);
            }
        }
        self.tasks[finished.0].status = Status::Success;
    }

    fn fail_task(&mut self, failed: Task) {
        self.tasks[failed.0].status = Status::Failed;
        self.ready.clear();
        for task in &mut self.tasks {
            if task.status == Status::Waiting {
                task.status = Status::Skipped;
            }
        }
    }

    pub fn pop_ready(&mut self) -> Option<Vec<Task>> {
        // could be simpler.
        let mut ready = Vec::new();
        while let Some(idx) = self.ready.pop_front() {
            ready.push(idx)
        }
        if ready.is_empty() { None } else { Some(ready) }
    }

    pub fn all_success(&self) -> bool {
        self.tasks.iter().all(|t| t.status == Status::Success)
    }

    pub fn none_waiting(&self) -> bool {
        !self.tasks.iter().any(|t| t.status == Status::Waiting)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tasks_with_no_deps_are_ready() {
        let mut tasks = TaskList::new(&[vec![], vec![0], vec![]]);
        assert_eq!(Some(vec![Task(0), Task(2)]), tasks.pop_ready());
        assert_eq!(None, tasks.pop_ready());
    }

    #[test]
    fn tasks_become_ready_when_there_dependencies_are_finished() {
        let mut tasks = TaskList::new(&[vec![], vec![0], vec![]]);
        assert_eq!(Some(vec![Task(0), Task(2)]), tasks.pop_ready());
        assert_eq!(None, tasks.pop_ready());

        tasks.set_status(Task(0), Status::Success);
        assert_eq!(Some(vec![Task(1)]), tasks.pop_ready());
        assert_eq!(None, tasks.pop_ready());
    }

    #[test]
    fn check_all_finished() {
        let mut tasks = TaskList::new(&[vec![], vec![0], vec![]]);
        assert_eq!(false, tasks.all_success());

        tasks.set_status(Task(0), Status::Success);
        tasks.set_status(Task(1), Status::Success);
        tasks.set_status(Task(2), Status::Success);
        assert_eq!(true, tasks.all_success());
    }
}
