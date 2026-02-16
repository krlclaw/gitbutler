import fs from "node:fs";

function die(msg) {
  console.error(msg);
  process.exit(2);
}

const path = process.argv[2];
if (!path) {
  die("usage: node scripts/scored-repeat-metrics.mjs <promptfoo-output.json>");
}

let data;
try {
  data = JSON.parse(fs.readFileSync(path, "utf8"));
} catch (e) {
  die(`failed to read/parse JSON: ${path}: ${e && e.message ? e.message : String(e)}`);
}

const results = data?.results?.results;
if (!Array.isArray(results)) {
  die(`unexpected promptfoo output shape: missing results.results array in ${path}`);
}

const scores = results
  .map((r) => r && typeof r.score === "number" && Number.isFinite(r.score) ? r.score : null)
  .filter((s) => s !== null);

if (scores.length === 0) {
  die(`no numeric scores found in results for ${path}`);
}

const n = scores.length;
const mean = scores.reduce((a, b) => a + b, 0) / n;
const variance = scores.reduce((a, b) => a + (b - mean) ** 2, 0) / n;
const stddev = Math.sqrt(variance);
const min = Math.min(...scores);
const max = Math.max(...scores);

process.stdout.write(
  JSON.stringify(
    {
      n,
      mean,
      stddev,
      min,
      max,
    },
    null,
    2,
  ) + "\n",
);

