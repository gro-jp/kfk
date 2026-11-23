mod common;
use common::*;

#[test]
fn pass_escalate_and_fail_in_a_worktree() {
    let _g = ENV.lock().unwrap();
    let s = Store::temp();
    std::env::set_var("KFK_CLAUDE", fake(&s.kh, GOOD));
    let (r, v) = (repo(&s), vault(&s));
    let easy = task(&s, "easy", "write good", "grep -q good out.txt", &r);
    s.claim(easy.id).unwrap();
    run_task(&s, &v, easy.id).unwrap();
    let t = s.task(easy.id).unwrap();
    assert_eq!((t.status.as_str(), t.attempts), ("done", 1));
    assert!(t.result.contains("branch kfk/T1") && t.result.contains("sonnet/medium") && t.result.contains("slot login"), "{}", t.result);
    assert!(git(&r, &["rev-parse", "kfk/T1"]).0);
    let run = &s.runs_of(easy.id).unwrap()[0];
    assert_eq!((run.weighted(), run.turns, run.status.as_str()), (485, 2, "pass"));
    assert_eq!(s.account(LOGIN).unwrap().unwrap().wins["seven_day"].util, Some(20.0));

    let flaky = task(&s, "flaky", "FLAKY", "grep -q good out.txt", &r);
    s.claim(flaky.id).unwrap();
    run_task(&s, &v, flaky.id).unwrap();
    let runs = s.runs_of(flaky.id).unwrap();
    assert_eq!(runs.iter().map(|r| (r.model.as_str(), r.status.as_str())).collect::<Vec<_>>(), [("sonnet", "fail"), ("opus", "pass")]);
    assert!(fs::read_to_string(format!("{}/prompt.md", runs[1].dir)).unwrap().contains("previous attempt"));
    assert_eq!(s.task(flaky.id).unwrap().status, "done");

    let never = task(&s, "never", "x", "echo nope; false", &r);
    s.set_field(never.id, "max_attempts", "2").unwrap();
    s.claim(never.id).unwrap();
    run_task(&s, &v, never.id).unwrap();
    let t = s.task(never.id).unwrap();
    assert_eq!((t.status.as_str(), t.attempts, t.result.as_str()), ("failed", 2, "check still failing after 2 attempts"));
    run_task(&s, &v, never.id).unwrap(); // not running any more: a no-op
    assert_eq!(s.runs_of(never.id).unwrap().len(), 2);
    std::env::remove_var("KFK_CLAUDE");
}
