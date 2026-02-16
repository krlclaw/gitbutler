import fs from "node:fs";

function load(path) {
  const j = JSON.parse(fs.readFileSync(path, "utf8"));
  const res = j.results.results[0];
  const output = JSON.parse(res.response.output);
  return { res, output };
}

async function compute(path, mode) {
  const { res, output } = load(path);
  // Recompute breakdown by importing the compiled scorer and reading global.
  // We call the scorer function directly so it sets globalThis.__butScoreBreakdown.
  const scorer = await import(`file://${process.cwd()}/dist/assertions/scored-multi-file-full.js`);
  const fn = mode === "legacy" ? scorer.legacyMultiFileScoredFull : scorer.rewriteMultiFileScoredFull;
  const score01 = fn(JSON.stringify(output));
  const breakdown = globalThis.__butScoreBreakdown;
  return { score01, breakdown, meta: output.resultMeta, changed: (output.watchedFiles||[]).filter(w=>w.changed).map(w=>w.path) };
}

const legacyPath = process.argv[2];
const rewritePath = process.argv[3];

const legacy = await compute(legacyPath, "legacy");
const rewrite = await compute(rewritePath, "rewrite");

function row(name, l, r, max) {
  const ld = `${l}/${max}`;
  const rd = `${r}/${max}`;
  const delta = r - l;
  const sign = delta > 0 ? `+${delta}` : `${delta}`;
  return `| ${name} | ${ld} | ${rd} | ${sign} |`;
}

const l = legacy.breakdown;
const r = rewrite.breakdown;

console.log(`| Dimension | Legacy | Rewrite | Delta |`);
console.log(`|---|---:|---:|---:|`);
console.log(row("Turn Efficiency", l.turnEfficiency.score, r.turnEfficiency.score, 20));
console.log(row("Message Quality", l.messageQuality.score, r.messageQuality.score, 25));
console.log(row("State Check Complete", l.stateCheckComplete.score, r.stateCheckComplete.score, 20));
console.log(row("Overhead Ratio", l.overheadRatio.score, r.overheadRatio.score, 15));
console.log(row("Edge Case Handling", l.edgeCaseHandling.score, r.edgeCaseHandling.score, 10));
console.log(row("Safety", l.safety.score, r.safety.score, 10));
console.log(`| TOTAL | ${l.total.score}/100 | ${r.total.score}/100 | ${r.total.score - l.total.score} |`);

console.log("\nNotes:");
console.log(`- Legacy changed: ${legacy.changed.join(", ") || "(none)"}`);
console.log(`- Rewrite changed: ${rewrite.changed.join(", ") || "(none)"}`);
