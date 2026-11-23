use kfk_mizan::{pane_wait, Wait};

#[test]
fn dialogs_are_recognised_in_the_tail_of_a_pane() {
    assert_eq!(pane_wait(""), None);
    assert_eq!(pane_wait("❯ \n  cos | Fable 5.2 | ctx 0 | wk ?\n"), None);
    assert_eq!(pane_wait(" Security guide\n\n ❯ No, exit\n   Yes, I trust this folder\n\n Enter to confirm\n\n\n"), Some(Wait::Trust));
    assert_eq!(pane_wait("WARNING: Bypass Permissions mode\n 1. No, exit\n 2. Yes, I accept\n"), Some(Wait::Bypass));
    assert_eq!(pane_wait("Bypass permissions mode is on"), None);
    assert_eq!(pane_wait("Select login method:\n 1. Claude account"), Some(Wait::Login));
    assert_eq!(pane_wait("Paste code here if prompted >"), Some(Wait::Login));
    assert_eq!(pane_wait("Invalid API key · Please run /login"), Some(Wait::Login));
    assert_eq!(pane_wait("You've hit your limit · resets 3pm"), Some(Wait::Limit));
    assert_eq!(pane_wait("Usage limit reached"), Some(Wait::Limit));
    assert_eq!(pane_wait("5-hour limit reached · resets 9pm"), Some(Wait::Limit));
    // only the tail counts: an old dialog that scrolled away is over
    let old = format!("Yes, I trust this folder\n{}", "line\n".repeat(30));
    assert_eq!(pane_wait(&old), None);
    for w in [Wait::Trust, Wait::Bypass, Wait::Login, Wait::Limit] {
        assert!(!w.note().is_empty());
    }
}
