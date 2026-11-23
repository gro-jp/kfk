use kfk_logs::*;

#[test]
fn chunk_keeps_a_partial_last_line() {
    let (l, used) = chunk(b"one\ntwo\n\nthr");
    assert_eq!(l, vec![(0, &b"one"[..]), (4, &b"two"[..])]);
    assert_eq!(used, 9);
    assert_eq!(chunk(b""), (vec![], 0));
    assert_eq!(chunk(b"no newline"), (vec![], 0));
}
