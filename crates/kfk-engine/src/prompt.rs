//! the text an agent starts from
use crate::prelude::*;

pub fn prompt(s: &Store, t: &Task, rid: i64, wd: &str) -> String {
    let mut p = format!(
        "{}\n# Task T{}: {}\n\n## Spec\n{}\n\n## Done when\n{}\n\n## Check (kfk runs this in {} after you exit; exit status 0 = pass)\n    {}\n",
        kfk_roles::AGENT, t.id, t.title, t.spec.trim(), t.dod.trim(), wd, t.chk.trim()
    );
    if let Ok(Some(dir)) = s.last_failed_dir(t.id, rid) {
        if let Ok(log) = fs::read_to_string(format!("{dir}/check.log")) {
            p += &format!("\n## A previous attempt left this work tree and failed the check. Its output (tail):\n```\n{}\n```\nFix what it shows; do not start over unless the approach is wrong.\n", tail_lines(&log, 40, 4000));
        }
    }
    p
}
