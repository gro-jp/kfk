//! the whole tool against a fake `claude`: a private tmux server, a file vault, a throwaway home and PostgreSQL
#[test]
fn end_to_end_with_a_fake_claude() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/e2e.sh");
    let o = std::process::Command::new("bash").arg(script).env("KFK_BIN", env!("CARGO_BIN_EXE_kfk")).output().unwrap();
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(o.status.success() && out.contains("ALL PASSED"), "{out}\n{}", String::from_utf8_lossy(&o.stderr));
}
