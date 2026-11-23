//! model and effort for an attempt: the task's own, else its class's ladder, one rung up per failed attempt
pub fn route(ladder: &str, attempt: i64, model: &str, effort: &str) -> (String, String) {
    let rungs: Vec<(&str, &str)> = ladder.split_whitespace().filter_map(|x| x.split_once(':')).collect();
    let (m, e) = if rungs.is_empty() { ("sonnet", "medium") } else { rungs[((attempt.max(1) - 1) as usize).min(rungs.len() - 1)] };
    ((if model.is_empty() { m } else { model }).to_string(), (if effort.is_empty() { e } else { effort }).to_string())
}
