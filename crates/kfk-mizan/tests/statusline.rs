use kfk_mizan::statusline;

#[test]
fn statusline_reads_windows_and_context() {
    let j = r#"{"model":{"id":"claude-fable-5-2","display_name":"Fable 5.2"},"context_window":{"current_usage":{"input_tokens":2,"cache_creation_input_tokens":756,"cache_read_input_tokens":136040,"output_tokens":10}},"rate_limits":{"seven_day":{"used_percentage":96,"resets_at":1795442400},"five_hour":{"used_percentage":12.5}}}"#;
    let (w, used, model, line) = statusline(j, "o1");
    assert_eq!(used, 136798);
    assert_eq!(model, "claude-fable-5-2");
    assert_eq!((w["seven_day"].util, w["seven_day"].reset, w["five_hour"].util, w["five_hour"].reset), (Some(96.0), Some(1795442400.0), Some(12.5), None));
    assert_eq!(line, "o1 | Fable 5.2 | ctx 136k | wk 96%");
    let (w, used, model, line) = statusline("not json", "");
    assert!(w.is_empty() && used == 0 && model.is_empty());
    assert_eq!(line, "kfk |  | ctx 0 | wk ?");
}
