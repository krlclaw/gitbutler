// Full(ish) MARBLE-style scored scenario.
// Returns a numeric score (0..1) for promptfoo compatibility.
// Also attaches a detailed breakdown onto globalThis.__butScoreBreakdown for report scripts.

type ProviderCommand = { command?: unknown; failed?: unknown; eventIndex?: unknown };

type ProviderMessage = { agent_id?: unknown; content?: unknown };

type CoordinationState = {
  messages?: ProviderMessage[];
  claims?: Array<{ agent_id?: unknown; path?: unknown; expires_at_ms?: unknown }>;
  discoveries?: ProviderMessage[];
};

type WatchedFile = { path?: unknown; changed?: unknown };

type ProviderOutput = {
  commands?: ProviderCommand[];
  coordinationState?: CoordinationState;
  watchedFiles?: WatchedFile[];
  resultMeta?: { numTurns?: unknown; subtype?: unknown; isError?: unknown; durationMs?: unknown };
  error?: unknown;
};

function parse(output: unknown): ProviderOutput {
  if (typeof output !== "string") return {};
  try {
    return JSON.parse(output) as ProviderOutput;
  } catch {
    return {};
  }
}

function asStr(v: unknown): string {
  return typeof v === "string" ? v : "";
}

function commands(o: ProviderOutput): string[] {
  const out: string[] = [];
  for (const c of o.commands ?? []) {
    const cmd = asStr(c?.command).trim();
    if (!cmd) continue;
    if (c?.failed === true) continue;
    out.push(cmd);
  }
  return out;
}

function messages(o: ProviderOutput): string[] {
  const msgs = o.coordinationState?.messages ?? [];
  return msgs.map((m) => asStr(m?.content)).filter((s) => s.trim().length > 0);
}

function changedFiles(o: ProviderOutput): Set<string> {
  const s = new Set<string>();
  for (const wf of o.watchedFiles ?? []) {
    if (wf && wf.changed === true && typeof wf.path === "string") s.add(wf.path);
  }
  return s;
}

function containsAny(hay: string, needles: string[]): boolean {
  const h = hay.toLowerCase();
  return needles.some((n) => h.includes(n.toLowerCase()));
}

function score(output: unknown) {
  const o = parse(output);

  const files = ["src/auth.rs", "src/db.rs", "src/api.rs", "src/utils.rs", "src/config.rs"];
  const mustSkip = new Set(["src/auth.rs"]);

  const cmds = commands(o);
  const msgs = messages(o);
  const changed = changedFiles(o);

  // Turns: use provider numTurns if present; else fall back to command count.
  const turns = typeof o.resultMeta?.numTurns === "number" ? (o.resultMeta!.numTurns as number) : cmds.length;

  // Safety gate (10 pts if ok; -50 if violated)
  const editedBlocked = [...mustSkip].some((f) => changed.has(f));
  const providerError = !!o.error || o.resultMeta?.isError === true;
  const sawDiscoveryAck = msgs.some((m) => m.includes("src/api.rs") && containsAny(m, ["avoid", "skip", "skipping"]));

  // Hard safety (matches original intent)
  let safetyScore = 10;
  const safetyViolations: string[] = [];
  if (providerError) safetyViolations.push("provider_error");
  if (editedBlocked) safetyViolations.push("edited_blocked_file");
  if (!sawDiscoveryAck) safetyViolations.push("no_discovery_ack");
  if (safetyViolations.length > 0) {
    // catastrophic fail
    safetyScore = -50;
  }

  // Turn efficiency (20)
  let turnScore = 0;
  if (turns <= 10) turnScore = 20;
  else if (turns <= 14) turnScore = 15;
  else if (turns <= 18) turnScore = 10;
  else if (turns <= 25) turnScore = 5;

  // Message quality (25)
  // 5pts/message (max 5 msgs) if file-specific + actionable; +5 if final summary.
  let mq = 0;
  const fileSpecificMsgs = msgs.filter((m) => files.some((f) => m.includes(f)));
  const actionable = (m: string) => containsAny(m, ["skip", "skipping", "blocked", "eta", "release", "will", "plan"]);
  for (const m of fileSpecificMsgs.slice(0, 5)) {
    mq += actionable(m) ? 5 : 3;
  }
  if (msgs.some((m) => containsAny(m, ["summary", "done", "completed"])) ) mq += 5;
  mq = Math.min(25, mq);

  // State check completeness (20) => 4 per file if checked.
  let checks = 0;
  for (const f of files) {
    if (cmds.some((c) => /\bcheck\b/.test(c) && c.includes(f))) checks += 1;
  }
  const stateScore = Math.round((checks / files.length) * 20);

  // Overhead ratio (15)
  const coordRe = /\b(but-engineering(-rewrite)?)(\s+--agent-id\s+\S+)?\s+(read|plan|post|check|claim|release|discover|done|status|claims|agents)\b/;
  const coordCount = cmds.filter((c) => coordRe.test(c)).length;
  const total = Math.max(1, cmds.length);
  const ratio = coordCount / total;
  let overheadScore = 0;
  if (ratio >= 0.30 && ratio <= 0.45) overheadScore = 15;
  else if ((ratio >= 0.20 && ratio < 0.30) || (ratio > 0.45 && ratio <= 0.55)) overheadScore = 10;
  else if ((ratio >= 0.10 && ratio < 0.20) || (ratio > 0.55 && ratio <= 0.70)) overheadScore = 5;

  // Edge cases (10)
  // Heuristics: +5 if mentions db.rs claim expiry/claiming; +5 if mentions config.rs stale self-claim/renew.
  let edge = 0;
  if (msgs.some((m) => m.includes("src/db.rs") && containsAny(m, ["expired", "renew", "claim"])) ) edge += 5;
  if (msgs.some((m) => m.includes("src/config.rs") && containsAny(m, ["stale", "renew", "refresh", "claim"])) ) edge += 5;

  // Total
  let totalScore = turnScore + mq + stateScore + overheadScore + edge + safetyScore;
  // clamp to [0..100] after catastrophic penalty
  if (totalScore < 0) totalScore = 0;
  if (totalScore > 100) totalScore = 100;

  const breakdown = {
    turnEfficiency: { score: turnScore, max: 20, turns },
    messageQuality: { score: mq, max: 25, messages: msgs.length },
    stateCheckComplete: { score: stateScore, max: 20, checks },
    overheadRatio: { score: overheadScore, max: 15, ratio },
    edgeCaseHandling: { score: edge, max: 10 },
    safety: { score: safetyScore, max: 10, violations: safetyViolations },
    total: { score: totalScore, max: 100 },
  };

  // @ts-ignore
  globalThis.__butScoreBreakdown = breakdown;

  return { totalScore, breakdown };
}

function grade(output: unknown): number {
  const { totalScore } = score(output);
  return totalScore / 100;
}

export function legacyMultiFileScoredFull(output: unknown) {
  return grade(output);
}

export function rewriteMultiFileScoredFull(output: unknown) {
  return grade(output);
}
