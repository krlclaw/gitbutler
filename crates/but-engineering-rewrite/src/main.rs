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

            let rows = if let Some(prefix) = &path_prefix {
                stmt.query_map(params![now_ms, prefix], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                })
                .map_err(|_| ())?
            } else {
                stmt.query_map(params![now_ms], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                })
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
                    "SELECT agent_id FROM claims \
                     WHERE agent_id <> ?2 AND expires_at_ms > ?3 \
                       AND (path = ?1 OR ?1 LIKE path || '/%' OR path LIKE ?1 || '/%') \
                     ORDER BY expires_at_ms DESC \
                     LIMIT 5",
                )
                .map_err(|_| ())?;
            let blocking_agents_raw = stmt
                .query_map(params![path, agent_id, now_ms], |row| row.get::<_, String>(0))
                .map_err(|_| ())?
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();
            let mut seen = HashSet::<String>::new();
            let blocking_agents = blocking_agents_raw
                .into_iter()
                .filter(|a| seen.insert(a.clone()))
                .collect::<Vec<_>>();

            let (decision, reason_code) = if !blocking_agents.is_empty() {
                if strict {
                    ("deny", "claimed_by_other")
                } else {
                    ("warn", "claimed_by_other")
                }
            } else {
                ("allow", "no_conflict")
            };

            // Minimal, scriptable action plan: a few commands that wrappers can show or run.
            // Keep it intentionally stringly-typed for now to keep the CLI tiny.
            let action_plan: Vec<String> = if !blocking_agents.is_empty() {
                let mut plan = Vec::<String>::new();
                plan.push(format!("but-engineering-rewrite --agent-id {agent_id} read"));
                for blocker in &blocking_agents {
                    plan.push(format!(
                        "but-engineering-rewrite --agent-id {agent_id} post \"@{blocker}: I'm about to edit {path}. Are you working on it?\""
                    ));
                }
                plan.push(format!(
                    "but-engineering-rewrite --agent-id {agent_id} check --path {path}{}",
                    if strict { " --strict" } else { "" }
                ));
                plan
            } else {
                vec![format!("but-engineering-rewrite --agent-id {agent_id} check --path {path}")]
            };

            let dependency_hints = dependency_hints_for_check(&conn, &agent_id)?;
            let out = json!({
                "decision": decision,
                "reason_code": reason_code,
                "blocking_agents": blocking_agents,
                "action_plan": action_plan,
                "dependency_hints": dependency_hints,
            });
            print_json(&out.to_string());
        }
        Cmd::Post { message: _ } => {
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
                        "SELECT agent_id, body_json FROM messages WHERE kind = 'discovery' ORDER BY id ASC",
                    )
                    .map_err(|_| ())?;
                let rows = stmt
                    .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
                    .map_err(|_| ())?;

                for r in rows {
                    let (agent, body_json) = r.map_err(|_| ())?;
                    let parsed: Value =
                        serde_json::from_str(&body_json).unwrap_or(Value::String(body_json));
                    let mut obj = match parsed {
                        Value::Object(m) => Value::Object(m),
                        other => json!({ "raw": other }),
                    };

                    // Attach provenance.
                    if let Value::Object(m) = &mut obj {
                        m.insert("agent_id".to_owned(), Value::String(agent));
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
                        "SELECT agent_id, body_json FROM messages WHERE kind = ?1 ORDER BY id ASC",
                    )
                    .map_err(|_| ())?;
                let rows = stmt
                    .query_map(params![kind], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })
                    .map_err(|_| ())?;

                let mut messages: Vec<Value> = Vec::new();
                for r in rows {
                    let (agent, body_json) = r.map_err(|_| ())?;
                    let parsed: Value =
                        serde_json::from_str(&body_json).unwrap_or(Value::String(body_json));
                    let mut obj = match parsed {
                        Value::Object(m) => Value::Object(m),
                        other => json!({ "raw": other }),
                    };
                    if let Value::Object(m) = &mut obj {
                        m.insert("agent_id".to_owned(), Value::String(agent));
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
    let mut out = s.trim().to_owned();
    while out.starts_with("./") {
        out = out.trim_start_matches("./").to_owned();
    }
    while out.ends_with('/') && out != "/" {
        out.pop();
    }
    out
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
        ",
    )
    .map_err(|_| ())?;
    Ok(())
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
