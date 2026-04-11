import path from "node:path";
import { access, readFile } from "node:fs/promises";

const SUPPORTED_MODES = new Set(["all", "drift"]);

async function readJson(filePath) {
  const raw = await readFile(filePath, "utf8").catch(() => null);
  if (!raw) {
    return null;
  }

  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

function buildFinding(detail, source) {
  return {
    category: "stale-doc-or-skill-reference",
    confidence: "probable",
    action: "review-first",
    source,
    detail,
  };
}

async function pathExists(filePath) {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}

export async function collectDriftSignals(options = {}) {
  const mode = options.mode ?? "all";
  if (!SUPPORTED_MODES.has(mode)) {
    return [];
  }

  const rootDir = path.resolve(options.rootDir ?? process.cwd());
  const packageJsonPath = path.join(rootDir, "package.json");
  const agentsPath = path.join(rootDir, "AGENTS.md");
  const repoHygieneDocPath = path.join(rootDir, "docs", "maintenance", "repo-hygiene.md");
  const repoHygieneReviewSkillPath = path.join(
    rootDir,
    ".agents",
    "skills",
    "workclaw-repo-hygiene-review",
    "SKILL.md",
  );
  const cleanupExecutionSkillPath = path.join(
    rootDir,
    ".agents",
    "skills",
    "workclaw-cleanup-execution",
    "SKILL.md",
  );

  const [packageJson, agentsDoc, repoHygieneDoc] = await Promise.all([
    readJson(packageJsonPath),
    readFile(agentsPath, "utf8").catch(() => ""),
    readFile(repoHygieneDocPath, "utf8").catch(() => ""),
  ]);

  const findings = [];
  const reviewScript = packageJson?.scripts?.["review:repo-hygiene"];

  if (!reviewScript) {
    findings.push(
      buildFinding("Missing review:repo-hygiene package script", "package.json"),
    );
  } else {
    const expectedTarget = "scripts/review-repo-hygiene.mjs";
    if (!reviewScript.includes(expectedTarget)) {
      findings.push(
        buildFinding(
          "Unexpected review:repo-hygiene script target",
          "package.json",
        ),
      );
    } else if (!(await pathExists(path.join(rootDir, expectedTarget)))) {
      findings.push(
        buildFinding(
          "Missing review:repo-hygiene script target",
          expectedTarget,
        ),
      );
    }
  }

  const referenceChecks = [
    {
      content: agentsDoc,
      source: "AGENTS.md",
      rules: [
        ["pnpm review:repo-hygiene", "Missing pnpm review:repo-hygiene AGENTS reference"],
        ["workclaw-repo-hygiene-review", "Missing workclaw-repo-hygiene-review AGENTS reference"],
        ["workclaw-cleanup-execution", "Missing workclaw-cleanup-execution AGENTS reference"],
      ],
    },
    {
      content: repoHygieneDoc,
      source: "docs/maintenance/repo-hygiene.md",
      rules: [
        ["pnpm review:repo-hygiene", "Missing pnpm review:repo-hygiene maintenance doc reference"],
        ["workclaw-repo-hygiene-review", "Missing workclaw-repo-hygiene-review maintenance doc reference"],
        ["workclaw-cleanup-execution", "Missing workclaw-cleanup-execution maintenance doc reference"],
        [".artifacts/repo-hygiene/", "Missing .artifacts/repo-hygiene/ maintenance doc artifact note"],
      ],
    },
  ];

  for (const { content, source, rules } of referenceChecks) {
    for (const [needle, detail] of rules) {
      if (!content.includes(needle)) {
        findings.push(buildFinding(detail, source));
      }
    }
  }

  const referencedSkillFiles = [
    [repoHygieneReviewSkillPath, "workclaw-repo-hygiene-review"],
    [cleanupExecutionSkillPath, "workclaw-cleanup-execution"],
  ];

  for (const [skillPath, skillName] of referencedSkillFiles) {
    const referencedByAgensOrDoc =
      agentsDoc.includes(skillName) || repoHygieneDoc.includes(skillName);
    if (referencedByAgensOrDoc && !(await pathExists(skillPath))) {
      findings.push(
        buildFinding(
          `Missing ${skillName} skill file`,
          path.relative(rootDir, skillPath).split(path.sep).join("/"),
        ),
      );
    }
  }

  return findings;
}
