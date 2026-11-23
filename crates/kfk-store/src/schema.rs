//! the tables; nothing is ever deleted from tasks, runs or events
pub const SCHEMA: &str = "
create table if not exists config(k text primary key, v text);
create table if not exists accounts(slot integer primary key, label text, minted text, off integer default 0,
  windows text default '{}', asof real);
create table if not exists seats(seat text primary key, role text, slot integer, sid text, pid integer, since real,
  ctx integer, model text default '', effort text default '', state text default 'stopped',
  restarts text default '[]', idle_since real, note text default '');
create table if not exists tasks(id integer primary key autoincrement, parent integer, owner text default '',
  title text not null, spec text default '', dod text default '', chk text default '', repo text default '',
  base text default '', class text default 'impl', model text default '', effort text default '',
  timeout_s integer, max_attempts integer, deps text default '', status text default 'todo',
  attempts integer default 0, runner integer, result text default '', created real, updated real);
create table if not exists runs(id integer primary key autoincrement, task integer, attempt integer, slot integer,
  model text, effort text, pid integer, status text, started real, ended real, dir text default '', wd text default '',
  check_rc integer, in_tok integer default 0, cw_tok integer default 0, cr_tok integer default 0,
  out_tok integer default 0, cost real default 0, turns integer default 0, note text default '');
create table if not exists events(id integer primary key autoincrement, ts real, task integer, actor text,
  kind text, data text);
create table if not exists sessions(sid text primary key, seat text, role text, slot integer, model text, effort text,
  started real, ended real, path text default '', bytes_done integer default 0, last text default '');
create table if not exists spend(sid text, hour integer, in_tok integer default 0, cw_tok integer default 0,
  cr_tok integer default 0, out_tok integer default 0, turns integer default 0, primary key(sid, hour));
create index if not exists tasks_parent on tasks(parent, status);
create index if not exists runs_task on runs(task);
";

/// columns added after the first release; `alter table` fails harmlessly where they already exist
pub const ADDED: &[&str] = &["alter table seats add column note text default ''"];
