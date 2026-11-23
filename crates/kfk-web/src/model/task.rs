//! one task, drilled into
use super::{card, Card};
use anyhow::Result;
use kfk_store::{Run, Store, Task};
use kfk_util::tail_lines;

pub struct TaskView {
    pub card: Card,
    pub parent: Option<Task>,
    pub kids: Vec<Card>,
    pub runs: Vec<Run>,
    /// (time, actor, kind, text), oldest first
    pub events: Vec<(f64, String, String, String)>,
    /// the tail of the last check output, while the task is not done
    pub check_tail: String,
    pub last_words: String,
}

const KINDS: [&str; 10] = ["add", "note", "done", "failed", "blocked", "cancelled", "todo", "requeued", "exec-settled", "running"];

pub fn task_view(s: &Store, id: i64) -> Result<TaskView> {
    let task = s.task(id)?;
    let parent = task.parent.and_then(|p| s.task(p).ok());
    let kids = s.children(id)?.into_iter().map(|k| card(s, k)).collect::<Result<Vec<_>>>()?;
    let runs = s.runs_of(id)?;
    let tail = |file: &str| match runs.last().filter(|_| task.status != "done") {
        Some(r) => tail_lines(std::fs::read_to_string(format!("{}/{file}", r.dir)).unwrap_or_default().trim(), 40, 4000),
        None => String::new(),
    };
    Ok(TaskView { check_tail: tail("check.log"), last_words: tail("result.md"), events: s.events(Some(id), &KINDS, 200)?, card: card(s, task)?, parent, kids, runs })
}
