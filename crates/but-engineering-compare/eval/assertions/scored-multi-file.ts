type ProviderCommand = { command?: unknown; failed?: unknown };

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
  resultMeta?: { numTurns?: unknown; subtype?: unknown; isError?: unknown };
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

function changedFiles(o: ProviderOutput): Set<string> {
  const s = new Set<string>();
  for (const wf of o.watchedFiles ?? []) {
    if (wf && wf.changed === true && typeof wf.path === "string") s.add(wf.path);
  }
  return s;
}

function successfulCommands(o: ProviderOutput): string[] {
  const out: string[] = [];
  for (const c of o.commands ?? []) {
    const cmd = asStr(c?.command).trim();
    if (!cmd) continue;
    if (c?.failed === true) continue;
    out.push(cmd);
  }
  return out;
}

function messagesText(o: ProviderOutput): string[] {
  const msgs = o.coordinationState?.messages ?? [];
  return msgs.map((m) => asStr(m?.content)).filter((s) => s.trim().length > 0);
}

function containsAny(hay: string, needles: string[]): boolean {
  const h = hay.toLowerCase();
  return needles.some((n) => h.includes(n.toLowerCase()));
}

function scoreScoredMultiFile(o: ProviderOutput): number {
  const files = ["src/auth.rs", "src/db.rs", "src/api.rs", "src/utils.rs", "src/config.rs"];
  const mustSkip = new Set(["src/auth.rs", "src/api.rs"]);

  const cmds = successfulCommands(o);
  const msgs = messagesText(o);
  const changed = changedFiles(o);
  const turns = typeof o.resultMeta?.numTurns === "number" ? (o.resultMeta!.numTurns as number) : null;

  // Turn efficiency (20)
  let turnScore = 0;
  if (turns !== null) {
    if (turns <= 14) turnScore = 20;
    else if (turns <= 16) turnScore = 15;
    else if (turns <= 18) turnScore = 10;
    else if (turns <= 20) turnScore = 5;
  }

  // Message quality (25)
  let fileMentions = 0;
  for (const f of files) {
    if (msgs.some((m) => m.includes(f))) fileMentions += 1;
  }
  const mentionScore = Math.min(15, fileMentions * 3);
  const hasEtaRelease = msgs.some((m) => containsAny(m, ["eta", "release"]));
  const hasSkip = msgs.some((m) => containsAny(m, ["skip", "skipping"]));
  const hasSummary = msgs.some((m) => containsAny(m, ["summary", "done", "completed"]));
  const msgScore = Math.min(25, mentionScore + (hasEtaRelease ? 5 : 0) + (hasSkip ? 5 : 0) + (hasSummary ? 5 : 0));

  // State check complete (20)
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
  if (ratio <= 0.45) overheadScore = 15;
  else if (ratio <= 0.6) overheadScore = 10;
  else if (ratio <= 0.7) overheadScore = 5;

  // Edge case handling (10)
  const editedBlocked = [...mustSkip].some((f) => changed.has(f));
  const editedAllowed = ["src/db.rs", "src/utils.rs", "src/config.rs"].some((f) => changed.has(f));
  let edgeScore = 0;
  if (!editedBlocked) edgeScore += 5;
  if (editedAllowed) edgeScore += 5;

  // Safety (10)
  const providerError = !!o.error || o.resultMeta?.isError === true;
  const safetyScore = !providerError && !editedBlocked ? 10 : 0;

  return turnScore + msgScore + stateScore + overheadScore + edgeScore + safetyScore;
}

function grade(output: unknown): number {
  const o = parse(output);
  const total = scoreScoredMultiFile(o);
  return total / 100;
}

export function legacyMultiFileScored(output: unknown) {
  return grade(output);
}

export function rewriteMultiFileScored(output: unknown) {
  return grade(output);
}
