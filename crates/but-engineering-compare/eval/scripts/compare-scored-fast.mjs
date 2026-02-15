import fs from "node:fs";

function readScore(path) {
  const j = JSON.parse(fs.readFileSync(path, "utf8"));
  const res = j.results.results[0];
  const out = JSON.parse(res.response.output);
  const changed = (out.watchedFiles || []).filter((w) => w && w.changed).map((w) => w.path);
  const turns = out.resultMeta?.numTurns ?? null;
  const subtype = out.resultMeta?.subtype ?? null;
  const msgs = out.coordinationState?.messages?.length ?? 0;
  return {
    score: res.gradingResult?.score ?? null,
    pass: res.gradingResult?.pass ?? null,
    turns,
    subtype,
    changed,
    msgCount: msgs,
  };
}

const legacyPath = process.argv[2];
const rewritePath = process.argv[3];

const legacy = readScore(legacyPath);
const rewrite = readScore(rewritePath);

function pct(s) {
  return s == null ? "?" : `${Math.round(s * 100)}/100`;
}

console.log(`Legacy : ${pct(legacy.score)}  turns=${legacy.turns} subtype=${legacy.subtype} msgs=${legacy.msgCount} changed=${legacy.changed.join(",")}`);
console.log(`Rewrite : ${pct(rewrite.score)}  turns=${rewrite.turns} subtype=${rewrite.subtype} msgs=${rewrite.msgCount} changed=${rewrite.changed.join(",")}`);

if (legacy.score != null && rewrite.score != null) {
  const delta = Math.round((rewrite.score - legacy.score) * 100);
  console.log(`Delta (rewrite-legacy): ${delta} pts`);
}
