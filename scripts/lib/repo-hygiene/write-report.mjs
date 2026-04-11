import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

function formatCountLines(countsByCategory) {
  const entries = Object.entries(countsByCategory);
  if (entries.length === 0) {
    return ["- none"];
  }

  return entries
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([category, count]) => `- ${category}: ${count}`);
}

function formatFindingLine(finding, index) {
  const parts = [
    `${index + 1}.`,
    `[${finding.category ?? "uncategorized"}]`,
    finding.confidence ?? "unknown",
    finding.action ?? "unknown",
  ];

  const source = finding.source ? `source=${finding.source}` : null;
  const detail = finding.detail ? `detail=${finding.detail}` : null;
  return [parts.join(" "), source, detail].filter(Boolean).join(" | ");
}

export async function writeRepoHygieneReport(outputDir, report) {
  await mkdir(outputDir, { recursive: true });

  const summaryLines = [
    "# Repo Hygiene Report",
    "",
    `Generated: ${report.generatedAt}`,
    `Mode: ${report.mode}`,
    `Total findings: ${report.findings.length}`,
    "",
    "## Counts By Category",
    ...formatCountLines(report.countsByCategory ?? {}),
    "",
    "## Findings",
    ...(report.findings.length === 0
      ? ["- none"]
      : report.findings.map((finding, index) => formatFindingLine(finding, index))),
    "",
  ];

  const summaryPath = path.join(outputDir, "summary.md");
  const reportPath = path.join(outputDir, "report.json");

  await writeFile(summaryPath, `${summaryLines.join("\n")}`, "utf8");
  await writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`, "utf8");

  return {
    summaryPath,
    reportPath,
  };
}
