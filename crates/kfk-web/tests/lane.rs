use kfk_store::Task;
use kfk_web::lane::*;

fn t(status: &str, parent: Option<i64>, result: &str) -> Task {
    Task { status: status.into(), parent, result: result.into(), ..Default::default() }
}

#[test]
fn a_stream_takes_the_most_urgent_lane() {
    let kid = |st: &str| t(st, Some(1), "");
    assert_eq!(lane(&t("todo", None, ""), &[]), Lane::Planned);
    assert_eq!(lane(&t("todo", None, ""), &[kid("cancelled")]), Lane::Planned);
    assert_eq!(lane(&t("todo", None, ""), &[kid("todo"), kid("done")]), Lane::Queued);
    assert_eq!(lane(&t("todo", Some(1), ""), &[]), Lane::Queued);
    assert_eq!(lane(&t("todo", None, ""), &[kid("todo"), kid("running")]), Lane::InFlight);
    assert_eq!(lane(&t("running", None, ""), &[]), Lane::InFlight);
    assert_eq!(lane(&t("todo", None, ""), &[kid("failed"), kid("running")]), Lane::InFlight);
    assert_eq!(lane(&t("todo", None, ""), &[kid("failed"), kid("todo")]), Lane::Stuck);
    assert_eq!(lane(&t("todo", None, ""), &[kid("blocked")]), Lane::Stuck);
    assert_eq!(lane(&t("failed", None, ""), &[kid("running")]), Lane::Stuck);
    assert_eq!(lane(&t("blocked", None, "no account has headroom"), &[]), Lane::Stuck);
    assert_eq!(lane(&t("blocked", None, "Needs owner: which region?"), &[]), Lane::NeedsYou);
    assert_eq!(lane(&t("todo", None, ""), &[kid("running"), t("blocked", Some(1), "needs owner: x")]), Lane::NeedsYou);
    assert_eq!(lane(&t("done", None, ""), &[kid("failed")]), Lane::Done);
    assert_eq!(lane(&t("cancelled", None, ""), &[kid("running")]), Lane::Dropped);
}

#[test]
fn every_lane_has_words_and_a_mark() {
    let all = [Lane::NeedsYou, Lane::Stuck, Lane::InFlight, Lane::Queued, Lane::Planned, Lane::Done, Lane::Dropped];
    for (i, l) in all.iter().enumerate() {
        assert!(!l.label().is_empty() && !l.hint().is_empty() && !l.mark().is_empty() && !l.slug().is_empty());
        for m in &all[i + 1..] {
            assert_ne!(l.slug(), m.slug());
            assert_ne!(l.mark(), m.mark());
        }
    }
    assert_eq!(BOARD.len(), 6);
    assert!(!BOARD.contains(&Lane::Dropped));
}
