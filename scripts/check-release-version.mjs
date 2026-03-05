import { readFile } from "node:fs/promises";
import path from "node:path";

function parseCargoPackageVersion(cargoTomlRaw) {
  const packageSectionMatch = cargoTomlRaw.match(/\[package\]([\s\S]*?)(?:\r?\n\[|$)/);
  if (!packageSectionMatch) {
    throw new Error("[release] Missing [package] section in Cargo.toml.");
  }

  const versionMatch = packageSectionMatch[1].match(/^\s*version\s*=\s*"([^"]+)"\s*$/m);
  if (!versionMatch) {
    throw new Error("[release] Missing package.version in Cargo.toml.");
  }

  return versionMatch[1];
}

async function main() {
  const refName = process.argv[2] ?? process.env.GITHUB_REF_NAME;
  if (!refName) {
    console.error("[release] Missing release tag. Pass as argv (e.g. v0.2.0) or set GITHUB_REF_NAME.");
    process.exit(1);
  }

  if (!/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(refName)) {
    console.error(`[release] Invalid tag format: "${refName}". Expected vX.Y.Z or vX.Y.Z-prerelease.`);
    process.exit(1);
  }

  const projectRoot = process.cwd();
  const runtimePackagePath = path.join(projectRoot, "apps", "runtime", "package.json");
  const tauriConfigPath = path.join(projectRoot, "apps", "runtime", "src-tauri", "tauri.conf.json");
  const cargoTomlPath = path.join(projectRoot, "apps", "runtime", "src-tauri", "Cargo.toml");

  const [runtimePackageRaw, tauriConfigRaw, cargoTomlRaw] = await Promise.all([
    readFile(runtimePackagePath, "utf8"),
    readFile(tauriConfigPath, "utf8"),
    readFile(cargoTomlPath, "utf8"),
  ]);

  const runtimePackage = JSON.parse(runtimePackageRaw);
  const tauriConfig = JSON.parse(tauriConfigRaw);
  const tagVersion = refName.slice(1);

  const checks = [
    { file: "apps/runtime/package.json", version: runtimePackage?.version },
    { file: "apps/runtime/src-tauri/tauri.conf.json", version: tauriConfig?.version },
    { file: "apps/runtime/src-tauri/Cargo.toml", version: parseCargoPackageVersion(cargoTomlRaw) },
  ];

  const mismatches = checks.filter((check) => check.version !== tagVersion);
  if (mismatches.length > 0) {
    console.error(`[release] Version mismatch detected for tag=${tagVersion}:`);
    for (const mismatch of mismatches) {
      console.error(`[release] - ${mismatch.file}: ${mismatch.version ?? "<missing>"}`);
    }
    console.error("[release] Please align all app versions before creating a release tag.");
    process.exit(1);
  }

  console.log(`[release] Version check passed: ${refName} matches runtime/package.json, tauri.conf.json, and Cargo.toml.`);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
