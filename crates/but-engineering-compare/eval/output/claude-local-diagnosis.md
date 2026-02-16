# Claude Local Runner Diagnosis and Fixes

**Date:** 2026-02-16  
**Branch:** `tars/but-engineering-rewrite`  
**Scope:** `crates/but-engineering-compare/eval`

## Executive Summary

Claude local runner was **unreliable** in promptfoo eval harness due to:
1. **PATH environment issue** - Claude CLI not found when invoked from npm/promptfoo child processes
2. **Timeout handling gaps** - Node.js `execFileSync({timeout})` insufficient for shell → CLI → agent process chains
3. **Retry logic blind spot** - Provider didn't detect runner-level timeout messages

**Status after fixes:** ✅ **Reliable** (3/3 trials pass, avg ~80s duration)

---

## Root Causes

### 1. **PATH Environment Missing Homebrew Binaries**

**Symptom:**  
Direct invocation of `providers/claude-local.sh` succeeded, but promptfoo-invoked runs failed with "claude: command not found" or hung indefinitely.

**Root cause:**  
- macOS Homebrew installs Claude CLI to `/opt/homebrew/bin/claude`
- `npm exec` / `promptfoo` child processes inherit minimal PATH without `/opt/homebrew/bin`
- Runner script relied on `command -v claude` without PATH fallback

**Evidence:**
```bash
# Direct shell (success):
$ claude --version
2.1.42 (Claude Code)

# From promptfoo child process (failure):
$ npm exec promptfoo eval ...
  → bash providers/claude-local.sh
    → claude: command not found
```

### 2. **Timeout Enforcement Insufficient**

**Symptom:**  
Eval runs would hang indefinitely (observed >4min with 240s timeout configured).

**Root cause:**  
- `execFileSync("bash", [...], {timeout: 240000})` sends SIGTERM to immediate child (bash script)
- Bash script spawns Python wrapper → spawns Claude CLI → spawns agent process
- Process group not killed; grandchildren continue running
- Node timeout only kills top-level bash, leaving Claude agent orphaned

**Evidence:**
```bash
# During hang:
$ ps -Ao pid,ppid,pgid,etime,command | grep claude
73274 73244 72996  04:12  Python - 240000 /opt/homebrew/bin/claude -p ...
# ^ Still running after parent bash killed
```

### 3. **Retry Logic Didn't Detect Runner Timeouts**

**Symptom:**  
Provider retry logic would not trigger on runner-level timeouts.

**Root cause:**  
- Python wrapper in `claude-local.sh` prints `"Claude runner timed out after 240000ms"` to stderr
- Provider's `parseJsonLines(rawAgentOutput)` returned empty events array
- Retry condition only checked `timedOut || (events.length === 0 && output.trim().length === 0)`
- Did not check for timeout message in stderr output

---

## Fixes Applied

### Fix 1: PATH Injection in Runner Script

**File:** `crates/but-engineering-compare/eval/providers/claude-local.sh`

**Change:**
```diff
 #!/usr/bin/env bash
 set -euo pipefail
 
+# Ensure common Homebrew install prefixes are visible when invoked from non-login shells
+# (e.g. promptfoo/npm child processes).
+case ":${PATH}:" in
+  *":/opt/homebrew/bin:"*) ;;
+  *) PATH="/opt/homebrew/bin:/opt/homebrew/sbin:${PATH}" ;;
+esac
+export PATH
+
 CLAUDE_BIN="${BUT_EVAL_CLAUDE_BIN:-${BUT_EVAL_RUNNER_BIN:-claude}}"
```

**Rationale:**  
- Idempotent PATH prepend (only adds if not present)
- Standard Homebrew install prefix for macOS ARM64
- Mirrors existing codex-local.sh pattern

---

### Fix 2: Timeout Buffer for execFileSync

**File:** `crates/but-engineering-compare/eval/providers/engineering-integration.ts`

**Change:**
```diff
       for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
         rawAgentOutput = "";
         cliRunError = null;
         let timedOut = false;
+        const runnerExecTimeoutMs = runnerTimeoutMs + 15_000;
 
         try {
           rawAgentOutput = execFileSync("bash", [runnerScript], {
             cwd: fixtureDir,
             encoding: "utf8",
             stdio: ["ignore", "pipe", "pipe"],
-            timeout: runnerTimeoutMs,
+            timeout: runnerExecTimeoutMs,
             killSignal: "SIGKILL",
```

**Rationale:**  
- Give Python wrapper (inside runner script) time to enforce timeout before Node.js kills process tree
- Python wrapper uses `runnerTimeoutMs` directly; Node adds 15s buffer
- Allows clean timeout message propagation vs abrupt SIGKILL

---

### Fix 3: Retry on Runner Timeout Message

**File:** `crates/but-engineering-compare/eval/providers/engineering-integration.ts`

**Change:**
```diff
         events = parseJsonLines(rawAgentOutput);
-        const retryable = timedOut || (events.length === 0 && rawAgentOutput.trim().length === 0);
+        const timeoutFromRunner = rawAgentOutput.includes("runner timed out after");
+        const retryable =
+          timedOut || timeoutFromRunner || (events.length === 0 && rawAgentOutput.trim().length === 0);
         if (!retryable || attempt >= maxAttempts) {
           break;
         }
```

**Rationale:**  
- Detect explicit timeout message from Python wrapper
- Treat runner-level timeouts as retryable (same as Node-level timeout)
- Improves transient failure recovery

---

## Reproduction Steps

### Before Fixes (Unreliable)

```bash
cd ~/src/gitbutler/crates/but-engineering-compare/eval

# Revert to baseline (before fixes):
git checkout HEAD~1 -- providers/claude-local.sh providers/engineering-integration.ts
npm run build

# Run trial:
BUT_EVAL_AGENT=claude \
BUT_EVAL_RUNNER=providers/claude-local.sh \
BUT_EVAL_AUTH_MODE=local \
BUT_EVAL_MODEL=claude-sonnet-4-5-20250929 \
BUT_EVAL_RUNNER_TIMEOUT_MS=240000 \
PROMPTFOO_DISABLE_TELEMETRY=1 \
PROMPTFOO_DISABLE_UPDATE=1 \
npx promptfoo eval -c promptfooconfig.compare-rewrite.fast.yaml --filter-pattern "SCORED"

# Expected: Hangs or fails with "claude: command not found"
```

### After Fixes (Reliable)

```bash
# Apply fixes (already in HEAD):
npm run build

# Run 3 trials:
for i in 1 2 3; do
  BUT_EVAL_AGENT=claude \
  BUT_EVAL_RUNNER=providers/claude-local.sh \
  BUT_EVAL_AUTH_MODE=local \
  BUT_EVAL_MODEL=claude-sonnet-4-5-20250929 \
  BUT_EVAL_RUNNER_TIMEOUT_MS=240000 \
  PROMPTFOO_DISABLE_TELEMETRY=1 \
  PROMPTFOO_DISABLE_UPDATE=1 \
  /usr/bin/time -p npx promptfoo eval \
    -c promptfooconfig.compare-rewrite.fast.yaml \
    --filter-pattern "SCORED" \
    -o output/claude-rewrite-fast-trial${i}.json
done

# Expected: All 3 pass in ~75-90s
```

---

## Reliability Evidence

### Trial Results (After Fixes)

| Trial | Status | Duration | Output Size | Eval ID |
|-------|--------|----------|-------------|---------|
| 1     | ✅ PASS | 76.06s   | 18,376 bytes | eval-rvU-2026-02-16T09:03:57 |
| 2     | ✅ PASS | 85.77s   | 18,777 bytes | eval-KwB-2026-02-16T09:05:13 |
| 3     | ✅ PASS | 86.00s   | 18,452 bytes | eval-1jo-2026-02-16T09:10:00 |

**Average successful duration:** ~82.6s  
**Success rate:** 100% (3/3 trials)

### Sample Output (Trial 1)
```json
{
  "fixtureDir": null,
  "taskPrompt": "Use Skill(but-engineering-rewrite)...",
  "commands": [...],
  "editOperations": [...],
  "result": "...",
  "resultMeta": {
    "subtype": "success",
    "isError": false,
    "totalCostUsd": 0.020429,
    "numTurns": 1,
    "durationMs": 1875,
    "error": null
  },
  ...
}
```

---

## Risk Assessment & Rollback

### Risk Level: **LOW**

**Why safe:**
1. **Scoped changes** - Only touches eval harness runners, no production code
2. **PATH injection** - Idempotent, only adds if missing (no override)
3. **Timeout buffer** - Conservative 15s pad, doesn't change runner timeout logic
4. **Retry condition** - Additive check, preserves existing retry triggers

### Rollback Procedure

If issues arise:

```bash
cd ~/src/gitbutler
git revert HEAD  # Revert diagnosis commit
npm run build --prefix crates/but-engineering-compare/eval
```

Or targeted rollback:

```bash
# Remove PATH injection:
sed -i.bak '/# Ensure common Homebrew/,/^$/d' \
  crates/but-engineering-compare/eval/providers/claude-local.sh

# Remove timeout buffer:
git checkout HEAD~1 -- \
  crates/but-engineering-compare/eval/providers/engineering-integration.ts

npm run build --prefix crates/but-engineering-compare/eval
```

---

## Additional Notes

### Known Limitations

1. **macOS-specific PATH fix** - Linux/Windows may need different Homebrew paths
2. **15s timeout buffer** - Arbitrary; may need tuning for slower machines
3. **No inter-runner isolation** - Concurrent Claude + Codex runs share fixture cleanup

### Future Improvements

1. **Runner health check** - Pre-flight validation of CLI availability
2. **Fixture locking** - Prevent concurrent eval runs from stomping fixtures
3. **Timeout auto-tuning** - Adaptive timeout based on historical run durations
4. **Better stderr parsing** - Structured error codes vs string matching

---

## Commit Message

```
fix(eval): make Claude local runner reliable in promptfoo harness

Root causes:
- PATH missing /opt/homebrew/bin when invoked from npm child processes
- execFileSync timeout insufficient for shell→CLI→agent process chains
- Retry logic didn't detect runner-level timeout messages

Fixes:
1. Inject Homebrew PATH in claude-local.sh (idempotent)
2. Add 15s timeout buffer for clean Python wrapper termination
3. Retry on "runner timed out after" stderr message

Evidence:
- Trial 1: ✅ 76s (18KB output)
- Trial 2: ✅ 86s (18KB output)
- Trial 3: ✅ 86s (18KB output)

Reliability: 3/3 trials pass (100%)
Risk: LOW (eval harness only, idempotent PATH, conservative timeout)

Ref: crates/but-engineering-compare/eval/output/claude-local-diagnosis.md
```

---

## Conclusion

Claude local runner is now **reliable enough for production eval use**. All identified issues have minimal-risk fixes applied. Recommend monitoring first 10 production runs for edge cases, but expect consistent ~80s completion times for fast-mode scored evals.

**Next steps:**
1. ✅ Commit fixes to `tars/but-engineering-rewrite`
2. ✅ Complete trial 3 for full 3/3 reliability confirmation
3. 📊 Run full-rubric compare (legacy vs rewrite) with Claude runner (optional)
4. 📝 Update eval harness docs with PATH requirements (optional)
