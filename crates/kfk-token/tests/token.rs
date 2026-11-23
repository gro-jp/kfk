use kfk_token::*;


fn tok(n: usize) -> String {
    format!("{PREFIX}{}", "a".repeat(n - PREFIX.len()))
}

#[test]
fn whole_token_on_one_line_takes_no_trailing_word() {
    let t = tok(LEN);
    assert_eq!(from_log(&format!("Your token:\n{t}\nStore it safely\n")).unwrap(), t);
    assert_eq!(from_log(&format!("\x1b[1m{t}\x1b[0m\x1b[2;1HStore")).unwrap(), t);
}

#[test]
fn wrapped_token_is_joined() {
    let t = tok(LEN);
    let log = format!("x {}\r\n   {}  \n next words here\n", &t[..79], &t[79..]);
    assert_eq!(from_log(&log).unwrap(), t);
}

#[test]
fn redraws_prefer_the_whole_one_and_none_is_none() {
    let t = tok(LEN);
    let log = format!("{}\n\nmore text follows\n{t}\n\n{}\n\nmore text\n", &t[..50], &t[..60]);
    assert_eq!(from_log(&log).unwrap(), t);
    assert_eq!(from_log(&format!("{} and words", &t[..40])).unwrap().len(), 40);
    assert!(from_log("no token here").is_none());
}

#[test]
fn shapes() {
    assert!(shape_ok(&tok(LEN)));
    assert!(!shape_ok(&tok(60)));
    assert!(!shape_ok("abc"));
    assert!(!shape_ok(&format!("{}\"", tok(LEN))));
    assert!(label_ok("a.b+c@x-y.com") && !label_ok("") && !label_ok("a b") && !label_ok("a\"b"));
}

#[test]
fn file_vault_round_trip() {
    let f = std::env::temp_dir().join(format!("kfk-vault-{}.json", uuid::Uuid::new_v4()));
    let v = FileVault(f.to_string_lossy().into_owned());
    assert_eq!(v.get(3), None);
    v.put(3, "t3", "a@b").unwrap();
    v.put(4, "t4", "a@b").unwrap();
    assert_eq!(v.get(3).as_deref(), Some("t3"));
    assert_eq!(v.get(-1), None);
    v.del(3);
    assert_eq!(v.get(3), None);
    assert_eq!(v.get(4).as_deref(), Some("t4"));
    let _ = std::fs::remove_file(f);
}

#[test]
fn keychain_round_trip_under_a_test_service() {
    let k = Keychain("kfk-selftest");
    k.del(7);
    assert_eq!(k.get(7), None);
    assert_eq!(k.get(-1), None);
    let t = tok(95);
    k.put(7, &t, "a@b").unwrap();
    assert_eq!(k.get(7).as_deref(), Some(t.as_str()));
    let t2 = tok(96);
    k.put(7, &t2, "a@b").unwrap(); // replaces
    assert_eq!(k.get(7).as_deref(), Some(t2.as_str()));
    k.del(7);
    assert_eq!(k.get(7), None);
    // a write that cannot be read back is an error, not a silent loss
    assert!(k.put(-1, &t, "a@b").unwrap_err().to_string().contains("did not read back"));
    k.del(-1);
}

#[test]
fn the_vault_is_the_keychain_unless_a_file_is_named() {
    let f = std::env::temp_dir().join(format!("kfk-vault-{}.json", uuid::Uuid::new_v4()));
    std::env::set_var("KFK_TOKEN_FILE", &f);
    let v = vault();
    v.put(1, "from-file", "a@b").unwrap();
    assert!(std::fs::read_to_string(&f).unwrap().contains("from-file"));
    std::env::set_var("KFK_TOKEN_FILE", "");
    assert_eq!(vault().get(-1), None);
    std::env::remove_var("KFK_TOKEN_FILE");
    assert_eq!(vault().get(-1), None);
    let _ = std::fs::remove_file(f);
}
