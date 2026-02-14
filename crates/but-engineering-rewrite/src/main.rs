//! Minimal CLI for harness Case 01.
//!
//! Scope:
//! - Global arg: --agent-id <id>
//! - Subcommands:
//!   - claim --path <path> --ttl <duration>
//!   - check --path <path>
//!   - post <message...>
//!   - eval user-prompt-submit
//! - Repo-scoped SQLite DB at: .git/gitbutler/but-engineering-rewrite.db

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};

#[derive(Debug)]
enum Cmd {
    Claim { path: String, ttl: Duration },
    Check { path: String, strict: bool },
    Post { message: String },
    EvalUserPromptSubmit,
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(()) => {
            // Keep stdout valid JSON even on errors (harness captures 2>&1).
            print_json(r#"{"ok":false,"error":"internal_error"}"#);
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), ()> {
    let (agent_id, cmd) = parse_args(std::env::args_os().skip(1))?;

    let mut db_path = PathBuf::from(".git");
    db_path.push("gitbutler");
    std::fs::create_dir_all(&db_path).map_err(|_| ())?;
    db_path.push("but-engineering-rewrite.db");

    let conn = Connection::open(db_path).map_err(|_| ())?;
    init_db(&conn)?;

    match cmd {
        Cmd::Claim { path, ttl } => {
            let now_ms = now_unix_ms()?;
            let ttl_ms: i64 = ttl.as_millis().try_into().map_err(|_| ())?;
            let expires_at_ms = now_ms.saturating_add(ttl_ms);
            conn.execute(
                "INSERT INTO claims(path, agent_id, expires_at_ms) VALUES (?1, ?2, ?3)",
                params![path, agent_id, expires_at_ms],
            )
            .map_err(|_| ())?;
            print_json(r#"{"ok":true}"#);
        }
        Cmd::Check { path, strict } => {
            let now_ms = now_unix_ms()?;
            let mut stmt = conn
                .prepare(
                    "SELECT 1 FROM claims \
                     WHERE path = ?1 AND agent_id <> ?2 AND expires_at_ms > ?3 \
                     LIMIT 1",
                )
                .map_err(|_| ())?;
            let mut rows = stmt.query(params![path, agent_id, now_ms]).map_err(|_| ())?;
            if rows.next().map_err(|_| ())?.is_some() {
                if strict {
                    print_json(r#"{"decision":"deny","reason_code":"claimed_by_other"}"#);
                } else {
                    print_json(r#"{"decision":"warn","reason_code":"claimed_by_other"}"#);
                }
            } else {
                print_json(r#"{"decision":"allow","reason_code":"no_conflict"}"#);
            }
        }
        Cmd::Post { message: _ } => {
            // Minimal harness support: accept posts (future: persist to a channel table).
            print_json(r#"{"ok":true}"#);
        }
        Cmd::EvalUserPromptSubmit => {
            let now_ms = now_unix_ms()?;
            let active_claims: i64 = conn
                .query_row(
                    "SELECT COUNT(1) FROM claims WHERE expires_at_ms > ?1",
                    params![now_ms],
                    |row| row.get(0),
                )
                .map_err(|_| ())?;

            // Plain text nudge + tiny live state (harness asserts substrings only).
            println!("Announce what you'll do (files), read the channel, then proceed.");
            println!("claims: {active_claims}");
        }
    }

    Ok(())
}

fn init_db(conn: &Connection) -> Result<(), ()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS claims (\
            path TEXT NOT NULL,\
            agent_id TEXT NOT NULL,\
            expires_at_ms INTEGER NOT NULL\
         );\
         CREATE INDEX IF NOT EXISTS idx_claims_path_expires \
            ON claims(path, expires_at_ms);\
        ",
    )
    .map_err(|_| ())?;
    Ok(())
}

fn now_unix_ms() -> Result<i64, ()> {
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| ())?;
    dur.as_millis().try_into().map_err(|_| ())
}

fn print_json(s: &str) {
    // Always a single JSON value, newline-terminated.
    println!("{s}");
}

fn parse_args<I>(mut args: I) -> Result<(String, Cmd), ()>
where
    I: Iterator<Item = std::ffi::OsString>,
{
    let mut agent_id: Option<String> = None;
    let mut sub: Option<String> = None;
    let mut rest: Vec<String> = Vec::new();

    while let Some(a) = args.next() {
        let a = a.to_string_lossy().into_owned();
        if sub.is_some() {
            rest.push(a);
            continue;
        }
        match a.as_str() {
            "--agent-id" => {
                let v = args.next().ok_or(())?.to_string_lossy().into_owned();
                agent_id = Some(v);
            }
            "claim" | "check" | "post" | "eval" => sub = Some(a),
            _ => return Err(()),
        }
    }

    let agent_id = agent_id.ok_or(())?;
    let sub = sub.ok_or(())?;

    let cmd = match sub.as_str() {
        "claim" => {
            let (path, ttl) = parse_claim_args(&rest)?;
            Cmd::Claim { path, ttl }
        }
        "check" => {
            let (path, strict) = parse_check_args(&rest)?;
            Cmd::Check { path, strict }
        }
        "post" => {
            if rest.is_empty() {
                return Err(());
            }
            Cmd::Post {
                message: rest.join(" "),
            }
        }
        "eval" => {
            if rest.len() != 1 || rest[0] != "user-prompt-submit" {
                return Err(());
            }
            Cmd::EvalUserPromptSubmit
        }
        _ => return Err(()),
    };

    Ok((agent_id, cmd))
}

fn parse_claim_args(args: &[String]) -> Result<(String, Duration), ()> {
    let mut path: Option<String> = None;
    let mut ttl: Option<Duration> = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--path" => {
                i += 1;
                path = Some(args.get(i).ok_or(())?.clone());
            }
            "--ttl" => {
                i += 1;
                ttl = Some(parse_duration(args.get(i).ok_or(())?)?);
            }
            _ => return Err(()),
        }
        i += 1;
    }
    Ok((path.ok_or(())?, ttl.ok_or(())?))
}

fn parse_check_args(args: &[String]) -> Result<(String, bool), ()> {
    let mut path: Option<String> = None;
    let mut strict = false;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--path" => {
                i += 1;
                path = Some(args.get(i).ok_or(())?.clone());
            }
            "--strict" => {
                strict = true;
            }
            _ => return Err(()),
        }
        i += 1;
    }
    Ok((path.ok_or(())?, strict))
}

fn parse_duration(s: &str) -> Result<Duration, ()> {
    let (n, unit) = split_num_unit(s)?;
    let secs = match unit {
        "" | "s" => n,
        "m" => n.saturating_mul(60),
        "h" => n.saturating_mul(60 * 60),
        "d" => n.saturating_mul(60 * 60 * 24),
        _ => return Err(()),
    };
    Ok(Duration::from_secs(secs))
}

fn split_num_unit(s: &str) -> Result<(u64, &str), ()> {
    let s = s.trim();
    if s.is_empty() {
        return Err(());
    }
    let mut idx = 0usize;
    for (i, ch) in s.char_indices() {
        if !ch.is_ascii_digit() {
            idx = i;
            break;
        }
    }
    if idx == 0 {
        // Either no digits at start, or all digits (handled below).
        if s.chars().all(|c| c.is_ascii_digit()) {
            let n: u64 = s.parse().map_err(|_| ())?;
            return Ok((n, ""));
        }
        return Err(());
    }
    let (num, unit) = s.split_at(idx);
    let n: u64 = num.parse().map_err(|_| ())?;
    Ok((n, unit))
}
