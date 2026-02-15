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
use std::{collections::HashSet};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use serde_json::Value;

#[derive(Debug)]
enum Cmd {
    Claim { path: String, ttl: Duration },
    Release { path: String },
    Claims { path_prefix: Option<String> },
    Check { path: String, strict: bool },
    Post { message: String },
    PostTyped { kind: String, json: String },
    Read { kind: Option<String> },
    Brief { kind: Option<String>, all: bool },
    Digest { kind: Option<String>, all: bool },
    Status { value: Option<String> },
    Plan { value: Option<String> },
    Agents,
    Done { summary: String },
    EvalUserPromptSubmit,
}

fn is_path_token_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'/')
}

// For boundary checks: treat "." as a continuation only when it starts an extension-like suffix (".bak").
fn continues_path_token(hs: &[u8], idx: usize) -> bool {
    if idx >= hs.len() {
        return false;
    }
    let b = hs[idx];
    if b == b'.' {
        return idx + 1 < hs.len() && hs[idx + 1].is_ascii_alphanumeric();
    }
    is_path_token_byte(b)
}

// Avoid substring false positives: `src/app.txt` should not match `src/app.txt.bak`.
// Treat directory needles like `src/` as directory mentions, not generic prefixes: `src/` should not match
// `src/app.txt` (the more specific file needle should match instead).
fn contains_path_token(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() || haystack.is_empty() {
        return false;
    }
    let hs = haystack.as_bytes();
    let mut start = 0usize;
    while let Some(rel) = haystack.get(start..).and_then(|s| s.find(needle)) {
        let i = start + rel;
        let j = i + needle.len();

        let before_ok = i == 0 || !is_path_token_byte(hs[i - 1]);
        let after_ok = j == hs.len() || !continues_path_token(hs, j);
        if before_ok && after_ok {
            return true;
        }
        start = i + 1;
        if start >= hs.len() {
            break;
        }
    }
    false
}

fn relevant_needles_for_path(path: &str) -> Vec<String> {
    let mut needles: Vec<String> = Vec::new();
    needles.push(path.to_owned());
    needles.push(format!("{path}/"));
    needles.push(format!("./{path}"));
    needles.push(format!("./{path}/"));
    // Add all ancestors: a/b/c.txt -> a/b and a
    let mut cur = path;
    while let Some((parent, _base)) = cur.rsplit_once('/') {
        if parent.is_empty() {
            break;
        }
        needles.push(parent.to_owned());
        needles.push(format!("{parent}/"));
        needles.push(format!("./{parent}"));
        needles.push(format!("./{parent}/"));
        cur = parent;
    }
    needles
}

fn extract_message_text(body_v: &Value, body_json: &str) -> String {
    if let Some(t) = body_v.get("text").and_then(|v| v.as_str()) {
        return t.to_owned();
    }
    if let Some(s) = body_v.as_str() {
        return s.to_owned();
    }
    body_json.to_owned()
}

fn last_relevant_update_from_agent(
    conn: &Connection,
    from_agent_id: &str,
    path: &str,
) -> Result<Option<(i64, i64, String)>, ()> {
    let needles = relevant_needles_for_path(path);
    let mut stmt = conn
        .prepare(
            "SELECT id, created_at_ms, body_json FROM messages \
             WHERE agent_id = ?1 AND kind IN ('message','discovery') \
             ORDER BY id DESC \
             LIMIT 500",
        )
        .map_err(|_| ())?;
    let rows = stmt
        .query_map(params![from_agent_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|_| ())?;

    for r in rows {
        let (id, created_at_ms, body_json) = r.map_err(|_| ())?;
        let body_v: Value =
            serde_json::from_str(&body_json).unwrap_or(Value::String(body_json.clone()));
        let txt = extract_message_text(&body_v, &body_json);
        if needles.iter().any(|n| contains_path_token(&txt, n)) {
            return Ok(Some((id, created_at_ms, txt)));
        }
    }
    Ok(None)
}

fn requester_has_acked_since(
    conn: &Connection,
    requester_agent_id: &str,
    target_agent_id: &str,
    since_created_at_ms: i64,
) -> Result<bool, ()> {
    // Humans often omit the colon after the @mention. Treat both as equivalent, but still require
    // a near-start direct mention to avoid mid-sentence false positives.
    //
    // Also accept common sentence punctuation (`.`/`!`/`?`) used in place of the trailing colon.
    let mut ack_needles_lower: Vec<String> = Vec::with_capacity(8);
    for base in [
        format!("@{target_agent_id}: ack"),
        format!("@{target_agent_id} ack"),
        format!("@{target_agent_id}: acknowledged"),
        format!("@{target_agent_id} acknowledged"),
        format!("@{target_agent_id}: thanks"),
        format!("@{target_agent_id} thanks"),
        format!("@{target_agent_id}: got it"),
        format!("@{target_agent_id} got it"),
    ] {
        let base_lower = base.to_ascii_lowercase();
        // Also accept a bare ack prefix without punctuation (often typed as "@A: ack thanks").
        ack_needles_lower.push(base_lower.clone());
        ack_needles_lower.push(format!("{base_lower} "));
        ack_needles_lower.push(format!("{base_lower}\t"));
        for p in [':', '.', '!', '?', ','] {
            ack_needles_lower.push(format!("{base_lower}{p}"));
        }
    }
    let mut stmt = conn
        .prepare(
            "SELECT body_json FROM messages \
             WHERE agent_id = ?1 AND kind = 'message' AND created_at_ms >= ?2 \
             ORDER BY id DESC \
             LIMIT 500",
        )
        .map_err(|_| ())?;
    let rows = stmt
        .query_map(params![requester_agent_id, since_created_at_ms], |row| row.get::<_, String>(0))
        .map_err(|_| ())?;

    for r in rows {
        let body_json = r.map_err(|_| ())?;
        let body_v: Value =
            serde_json::from_str(&body_json).unwrap_or(Value::String(body_json.clone()));
        let txt = extract_message_text(&body_v, &body_json);
        let mut scanned_bytes: usize = 0;
        let mut in_fenced_code_block = false;
        for line in txt.lines().take(16) {
            scanned_bytes = scanned_bytes.saturating_add(line.len());
            if scanned_bytes > 1024 {
                break;
            }
            let l = line.trim_start();
            if l.starts_with("```") {
                in_fenced_code_block = !in_fenced_code_block;
                continue;
            }
            if in_fenced_code_block || l.is_empty() {
                continue;
            }
            for candidate in [
                l,
                strip_common_list_prefix(l),
                strip_leading_markdown_emphasis(l),
                strip_leading_markdown_emphasis(strip_common_list_prefix(l)),
            ] {
                for c in [candidate, strip_leading_wrappers(candidate)] {
                    let c_lower = c.to_ascii_lowercase();
                    if ack_needles_lower.iter().any(|n| c_lower.starts_with(n)) {
                        return Ok(true);
                    }
                }
            }
        }
    }
    Ok(false)
}

fn strip_common_list_prefix(line: &str) -> &str {
    // Support common "reply checklist" formatting like:
    // - @B: ack: ...
    // - [x] @B: ack: ...
    // 1. @B: ack: ...
    // - 1. @B: ack: ...
    // without becoming overly permissive (avoid mid-sentence false positives).
    let b = line.as_bytes();
    if b.is_empty() {
        return line;
    }

    let mut i: usize = 0;
    if matches!(b[0], b'-' | b'*' | b'+') {
        i = 1;
    }

    // Skip whitespace after a bullet marker (or at start of line, though callers typically lstrip already).
    while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
        i += 1;
    }

    // Optional numeric list marker (supports nested patterns like "- 1. ...").
    let mut j = i;
    while j < b.len() && b[j].is_ascii_digit() {
        j += 1;
    }
    if j > i && j < b.len() && (b[j] == b'.' || b[j] == b')') {
        // Require at least one whitespace after the marker to avoid stripping e.g. "1.23".
        if j + 1 < b.len() && (b[j + 1] == b' ' || b[j + 1] == b'\t') {
            i = j + 1;
            while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
                i += 1;
            }
        }
    }

    // If we didn't strip a bullet and didn't strip a numeric marker, this isn't a list prefix.
    if i == 0 {
        return line;
    }

    // GitHub-style task list marker: "- [x] ..." / "- [ ] ..."
    if i + 3 <= b.len()
        && b[i] == b'['
        && (b[i + 1] == b' ' || b[i + 1] == b'x' || b[i + 1] == b'X')
        && b[i + 2] == b']'
    {
        i += 3;
        while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
            i += 1;
        }
    }
    &line[i..]
}

fn strip_leading_markdown_emphasis(line: &str) -> &str {
    // Support common emphasis wrappers at the start of a reply line, e.g.:
    // **@B: ack:** ...
    // *@B: ack:* ...
    //
    // Keep this intentionally narrow: only strip leading '*' / '_' when they are immediately
    // followed by '@'.
    let b = line.as_bytes();
    if b.is_empty() {
        return line;
    }

    let mut i: usize = 0;
    while i < b.len() && (b[i] == b'*' || b[i] == b'_') {
        i += 1;
    }
    if i == 0 || i > 3 || i >= b.len() {
        return line;
    }
    if b[i] != b'@' {
        return line;
    }
    &line[i..]
}

fn strip_leading_wrappers(line: &str) -> &str {
    // Support common punctuation wrappers at the start of a directive, e.g.:
    // (@B: resolve: ...) / [@B: ack: ...]
    // `@B: resolve:` (inline-code formatted)
    //
    // Keep this intentionally narrow to avoid mid-sentence false positives.
    let b = line.as_bytes();
    if b.is_empty() {
        return line;
    }

    let mut i: usize = 0;
    let mut stripped: usize = 0;
    while i < b.len() && stripped < 3 {
        match b[i] {
            b'(' | b'[' | b'{' => {
                i += 1;
                stripped += 1;
                while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
                    i += 1;
                }
            }
            b'`' => {
                // Inline-code wrapper: only treat as a wrapper when it directly precedes a mention,
                // e.g. "`@B: resolve:` ...". Avoid becoming permissive about arbitrary backticks.
                if i + 1 < b.len() && b[i + 1] == b'@' {
                    i += 1;
                    stripped += 1;
                    while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
                        i += 1;
                    }
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    &line[i..]
}

fn is_indented_at_mention(line: &str) -> bool {
    // Ignore indented Markdown "code block" style pastes like:
    // "    @B: resolve: ..." which are almost always prior context.
    // Keep this intentionally narrow to avoid breaking nested-list variants.
    let b = line.as_bytes();
    if b.is_empty() {
        return false;
    }
    let mut i: usize = 0;
    let mut spaces: usize = 0;
    let mut saw_tab = false;
    while i < b.len() {
        match b[i] {
            b' ' => {
                spaces += 1;
                i += 1;
            }
            b'\t' => {
                saw_tab = true;
                i += 1;
            }
            _ => break,
        }
    }
    if i >= b.len() || b[i] != b'@' {
        return false;
    }
    saw_tab || spaces >= 4
}

fn is_explicit_closure_to_me(
    text: &str,
    ack_to_me_prefix: &str,
    resolve_to_me_prefix_lower: &str,
    resolved_to_me_prefix_lower: &str,
    released_to_me_prefix_lower: &str,
) -> bool {
    let ack_to_me_prefix_lower = ack_to_me_prefix.to_ascii_lowercase();
    let ack_to_me_prefix_space_lower =
        ack_to_me_prefix_lower.replacen(": ack:", " ack:", 1);
    let mut ack_needles_lower: Vec<String> = Vec::with_capacity(12);
    let mut ack_bare_lower: Vec<String> = Vec::with_capacity(4);
    for n in [
        ack_to_me_prefix_lower.clone(),
        ack_to_me_prefix_space_lower.clone(),
        ack_to_me_prefix_lower.replacen(": ack:", ": acknowledged", 1),
        ack_to_me_prefix_space_lower.replacen(" ack:", " acknowledged", 1),
        // Common "ack" synonyms used as a directed reply to an agent.
        ack_to_me_prefix_lower.replacen(": ack:", ": thanks", 1),
        ack_to_me_prefix_space_lower.replacen(" ack:", " thanks", 1),
        ack_to_me_prefix_lower.replacen(": ack:", ": got it", 1),
        ack_to_me_prefix_space_lower.replacen(" ack:", " got it", 1),
    ] {
        ack_needles_lower.push(n.to_owned());
        if let Some(base) = n.strip_suffix(':') {
            ack_bare_lower.push(base.to_owned());
            for p in ['.', '!', '?', ','] {
                ack_needles_lower.push(format!("{base}{p}"));
            }
        }
    }

    // Treat `@<me>: ack:` as explicit closure, but avoid quote-induced false positives and
    // false negatives by evaluating per-line (humans commonly quote on one line and reply on the next).
    //
    // For `resolve:` / `resolved:` / `released:`, we keep the heuristic intentionally narrow:
    // only treat it as closure when it's a near-start direct `@<me>:` mention on that specific line.
    // Also accept common sentence punctuation variants like `resolved.` / `resolve!` (humans often
    // use '.'/'!'/'?' instead of a trailing colon).
    //
    // NOTE: ignore fenced code blocks (```), since users often paste prior context containing
    // `@<me>: resolve:` / `@<me>: ack:` inside code blocks.
    let mut closure_needles: Vec<String> = Vec::with_capacity(12);
    for n in [
        resolve_to_me_prefix_lower,
        resolved_to_me_prefix_lower,
        released_to_me_prefix_lower,
    ] {
        closure_needles.push(n.to_owned());
        if let Some(base) = n.strip_suffix(':') {
            // `resolved?` / `resolve?` is frequently used as a question ("is this resolved?"),
            // so avoid treating `?` as an explicit closure signal.
            for p in ['.', '!', ','] {
                closure_needles.push(format!("{base}{p}"));
            }
        }
    }
    // Also treat bare `@<me>: resolve` / `@<me>: resolved` as explicit closure when followed by
    // a word boundary. Humans often omit trailing punctuation entirely (e.g. "@B: resolved thanks").
    // Keep this intentionally narrow; in particular, reject `resolve?` / `resolved?`.
    let mut closure_bare: Vec<String> = Vec::with_capacity(4);
    for n in [
        resolve_to_me_prefix_lower,
        resolved_to_me_prefix_lower,
        released_to_me_prefix_lower,
    ] {
        if let Some(base) = n.strip_suffix(':') {
            closure_bare.push(base.to_owned());
        }
    }

    let mut scanned_bytes: usize = 0;
    let mut in_fenced_code_block = false;
    for line in text.lines().take(16) {
        scanned_bytes = scanned_bytes.saturating_add(line.len());
        if scanned_bytes > 1024 {
            break;
        }

        if is_indented_at_mention(line) {
            continue;
        }

        let l = line.trim_start();
        if l.starts_with("```") {
            in_fenced_code_block = !in_fenced_code_block;
            continue;
        }
        if in_fenced_code_block {
            continue;
        }
        if l.is_empty() {
            continue;
        }

        let l_lower = l.to_ascii_lowercase();
        let l_md = strip_leading_markdown_emphasis(l);
        let l_md_lower = l_md.to_ascii_lowercase();
        let l_list_md = strip_leading_markdown_emphasis(strip_common_list_prefix(l));
        let l_list_md_lower = l_list_md.to_ascii_lowercase();

        let l_wrap = strip_leading_wrappers(l);
        let l_wrap_lower = l_wrap.to_ascii_lowercase();
        let l_md_wrap = strip_leading_wrappers(l_md);
        let l_md_wrap_lower = l_md_wrap.to_ascii_lowercase();
        let l_list_md_wrap = strip_leading_wrappers(l_list_md);
        let l_list_md_wrap_lower = l_list_md_wrap.to_ascii_lowercase();

        let is_ack_line = |raw: &str, lower: &str| {
            if ack_needles_lower.iter().any(|n| lower.starts_with(n)) {
                return true;
            }
            for base in &ack_bare_lower {
                if lower.starts_with(base) {
                    let after = raw.as_bytes().get(base.len()).copied();
                    match after {
                        None => return true,
                        Some(b'?') => return true,
                        Some(b' ' | b'\t') => return true,
                        Some(b'.' | b'!' | b',' | b':') => return true,
                        _ => continue,
                    }
                }
            }
            false
        };

        if is_ack_line(l, &l_lower)
            || is_ack_line(l_md, &l_md_lower)
            || is_ack_line(l_list_md, &l_list_md_lower)
            || is_ack_line(l_wrap, &l_wrap_lower)
            || is_ack_line(l_md_wrap, &l_md_wrap_lower)
            || is_ack_line(l_list_md_wrap, &l_list_md_wrap_lower)
        {
            return true;
        }

        let candidates: [(&str, &str); 6] = [
            (l, &l_lower),
            (l_md, &l_md_lower),
            (l_list_md, &l_list_md_lower),
            (l_wrap, &l_wrap_lower),
            (l_md_wrap, &l_md_wrap_lower),
            (l_list_md_wrap, &l_list_md_wrap_lower),
        ];
        for (c_raw, c_lower) in candidates {
            for needle in &closure_needles {
                if let Some(idx) = c_lower.find(needle) {
                    if idx > 64 {
                        continue;
                    }
                    if idx > 0 {
                        let prev = c_raw.as_bytes().get(idx - 1).copied().unwrap_or(b' ');
                        if prev != b' '
                            && prev != b':'
                            && prev != b'('
                            && prev != b'['
                            && prev != b'{'
                        {
                            continue;
                        }
                    }
                    let prefix = &c_raw[..idx];
                    if prefix.contains('"')
                        || prefix.contains('\'')
                        || prefix.contains('`')
                        || prefix.contains('>')
                    {
                        continue;
                    }
                    return true;
                }
            }
            for needle in &closure_bare {
                if let Some(idx) = c_lower.find(needle) {
                    if idx > 64 {
                        continue;
                    }
                    if idx > 0 {
                        let prev = c_raw.as_bytes().get(idx - 1).copied().unwrap_or(b' ');
                        if prev != b' '
                            && prev != b':'
                            && prev != b'('
                            && prev != b'['
                            && prev != b'{'
                        {
                            continue;
                        }
                    }
                    let prefix = &c_raw[..idx];
                    if prefix.contains('"')
                        || prefix.contains('\'')
                        || prefix.contains('`')
                        || prefix.contains('>')
                    {
                        continue;
                    }
                    let after = c_raw.as_bytes().get(idx + needle.len()).copied();
                    match after {
                        None => return true,
                        Some(b'?') => continue,
                        Some(b' ' | b'\t') => return true,
                        Some(b'.' | b'!' | b',' | b':') => return true,
                        _ => continue,
                    }
                }
            }
        }
    }

    false
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
    touch_agent(&conn, &agent_id)?;

    match cmd {
        Cmd::Claim { path, ttl } => {
            let path = normalize_claim_path(&path);
            let now_ms = now_unix_ms()?;
            let ttl_ms: i64 = ttl.as_millis().try_into().map_err(|_| ())?;
            let expires_at_ms = now_ms.saturating_add(ttl_ms);
            // Treat repeated claims by the same agent for the same path as "renewal"
            // rather than creating multiple rows (keeps `claims` and conflict output
            // low-noise and matches the "lease refresh" intent).
            conn.execute(
                "DELETE FROM claims WHERE path = ?1 AND agent_id = ?2",
                params![path, agent_id],
            )
            .map_err(|_| ())?;
            conn.execute(
                "INSERT INTO claims(path, agent_id, expires_at_ms) VALUES (?1, ?2, ?3)",
                params![path, agent_id, expires_at_ms],
            )
            .map_err(|_| ())?;
            print_json(r#"{"ok":true}"#);
        }
        Cmd::Release { path } => {
            let path = normalize_claim_path(&path);
            conn.execute(
                "DELETE FROM claims WHERE path = ?1 AND agent_id = ?2",
                params![path, agent_id],
            )
            .map_err(|_| ())?;
            print_json(r#"{"ok":true}"#);
        }
        Cmd::Claims { path_prefix } => {
            let now_ms = now_unix_ms()?;
            let mut stmt = if let Some(prefix) = &path_prefix {
                conn.prepare(
                    "SELECT path, agent_id, expires_at_ms FROM claims \
                     WHERE expires_at_ms > ?1 \
                       AND (path = ?2 OR ?2 LIKE path || '/%' OR path LIKE ?2 || '/%') \
                     ORDER BY expires_at_ms ASC, path ASC",
                )
                .map_err(|_| ())?
            } else {
                conn.prepare(
                    "SELECT path, agent_id, expires_at_ms FROM claims \
                     WHERE expires_at_ms > ?1 \
                     ORDER BY expires_at_ms ASC, path ASC",
                )
                .map_err(|_| ())?
            };

            fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<(String, String, i64)> {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            }

            let rows = if let Some(prefix) = &path_prefix {
                stmt.query_map(params![now_ms, prefix], map_row)
                    .map_err(|_| ())?
            } else {
                stmt.query_map(params![now_ms], map_row)
                    .map_err(|_| ())?
            };

            let mut claims: Vec<Value> = Vec::new();
            for r in rows {
                let (path, agent_id, expires_at_ms) = r.map_err(|_| ())?;
                claims.push(json!({
                    "path": path,
                    "agent_id": agent_id,
                    "expires_at_ms": expires_at_ms,
                }));
            }

            let out = json!({
                "ok": true,
                "claims": claims,
            });
            print_json(&out.to_string());
        }
        Cmd::Check { path, strict } => {
            let path = normalize_claim_path(&path);
            let now_ms = now_unix_ms()?;
            let mut stmt = conn
                .prepare(
                    "SELECT agent_id, path, expires_at_ms FROM claims \
                     WHERE agent_id <> ?2 AND expires_at_ms > ?3 \
                       AND (path = ?1 OR ?1 LIKE path || '/%' OR path LIKE ?1 || '/%') \
                     ORDER BY expires_at_ms DESC \
                     LIMIT 20",
                )
                .map_err(|_| ())?;
            let blocking_claims_raw = stmt
                .query_map(params![path, agent_id, now_ms], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                })
                .map_err(|_| ())?
                .filter_map(|r| r.ok())
                .collect::<Vec<(String, String, i64)>>();
            let mut seen = HashSet::<String>::new();
            let mut blocking_agents = Vec::<String>::new();
            // Keep a deterministic, actionable representative claim path per blocker:
            // prefer the most specific (longest) overlapping claim; tie-break on expiry then path.
            let mut blocking_claim_path_by_agent =
                std::collections::BTreeMap::<String, (String, i64)>::new();
            for (blocker, claim_path, expires_at_ms) in blocking_claims_raw {
                if seen.insert(blocker.clone()) {
                    blocking_agents.push(blocker.clone());
                }
                match blocking_claim_path_by_agent.get_mut(&blocker) {
                    None => {
                        blocking_claim_path_by_agent
                            .insert(blocker, (claim_path, expires_at_ms));
                    }
                    Some((best_path, best_exp)) => {
                        let better = claim_path.len() > best_path.len()
                            || (claim_path.len() == best_path.len()
                                && (expires_at_ms > *best_exp
                                    || (expires_at_ms == *best_exp && claim_path < *best_path)));
                        if better {
                            *best_path = claim_path;
                            *best_exp = expires_at_ms;
                        }
                    }
                }
            }
            // Preserve the old low-noise behavior: cap the list.
            if blocking_agents.len() > 5 {
                blocking_agents.truncate(5);
            }
            let kept_blockers: HashSet<String> = blocking_agents.iter().cloned().collect();
            blocking_claim_path_by_agent.retain(|k, _| kept_blockers.contains(k));

            // Expose all overlapping claim paths per blocker for actionable coordination.
            // This is intentionally redundant with `blocking_agents`: consumers can show both.
            let mut blocking_claims: Vec<Value> = Vec::new();
            let mut blocking_claim_paths_by_agent = serde_json::Map::new();
            if !blocking_agents.is_empty() {
                let mut stmt_blocking = conn
                    .prepare(
                        "SELECT path, expires_at_ms FROM claims \
                         WHERE agent_id = ?1 AND expires_at_ms > ?2 \
                           AND (path = ?3 OR ?3 LIKE path || '/%' OR path LIKE ?3 || '/%') \
                         ORDER BY LENGTH(path) DESC, expires_at_ms DESC, path ASC \
                         LIMIT 20",
                    )
                    .map_err(|_| ())?;
                for blocker in &blocking_agents {
                    let mut paths_for_blocker: Vec<Value> = Vec::new();
                    let rows = stmt_blocking
                        .query_map(params![blocker, now_ms, path], |row| {
                            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                        })
                        .map_err(|_| ())?;
                    for r in rows {
                        let (p, expires_at_ms) = r.map_err(|_| ())?;
                        paths_for_blocker.push(Value::String(p.clone()));
                        blocking_claims.push(json!({
                            "agent_id": blocker,
                            "path": p,
                            "expires_at_ms": expires_at_ms,
                        }));
                    }
                    if !paths_for_blocker.is_empty() {
                        blocking_claim_paths_by_agent
                            .insert(blocker.to_owned(), Value::from(paths_for_blocker));
                    }
                }
            }

            let (decision, reason_code) = if !blocking_agents.is_empty() {
                if strict {
                    ("deny", "claimed_by_other")
                } else {
                    ("warn", "claimed_by_other")
                }
            } else {
                ("allow", "no_conflict")
            };

            let (unread_relevant_updates, unread_prev_cursor, unread_cursor) =
                unread_relevant_updates_for_check(&conn, &agent_id, &path, now_ms)?;

            // If a blocking agent has already posted a relevant update about this path, suppress the
            // generic "Are you working on it?" ping and instead prefer closed-loop acknowledgement.
            let blocking_set: HashSet<&str> = blocking_agents.iter().map(|s| s.as_str()).collect();
            let blockers_with_unread_update: HashSet<String> = unread_relevant_updates
                .iter()
                .filter_map(|u| u.get("agent_id").and_then(|v| v.as_str()).map(|s| s.to_owned()))
                .filter(|a| blocking_set.contains(a.as_str()))
                .collect();

            // Even after an update is surfaced (cursor advanced), we should not regress into spamming
            // the generic ping. Track whether each blocker has communicated about this path at all,
            // and keep suggesting an explicit ack until the requester actually acks.
            let ack_to_me_prefix = format!("@{agent_id}: ack:");
            let resolve_to_me_prefix = format!("@{agent_id}: resolve:");
            let resolved_to_me_prefix = format!("@{agent_id}: resolved:");
            let released_to_me_prefix = format!("@{agent_id}: released:");
            let resolve_to_me_prefix_lower = resolve_to_me_prefix.to_ascii_lowercase();
            let resolved_to_me_prefix_lower = resolved_to_me_prefix.to_ascii_lowercase();
            let released_to_me_prefix_lower = released_to_me_prefix.to_ascii_lowercase();
            let mut blockers_with_any_relevant_update: HashSet<String> = HashSet::new();
            let mut pending_ack_blockers: HashSet<String> = HashSet::new();
            for blocker in &blocking_agents {
                if let Some((_id, created_at_ms, txt)) =
                    last_relevant_update_from_agent(&conn, blocker, &path)?
                {
                    blockers_with_any_relevant_update.insert(blocker.to_owned());
                    if is_explicit_closure_to_me(
                        &txt,
                        &ack_to_me_prefix,
                        &resolve_to_me_prefix_lower,
                        &resolved_to_me_prefix_lower,
                        &released_to_me_prefix_lower,
                    ) {
                        continue;
                    }
                    if !requester_has_acked_since(&conn, &agent_id, blocker, created_at_ms)? {
                        pending_ack_blockers.insert(blocker.to_owned());
                    }
                }
            }

            // Minimal, scriptable action plan: a few commands that wrappers can show or run.
            // Keep it intentionally stringly-typed for now to keep the CLI tiny.
            let mut pinged_blockers: HashSet<String> = HashSet::new();
            let mut action_plan: Vec<String> = if !blocking_agents.is_empty() {
                let mut plan = Vec::<String>::new();
                plan.push(format!("but-engineering-rewrite --agent-id {agent_id} read"));
                for blocker in &blocking_agents {
                    if blockers_with_any_relevant_update.contains(blocker)
                        || blockers_with_unread_update.contains(blocker)
                    {
                        continue;
                    }
                    // Anti-spam: if the requester already pinged this blocker about this path using
                    // the standard "Are you working on it?" template, don't keep suggesting the same
                    // @mention on every subsequent `check` poll.
                    if requester_already_pinged_blocker(&conn, &agent_id, blocker, &path)? {
                        continue;
                    }
                    plan.push(format!(
                        "but-engineering-rewrite --agent-id {agent_id} post \"@{blocker}: I'm about to edit {path}. Are you working on it?\""
                    ));
                    pinged_blockers.insert(blocker.to_owned());
                }
                plan.push(format!(
                    "but-engineering-rewrite --agent-id {agent_id} check --path {path}{}",
                    if strict { " --strict" } else { "" }
                ));
                plan
            } else {
                // For allow decisions, `check` already happened; avoid suggesting a redundant re-check.
                Vec::new()
            };

            // Multi-step coordination is usually a multi-agent process. Provide a low-noise,
            // per-agent action plan to help wrappers drive convergence without guesswork.
            let mut action_plan_by_agent = serde_json::Map::new();
            let mut stmt_agents = conn
                .prepare("SELECT agent_id FROM agent_state ORDER BY agent_id ASC")
                .map_err(|_| ())?;
            let agents = stmt_agents
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|_| ())?
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();

            for a in agents {
                if a == agent_id {
                    continue;
                }

                if let Some((claim_path, _)) = blocking_claim_path_by_agent.get(&a) {
                    action_plan_by_agent.insert(
                        a.clone(),
                        Value::from(vec![
                            format!("but-engineering-rewrite --agent-id {a} read"),
                            format!(
                                "but-engineering-rewrite --agent-id {a} post \"@{agent_id}: I'm holding a claim on {claim_path} (overlaps {path}). ETA update soon.\""
                            ),
                            format!("but-engineering-rewrite --agent-id {a} release --path {claim_path}"),
                        ]),
                    );
                    continue;
                }

                let active_claim_path: Option<String> = conn
                    .query_row(
                        "SELECT path FROM claims WHERE agent_id = ?1 AND expires_at_ms > ?2 ORDER BY expires_at_ms DESC LIMIT 1",
                        params![a, now_ms],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|_| ())?;

                if let Some(p) = active_claim_path {
                    action_plan_by_agent.insert(
                        a.clone(),
                        Value::from(vec![format!(
                            "but-engineering-rewrite --agent-id {a} post \"@{agent_id}: FYI I'm working on {p}; not touching {path}.\""
                        )]),
                    );
                }
            }

            // Coordination compliance closure semantics:
            // If `check` surfaces an unread relevant update, propose a single explicit "ack" per
            // update-author (excluding agents already pinged by the conflict plan).
            //
            // Also keep suggesting an ack for blockers who have already communicated about this path
            // until the requester actually posts an ack (prevents "ping regression" after cursor advances).
            if !unread_relevant_updates.is_empty() || !pending_ack_blockers.is_empty() {
                use std::collections::BTreeSet;
                let pinged_set: HashSet<&str> = pinged_blockers.iter().map(|s| s.as_str()).collect();

                // Avoid "ack ping-pong" loops: if the unread update is itself an ack directed
                // at this agent, do not suggest acknowledging it back.
                //
                // Similarly, treat `@<me>: resolve:` / `resolved:` as explicit closure and do not
                // suggest acknowledging it back.
                //
                // Keep this intentionally simple and prefix-based so it also covers human-written
                // ack variants (not just the exact auto-ack template).
                let mut ack_agents: BTreeSet<String> = BTreeSet::new();
                for u in &unread_relevant_updates {
                    if let Some(a) = u.get("agent_id").and_then(|v| v.as_str()) {
                        if a == agent_id {
                            continue;
                        }
                        if pinged_set.contains(a) {
                            continue;
                        }
                        let body_text = u.get("body").and_then(|b| match b {
                            Value::Object(m) => m.get("text").and_then(|v| v.as_str()),
                            Value::String(s) => Some(s.as_str()),
                            _ => None,
                        });
                        if body_text.is_some_and(|t| {
                            is_explicit_closure_to_me(
                                t,
                                &ack_to_me_prefix,
                                &resolve_to_me_prefix_lower,
                                &resolved_to_me_prefix_lower,
                                &released_to_me_prefix_lower,
                            )
                        }) {
                            continue;
                        }
                        ack_agents.insert(a.to_owned());
                    }
                }

                for a in &pending_ack_blockers {
                    if a == &agent_id {
                        continue;
                    }
                    if pinged_set.contains(a.as_str()) {
                        continue;
                    }
                    ack_agents.insert(a.to_owned());
                }

                for a in ack_agents {
                    let cmd = format!(
                        "but-engineering-rewrite --agent-id {agent_id} post \"@{a}: ack: saw your update re {path}.\""
                    );
                    // Insert before a trailing re-check if present, to keep the plan "read/ack/check".
                    if action_plan
                        .last()
                        .is_some_and(|s| s.contains(" check --path "))
                    {
                        let idx = action_plan.len().saturating_sub(1);
                        action_plan.insert(idx, cmd);
                    } else {
                        action_plan.push(cmd);
                    }
                }
            }

            action_plan_by_agent.insert(
                agent_id.clone(),
                Value::from(action_plan.iter().cloned().collect::<Vec<_>>()),
            );

            let dependency_hints = dependency_hints_for_check(&conn, &agent_id)?;
            let stale_agents = stale_agents_for_blockers(
                &conn,
                &agent_id,
                &blocking_agents,
                &path,
                now_ms,
            )?;

            // Coordination usefulness: include the current status/plan of blocking agents
            // so callers don't need an extra `agents` round-trip when they hit a conflict.
            let mut blocking_agents_state: Vec<Value> = Vec::new();
            if !blocking_agents.is_empty() {
                let mut stmt = conn
                    .prepare(
                        "SELECT status, plan, updated_at_ms FROM agent_state WHERE agent_id = ?1",
                    )
                    .map_err(|_| ())?;
                for blocker in &blocking_agents {
                    let row: Option<(Option<String>, Option<String>, i64)> = stmt
                        .query_row(params![blocker], |row| {
                            Ok((
                                row.get::<_, Option<String>>(0)?,
                                row.get::<_, Option<String>>(1)?,
                                row.get::<_, i64>(2)?,
                            ))
                        })
                        .optional()
                        .map_err(|_| ())?;
                    if let Some((status, plan, updated_at_ms)) = row {
                        blocking_agents_state.push(json!({
                            "agent_id": blocker,
                            "status": status,
                            "plan": plan,
                            "updated_at_ms": updated_at_ms,
                        }));
                    } else {
                        blocking_agents_state.push(json!({
                            "agent_id": blocker,
                            "status": Value::Null,
                            "plan": Value::Null,
                            "updated_at_ms": 0,
                        }));
                    }
                }
            }

            let out = json!({
                "decision": decision,
                "reason_code": reason_code,
                "blocking_agents": blocking_agents,
                "blocking_claims": blocking_claims,
                "blocking_claim_paths_by_agent": Value::Object(blocking_claim_paths_by_agent),
                "blocking_agents_state": blocking_agents_state,
                "action_plan": action_plan,
                "action_plan_by_agent": Value::Object(action_plan_by_agent),
                "dependency_hints": dependency_hints,
                "stale_agents": stale_agents,
                "unread_relevant_updates_label": "unread relevant updates since last seen",
                "unread_relevant_updates_prev_cursor": unread_prev_cursor,
                "unread_relevant_updates_cursor": unread_cursor,
                "unread_relevant_updates": unread_relevant_updates,
            });
            print_json(&out.to_string());
        }
        Cmd::Post { message } => {
            // Minimal channel support: persist plain text messages so other agents can read them.
            // This keeps the harness realistic (coordination requires a shared transcript).
            let now_ms = now_unix_ms()?;
            let body_json = json!({ "text": message }).to_string();
            conn.execute(
                "INSERT INTO messages(created_at_ms, agent_id, kind, body_json) VALUES (?1, ?2, 'message', ?3)",
                params![now_ms, agent_id, body_json],
            )
            .map_err(|_| ())?;
            print_json(r#"{"ok":true}"#);
        }
        Cmd::PostTyped { kind, json: body_json } => {
            let now_ms = now_unix_ms()?;
            let v: Value = serde_json::from_str(&body_json).map_err(|_| ())?;
            match kind.as_str() {
                "discovery" => {
                    // Minimal validation for high-signal discovery payloads.
                    // This is intentionally shallow: enough to gate propagation and avoid obviously broken posts.
                    validate_discovery_payload(&v)?;
                }
                "declaration" | "intent" => {
                    validate_surface_payload(&v)?;
                }
                _ => return Err(()),
            }
            conn.execute(
                "INSERT INTO messages(created_at_ms, agent_id, kind, body_json) VALUES (?1, ?2, ?3, ?4)",
                params![now_ms, agent_id, kind, body_json],
            )
            .map_err(|_| ())?;
            print_json(r#"{"ok":true}"#);
        }
        Cmd::Read { kind } => {
            // Default `read` to the shared channel transcript: it's the most common
            // operation and keeps the CLI ergonomic (no need to remember `--type message`).
            let kind = kind.unwrap_or_else(|| "message".to_owned());
            if kind == "discovery" || kind == "all" {
                let mut discoveries: Vec<Value> = Vec::new();
                let mut next_steps: Vec<Value> = Vec::new();

                let mut stmt = conn
                    .prepare(
                        "SELECT created_at_ms, agent_id, body_json FROM messages WHERE kind = 'discovery' ORDER BY id ASC",
                    )
                    .map_err(|_| ())?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                        ))
                    })
                    .map_err(|_| ())?;

                for r in rows {
                    let (created_at_ms, agent, body_json) = r.map_err(|_| ())?;
                    let parsed: Value =
                        serde_json::from_str(&body_json).unwrap_or(Value::String(body_json));
                    let mut obj = match parsed {
                        Value::Object(m) => Value::Object(m),
                        other => json!({ "raw": other }),
                    };

                    // Attach provenance.
                    if let Value::Object(m) = &mut obj {
                        m.insert("agent_id".to_owned(), Value::String(agent));
                        m.insert("created_at_ms".to_owned(), Value::from(created_at_ms));
                    }

                    // Derive a minimal actionable step, if present.
                    if let Some(cmd) = obj
                        .get("suggested_action")
                        .and_then(|sa| sa.get("cmd"))
                        .and_then(|c| c.as_str())
                    {
                        next_steps.push(json!({"kind":"run","cmd":cmd}));
                    }

                    discoveries.push(obj);
                }

                let out = json!({
                    "ok": true,
                    "kind": kind,
                    "discoveries": discoveries,
                    "next_steps": next_steps,
                });
                print_json(&out.to_string());
            } else if kind == "declaration" || kind == "intent" || kind == "message" || kind == "block" {
                // Minimal inspection/debug surface: list stored structured payloads with provenance.
                let mut stmt = conn
                    .prepare(
                        "SELECT created_at_ms, agent_id, body_json FROM messages WHERE kind = ?1 ORDER BY id ASC",
                    )
                    .map_err(|_| ())?;
                let rows = stmt
                    .query_map(params![kind], |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                        ))
                    })
                    .map_err(|_| ())?;

                let mut messages: Vec<Value> = Vec::new();
                for r in rows {
                    let (created_at_ms, agent, body_json) = r.map_err(|_| ())?;
                    let parsed: Value =
                        serde_json::from_str(&body_json).unwrap_or(Value::String(body_json));
                    let mut obj = match parsed {
                        Value::Object(m) => Value::Object(m),
                        other => json!({ "raw": other }),
                    };
                    if let Value::Object(m) = &mut obj {
                        m.insert("agent_id".to_owned(), Value::String(agent));
                        m.insert("created_at_ms".to_owned(), Value::from(created_at_ms));
                    }
                    messages.push(obj);
                }

                let out = json!({
                    "ok": true,
                    "kind": kind,
                    "messages": messages,
                });
                print_json(&out.to_string());
            } else {
                let out = json!({
                    "ok": true,
                    "kind": kind,
                    "messages": [],
                });
                print_json(&out.to_string());
            }
        }
        Cmd::Brief { kind, all } => {
            let kind = kind.unwrap_or_else(|| "discovery".to_owned());

            let (discoveries, next_steps) = load_discoveries_and_next_steps(&conn, &kind, all)?;

            let out = json!({
                "ok": true,
                "mode": "brief",
                "kind": kind,
                "discoveries": discoveries,
                "next_steps": next_steps,
            });
            print_json(&out.to_string());
        }
        Cmd::Digest { kind, all } => {
            let kind = kind.unwrap_or_else(|| "discovery".to_owned());
            let (discoveries, next_steps) = load_discoveries_and_next_steps(&conn, &kind, all)?;

            // Digest: keep the discovery list intentionally smaller than brief.
            let discoveries = discoveries
                .into_iter()
                .map(|d| match d {
                    Value::Object(mut m) => {
                        let title = m.remove("title");
                        let agent_id = m.remove("agent_id");
                        json!({
                            "title": title,
                            "agent_id": agent_id,
                        })
                    }
                    other => other,
                })
                .collect::<Vec<_>>();

            let out = json!({
                "ok": true,
                "mode": "digest",
                "kind": kind,
                "discoveries": discoveries,
                "next_steps": next_steps,
            });
            print_json(&out.to_string());
        }
        Cmd::Status { value } => {
            let now_ms = now_unix_ms()?;
            ensure_agent_row(&conn, &agent_id, now_ms)?;
            conn.execute(
                "UPDATE agent_state SET status = ?2, updated_at_ms = ?3 WHERE agent_id = ?1",
                params![agent_id, value, now_ms],
            )
            .map_err(|_| ())?;
            print_json(r#"{"ok":true}"#);
        }
        Cmd::Plan { value } => {
            let now_ms = now_unix_ms()?;
            ensure_agent_row(&conn, &agent_id, now_ms)?;
            conn.execute(
                "UPDATE agent_state SET plan = ?2, updated_at_ms = ?3 WHERE agent_id = ?1",
                params![agent_id, value, now_ms],
            )
            .map_err(|_| ())?;
            print_json(r#"{"ok":true}"#);
        }
        Cmd::Agents => {
            let mut stmt = conn
                .prepare(
                    "SELECT agent_id, status, plan, updated_at_ms FROM agent_state ORDER BY agent_id ASC",
                )
                .map_err(|_| ())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                })
                .map_err(|_| ())?;

            let mut agents: Vec<Value> = Vec::new();
            for r in rows {
                let (agent_id, status, plan, updated_at_ms) = r.map_err(|_| ())?;
                agents.push(json!({
                    "agent_id": agent_id,
                    "status": status,
                    "plan": plan,
                    "updated_at_ms": updated_at_ms,
                }));
            }

            let out = json!({
                "ok": true,
                "agents": agents,
            });
            print_json(&out.to_string());
        }
        Cmd::Done { summary } => {
            let now_ms = now_unix_ms()?;

            // Release all leases for this agent.
            let released: i64 = conn
                .query_row(
                    "SELECT COUNT(1) FROM claims WHERE agent_id = ?1",
                    params![agent_id],
                    |row| row.get(0),
                )
                .map_err(|_| ())?;
            conn.execute("DELETE FROM claims WHERE agent_id = ?1", params![agent_id])
                .map_err(|_| ())?;

            // Clear ephemeral agent metadata.
            ensure_agent_row(&conn, &agent_id, now_ms)?;
            conn.execute(
                "UPDATE agent_state SET status = NULL, plan = NULL, updated_at_ms = ?2 WHERE agent_id = ?1",
                params![agent_id, now_ms],
            )
            .map_err(|_| ())?;

            // Post a completion summary to the shared channel so others can see the work wrapped up.
            let body_json = json!({ "text": format!("DONE: {summary}") }).to_string();
            conn.execute(
                "INSERT INTO messages(created_at_ms, agent_id, kind, body_json) VALUES (?1, ?2, 'message', ?3)",
                params![now_ms, agent_id, body_json],
            )
            .map_err(|_| ())?;

            let out = json!({
                "ok": true,
                "released_claims": released,
                "cleared": ["status", "plan"],
            });
            print_json(&out.to_string());
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

fn normalize_claim_path(s: &str) -> String {
    // Minimal, string-based normalization for harness use:
    // - Drop a leading "./"
    // - Drop trailing "/" (so claiming "src/" and checking "src/app.txt" overlaps cleanly)
    // - Collapse "." and ".." segments to avoid false positives like "src/../README.md"
    //   being treated as overlapping "src/".
    let mut out = s.trim().to_owned();
    while out.starts_with("./") {
        out = out.trim_start_matches("./").to_owned();
    }
    while out.ends_with('/') && out != "/" {
        out.pop();
    }

    let mut parts: Vec<&str> = Vec::new();
    for p in out.split('/') {
        if p.is_empty() || p == "." {
            continue;
        }
        if p == ".." {
            if let Some(last) = parts.last() {
                if *last != ".." {
                    parts.pop();
                    continue;
                }
            }
            parts.push("..");
            continue;
        }
        parts.push(p);
    }

    parts.join("/")
}

fn load_discoveries_and_next_steps(
    conn: &Connection,
    kind: &str,
    all: bool,
) -> Result<(Vec<Value>, Vec<Value>), ()> {
    let mut discoveries: Vec<Value> = Vec::new();
    let mut next_steps: Vec<Value> = Vec::new();

    if kind == "discovery" || kind == "all" {
        let mut stmt = conn
            .prepare("SELECT agent_id, body_json FROM messages WHERE kind = 'discovery' ORDER BY id ASC")
            .map_err(|_| ())?;
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|_| ())?;

        for r in rows {
            let (agent, body_json) = r.map_err(|_| ())?;
            let parsed: Value = serde_json::from_str(&body_json).unwrap_or(Value::String(body_json));
            let mut obj = match parsed {
                Value::Object(m) => Value::Object(m),
                other => json!({ "raw": other }),
            };

            if let Value::Object(m) = &mut obj {
                m.insert("agent_id".to_owned(), Value::String(agent));
            }

            // High-signal gate: only propagate discoveries explicitly marked as high-signal.
            // This keeps the default channel "quiet" and matches the "share only valuable findings" intent.
            if !all && !is_high_signal_discovery(&obj) {
                continue;
            }

            // Derive minimal actionable steps from stored payload.
            if let Some(cmd) = obj
                .get("suggested_action")
                .and_then(|sa| sa.get("cmd"))
                .and_then(|c| c.as_str())
            {
                next_steps.push(json!({"kind":"run","cmd":cmd}));
            }

            discoveries.push(obj);
        }
    }

    Ok((discoveries, next_steps))
}

fn is_high_signal_discovery(v: &Value) -> bool {
    v.get("signal")
        .and_then(|s| s.as_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("high"))
}

fn validate_surface_payload(v: &Value) -> Result<(), ()> {
    // Accept only object payloads for structured surface declarations/intents.
    let obj = v.as_object().ok_or(())?;

    let scope_ok = obj
        .get("scope")
        .and_then(|s| s.as_str())
        .is_some_and(|s| !s.trim().is_empty());
    if !scope_ok {
        return Err(());
    }

    let tags_ok = obj
        .get("tags")
        .and_then(|t| t.as_array())
        .is_some_and(|a| !a.is_empty() && a.iter().all(|v| v.as_str().is_some()));
    if !tags_ok {
        return Err(());
    }

    let surface_ok = obj
        .get("surface")
        .and_then(|t| t.as_array())
        .is_some_and(|a| !a.is_empty() && a.iter().all(|v| v.as_str().is_some()));
    if !surface_ok {
        return Err(());
    }

    Ok(())
}

fn validate_discovery_payload(v: &Value) -> Result<(), ()> {
    // Accept only object payloads for structured discoveries.
    let obj = v.as_object().ok_or(())?;

    let title_ok = obj
        .get("title")
        .and_then(|t| t.as_str())
        .is_some_and(|t| !t.trim().is_empty());
    if !title_ok {
        return Err(());
    }

    let evidence_ok = obj
        .get("evidence")
        .and_then(|e| e.as_array())
        .is_some_and(|a| !a.is_empty());
    if !evidence_ok {
        return Err(());
    }

    let suggested_cmd_ok = obj
        .get("suggested_action")
        .and_then(|sa| sa.get("cmd"))
        .and_then(|c| c.as_str())
        .is_some_and(|c| !c.trim().is_empty());
    if !suggested_cmd_ok {
        return Err(());
    }

    Ok(())
}

fn dependency_hints_for_check(conn: &Connection, agent_id: &str) -> Result<Vec<Value>, ()> {
    // Heuristic:
    // - If agent B has a latest intent with `surface[]`
    // - And the intent/declaration scopes match (to avoid cross-component token collisions)
    // - And some other agent A posted a declaration tagged as API-ish
    // - And intent.surface intersects declaration.surface
    // Then emit an actionable hint (no blocking).

    let intent_json: Option<String> = conn
        .query_row(
            "SELECT body_json FROM messages WHERE kind = 'intent' AND agent_id = ?1 ORDER BY id DESC LIMIT 1",
            params![agent_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| ())?;

    let Some(intent_json) = intent_json else {
        return Ok(Vec::new());
    };
    let intent_v: Value = serde_json::from_str(&intent_json).map_err(|_| ())?;
    let intent_scope = intent_v
        .get("scope")
        .and_then(|s| s.as_str())
        .map(|s| s.to_owned());
    let intent_surface: Vec<String> = intent_v
        .get("surface")
        .and_then(|s| s.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_owned())).collect())
        .unwrap_or_default();

    if intent_surface.is_empty() {
        return Ok(Vec::new());
    }

    let mut stmt = conn
        .prepare(
            "SELECT agent_id, body_json FROM messages WHERE kind = 'declaration' AND agent_id <> ?1 ORDER BY id DESC",
        )
        .map_err(|_| ())?;
    let rows = stmt
        .query_map(params![agent_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|_| ())?;

    let mut hints: Vec<Value> = Vec::new();
    // Dedupe: the same provider may post multiple declarations for the same scope while iterating.
    // Keep only the newest hint per (provider_agent_id, scope) to avoid noisy repeats.
    let mut seen_provider_scope: HashSet<(String, String)> = HashSet::new();
    for r in rows {
        let (provider_agent_id, decl_json) = r.map_err(|_| ())?;
        let decl_v: Value = serde_json::from_str(&decl_json).map_err(|_| ())?;
        let decl_obj = decl_v.as_object().ok_or(())?;

        let scope = decl_obj
            .get("scope")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_owned();

        if let Some(ref intent_scope) = intent_scope {
            if scope != *intent_scope {
                continue;
            }
        }

        let tags: Vec<String> = decl_obj
            .get("tags")
            .and_then(|t| t.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_owned())).collect())
            .unwrap_or_default();

        // Avoid substring false positives (e.g. "capistrano" contains "api").
        // Treat a tag as API-ish only if it contains an "api" segment.
        let is_api_decl = tags.iter().any(|t| tag_has_api_segment(t));
        if !is_api_decl {
            continue;
        }

        let decl_surface: Vec<String> = decl_obj
            .get("surface")
            .and_then(|s| s.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_owned())).collect())
            .unwrap_or_default();
        if decl_surface.is_empty() {
            continue;
        }

        let mut overlap: Vec<String> = Vec::new();
        for tok in &decl_surface {
            if intent_surface.iter().any(|t| t == tok) {
                overlap.push(tok.clone());
            }
        }
        if overlap.is_empty() {
            continue;
        }

        if !seen_provider_scope.insert((provider_agent_id.clone(), scope.clone())) {
            continue;
        }

        let why = format!(
            "intent.surface intersects declaration.surface on token(s): {} (declaration tagged api). Coordinate before consuming/changing it.",
            overlap.join(", ")
        );
        let suggested_cmd = format!(
            "but-engineering-rewrite --agent-id {agent_id} post \"@{provider_agent_id}: I'm about to consume {scope} (overlap: {tok}). Are you changing the contract? Any migration notes?\"",
            tok = overlap.get(0).cloned().unwrap_or_else(|| "unknown".to_owned())
        );

        hints.push(json!({
            "kind": "dependency_hint",
            "provider_agent_id": provider_agent_id,
            "scope": scope,
            "tags": tags,
            "overlap_tokens": overlap,
            "why": why,
            "next_step": {
                "kind": "ask",
                "suggested_cmd": suggested_cmd,
            }
        }));
    }

    Ok(hints)
}

fn tag_has_api_segment(tag: &str) -> bool {
    tag.split(|c: char| !c.is_ascii_alphanumeric())
        .any(|seg| seg.eq_ignore_ascii_case("api"))
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
         CREATE TABLE IF NOT EXISTS agent_state (\
            agent_id TEXT PRIMARY KEY,\
            status TEXT,\
            plan TEXT,\
            updated_at_ms INTEGER NOT NULL\
         );\
         CREATE TABLE IF NOT EXISTS messages (\
            id INTEGER PRIMARY KEY AUTOINCREMENT,\
            created_at_ms INTEGER NOT NULL,\
            agent_id TEXT NOT NULL,\
            kind TEXT NOT NULL,\
            body_json TEXT NOT NULL\
         );\
         CREATE TABLE IF NOT EXISTS agent_cursors (\
            agent_id TEXT NOT NULL,\
            topic TEXT NOT NULL,\
            last_seen_msg_id INTEGER NOT NULL,\
            updated_at_ms INTEGER NOT NULL,\
            PRIMARY KEY(agent_id, topic)\
         );\
        ",
    )
    .map_err(|_| ())?;
    Ok(())
}

fn coord_stale_threshold_ms() -> i64 {
    // Harness-controlled staleness threshold for coordination surfaces.
    // Keep a sensible default so wrappers can opt in without configuration.
    let default_s: i64 = 15 * 60;
    let s = std::env::var("COORD_STALE_SECONDS")
        .ok()
        .and_then(|v| v.trim().parse::<i64>().ok())
        .filter(|v| *v >= 0)
        .unwrap_or(default_s);
    s.saturating_mul(1000)
}

fn requester_already_pinged_blocker(
    conn: &Connection,
    requester_agent_id: &str,
    blocker_agent_id: &str,
    path: &str,
) -> Result<bool, ()> {
    let prefix = format!("@{blocker_agent_id}:");
    let mut stmt = conn
        .prepare(
            "SELECT body_json FROM messages \
             WHERE agent_id = ?1 AND kind = 'message' \
             ORDER BY id DESC \
             LIMIT 200",
        )
        .map_err(|_| ())?;
    let rows = stmt
        .query_map(params![requester_agent_id], |row| row.get::<_, String>(0))
        .map_err(|_| ())?;
    for r in rows {
        let body_json = r.map_err(|_| ())?;
        let v: Value = serde_json::from_str(&body_json).unwrap_or(Value::String(body_json));
        let text = v
            .get("text")
            .and_then(|t| t.as_str())
            .or_else(|| v.as_str())
            .unwrap_or("");
        if text.contains(&prefix)
            && text.contains(path)
            && text.contains("Are you working on it?")
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn stale_agents_for_blockers(
    conn: &Connection,
    requester_agent_id: &str,
    blockers: &[String],
    path: &str,
    now_ms: i64,
) -> Result<Vec<Value>, ()> {
    let thresh_ms = coord_stale_threshold_ms();
    if thresh_ms <= 0 || blockers.is_empty() {
        return Ok(Vec::new());
    }

    let mut out: Vec<Value> = Vec::new();
    for a in blockers {
        let updated_at_ms: Option<i64> = conn
            .query_row(
                "SELECT updated_at_ms FROM agent_state WHERE agent_id = ?1",
                params![a],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| ())?;
        let Some(updated_at_ms) = updated_at_ms else {
            continue;
        };

        let stale_for_ms = now_ms.saturating_sub(updated_at_ms);
        let is_stale = stale_for_ms >= thresh_ms;
        if !is_stale {
            continue;
        }

        let suggested_cmd = format!(
            "but-engineering-rewrite --agent-id {requester_agent_id} post \"@{a}: can you update your status and plan for {path}? (stale)\""
        );

        out.push(json!({
            "kind": "stale_agent",
            "agent_id": a,
            "updated_at_ms": updated_at_ms,
            "stale_for_ms": stale_for_ms,
            "threshold_ms": thresh_ms,
            "is_stale": true,
            "suggested_cmd": suggested_cmd,
        }));
    }

    Ok(out)
}

fn unread_relevant_updates_for_check(
    conn: &Connection,
    agent_id: &str,
    path: &str,
    now_ms: i64,
) -> Result<(Vec<Value>, i64, i64), ()> {
    let topic = format!("check_path:{path}");

    let prev_cursor: i64 = conn
        .query_row(
            "SELECT last_seen_msg_id FROM agent_cursors WHERE agent_id = ?1 AND topic = ?2",
            params![agent_id, topic],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| ())?
        .unwrap_or(0);

    // Heuristic relevance: surface updates that mention either the exact path being checked
    // or an overlapping parent directory (any ancestor). This matches real-world coordination
    // where agents often talk about directories ("src/") rather than exact files ("src/app.txt").
    let needles = relevant_needles_for_path(path);

    let mut stmt = conn
        .prepare(
            "SELECT id, created_at_ms, agent_id, kind, body_json FROM messages \
             WHERE id > ?1 AND agent_id <> ?2 AND kind IN ('message','discovery') \
             ORDER BY id ASC \
             LIMIT 500",
        )
        .map_err(|_| ())?;

    let rows = stmt
        .query_map(params![prev_cursor, agent_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|_| ())?;

    let mut updates: Vec<Value> = Vec::new();
    let mut max_id = prev_cursor;
    for r in rows {
        let (id, created_at_ms, from_agent, kind, body_json) = r.map_err(|_| ())?;
        max_id = max_id.max(id);

        let body_v: Value =
            serde_json::from_str(&body_json).unwrap_or(Value::String(body_json.clone()));

        let txt = extract_message_text(&body_v, &body_json);
        if !needles.iter().any(|n| contains_path_token(&txt, n)) {
            continue;
        }
        if updates.len() >= 20 {
            continue;
        }

        updates.push(json!({
            "id": id,
            "created_at_ms": created_at_ms,
            "agent_id": from_agent,
            "kind": kind,
            "body": body_v,
        }));
    }

    let new_cursor = max_id;
    if new_cursor != prev_cursor {
        conn.execute(
            "INSERT INTO agent_cursors(agent_id, topic, last_seen_msg_id, updated_at_ms) VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(agent_id, topic) DO UPDATE SET last_seen_msg_id = excluded.last_seen_msg_id, updated_at_ms = excluded.updated_at_ms",
            params![agent_id, topic, new_cursor, now_ms],
        )
        .map_err(|_| ())?;
    }

    Ok((updates, prev_cursor, new_cursor))
}

fn ensure_agent_row(conn: &Connection, agent_id: &str, now_ms: i64) -> Result<(), ()> {
    conn.execute(
        "INSERT INTO agent_state(agent_id, status, plan, updated_at_ms) VALUES (?1, NULL, NULL, ?2) \
         ON CONFLICT(agent_id) DO UPDATE SET updated_at_ms = excluded.updated_at_ms",
        params![agent_id, now_ms],
    )
    .map_err(|_| ())?;
    Ok(())
}

fn touch_agent(conn: &Connection, agent_id: &str) -> Result<(), ()> {
    let now_ms = now_unix_ms()?;
    ensure_agent_row(conn, agent_id, now_ms)?;
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
            "claim"
            | "release"
            | "claims"
            | "check"
            | "post"
            | "read"
            | "brief"
            | "digest"
            | "status"
            | "plan"
            | "agents"
            | "done"
            | "eval" => {
                sub = Some(a)
            }
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
        "release" => {
            let path = parse_release_args(&rest)?;
            Cmd::Release { path }
        }
        "claims" => {
            let path_prefix = parse_claims_args(&rest)?;
            let path_prefix = path_prefix.map(|p| normalize_claim_path(&p));
            Cmd::Claims { path_prefix }
        }
        "check" => {
            let (path, strict) = parse_check_args(&rest)?;
            Cmd::Check { path, strict }
        }
        "post" => {
            if rest.is_empty() {
                return Err(());
            }
            if rest.get(0).map(String::as_str) == Some("--type") {
                // Minimal structured posts: `post --type discovery --json <payload>`
                let (kind, body_json) = parse_post_structured_args(&rest)?;
                Cmd::PostTyped { kind, json: body_json }
            } else {
                Cmd::Post {
                    message: rest.join(" "),
                }
            }
        }
        "read" => {
            let kind = parse_read_args(&rest)?;
            Cmd::Read { kind }
        }
        "brief" => {
            let (kind, all) = parse_brief_digest_args(&rest)?;
            Cmd::Brief { kind, all }
        }
        "digest" => {
            let (kind, all) = parse_brief_digest_args(&rest)?;
            Cmd::Digest { kind, all }
        }
        "status" => {
            let value = parse_free_text_or_clear(&rest)?;
            Cmd::Status { value }
        }
        "plan" => {
            let value = parse_free_text_or_clear(&rest)?;
            Cmd::Plan { value }
        }
        "agents" => {
            if !rest.is_empty() {
                return Err(());
            }
            Cmd::Agents
        }
        "done" => {
            if rest.is_empty() {
                return Err(());
            }
            Cmd::Done {
                summary: rest.join(" "),
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

fn parse_free_text_or_clear(args: &[String]) -> Result<Option<String>, ()> {
    if args.is_empty() {
        return Err(());
    }
    if args.len() == 1 && args[0] == "--clear" {
        return Ok(None);
    }
    if args.iter().any(|a| a == "--clear") {
        return Err(());
    }
    Ok(Some(args.join(" ")))
}

fn parse_claims_args(args: &[String]) -> Result<Option<String>, ()> {
    if args.is_empty() {
        return Ok(None);
    }
    if args.len() == 2 && args[0] == "--path-prefix" {
        let v = args[1].trim();
        if v.is_empty() {
            return Err(());
        }
        return Ok(Some(v.to_owned()));
    }
    Err(())
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

fn parse_release_args(args: &[String]) -> Result<String, ()> {
    let mut path: Option<String> = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--path" => {
                i += 1;
                path = Some(args.get(i).ok_or(())?.clone());
            }
            _ => return Err(()),
        }
        i += 1;
    }
    Ok(path.ok_or(())?)
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

fn parse_post_structured_args(args: &[String]) -> Result<(String, String), ()> {
    let mut kind: Option<String> = None;
    let mut body_json: Option<String> = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--type" => {
                i += 1;
                kind = Some(args.get(i).ok_or(())?.clone());
            }
            "--json" => {
                i += 1;
                body_json = Some(args.get(i).ok_or(())?.clone());
            }
            _ => return Err(()),
        }
        i += 1;
    }
    Ok((kind.ok_or(())?, body_json.ok_or(())?))
}

fn parse_read_args(args: &[String]) -> Result<Option<String>, ()> {
    if args.is_empty() {
        return Ok(None);
    }
    let mut kind: Option<String> = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--type" => {
                i += 1;
                kind = Some(args.get(i).ok_or(())?.clone());
            }
            _ => return Err(()),
        }
        i += 1;
    }
    Ok(kind)
}

fn parse_brief_digest_args(args: &[String]) -> Result<(Option<String>, bool), ()> {
    if args.is_empty() {
        return Ok((None, false));
    }
    let mut kind: Option<String> = None;
    let mut all = false;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--type" => {
                i += 1;
                kind = Some(args.get(i).ok_or(())?.clone());
            }
            "--all" => {
                all = true;
            }
            _ => return Err(()),
        }
        i += 1;
    }
    Ok((kind, all))
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
