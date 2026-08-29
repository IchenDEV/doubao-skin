import assert from "node:assert/strict";
import {
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  checkSkillDiscovery,
  syncSkillDiscovery,
} from "./sync-skills.mjs";

const webRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = path.resolve(webRoot, "../..");
const pluginRoot = path.join(repoRoot, "plugins/doubao-skin");
const publicRoot = path.join(
  webRoot,
  "public/.well-known/agent-skills",
);
const skillNames = ["apply-doubao-theme", "create-doubao-theme"];

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function workspaceVersion() {
  const cargoToml = readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8");
  return cargoToml.match(/\[workspace\.package\][\s\S]*?\nversion = "([^"]+)"/)[1];
}

test("the repository exposes the two theme skills through one plugin package", () => {
  assert.equal(
    existsSync(pluginRoot),
    true,
    "plugins/doubao-skin must be the canonical plugin package",
  );

  for (const skillName of skillNames) {
    assert.equal(
      existsSync(path.join(pluginRoot, "skills", skillName, "SKILL.md")),
      true,
      `${skillName} must be packaged under plugins/doubao-skin/skills`,
    );
    assert.equal(
      existsSync(path.join(repoRoot, "skills", skillName)),
      false,
      `${skillName} must not have a second canonical copy under root skills`,
    );
  }
});

test("Codex and Claude manifests share package identity and workspace version", () => {
  const codex = readJson(
    path.join(pluginRoot, ".codex-plugin/plugin.json"),
  );
  const claude = readJson(
    path.join(pluginRoot, ".claude-plugin/plugin.json"),
  );
  for (const field of [
    "name",
    "version",
    "description",
    "author",
    "homepage",
    "repository",
    "license",
    "keywords",
    "skills",
  ]) {
    assert.deepEqual(codex[field], claude[field], `${field} must stay aligned`);
  }
  assert.equal(codex.version, workspaceVersion());
  assert.equal(codex.skills, "./skills/");
  for (const field of [
    "displayName",
    "shortDescription",
    "longDescription",
    "developerName",
    "category",
    "capabilities",
    "websiteURL",
    "defaultPrompt",
    "brandColor",
  ]) {
    assert.ok(codex.interface[field], `Codex interface.${field} is required`);
  }
});

test("both repository marketplaces expose only the plugin subdirectory", () => {
  const codex = readJson(path.join(repoRoot, ".agents/plugins/marketplace.json"));
  const claude = readJson(path.join(repoRoot, ".claude-plugin/marketplace.json"));
  assert.equal(codex.plugins.length, 1);
  assert.equal(claude.plugins.length, 1);
  assert.equal(codex.plugins[0].name, "doubao-skin");
  assert.deepEqual(codex.plugins[0].source, {
    source: "local",
    path: "./plugins/doubao-skin",
  });
  assert.deepEqual(codex.plugins[0].policy, {
    installation: "AVAILABLE",
    authentication: "ON_INSTALL",
  });
  assert.equal(claude.plugins[0].name, "doubao-skin");
  assert.equal(claude.plugins[0].source, "./plugins/doubao-skin");
});

test("checked-in well-known files match the canonical skill bytes", () => {
  assert.doesNotThrow(() =>
    checkSkillDiscovery({ pluginRoot, outputRoot: publicRoot }),
  );
});

test("well-known check rejects metadata, artifact, and file-list tampering", (t) => {
  const temporaryParent = mkdtempSync(path.join(os.tmpdir(), "doubao-skills-"));
  t.after(() => rmSync(temporaryParent, { recursive: true, force: true }));
  const outputRoot = path.join(temporaryParent, "agent-skills");

  syncSkillDiscovery({ pluginRoot, outputRoot });
  const indexPath = path.join(outputRoot, "index.json");
  const index = readJson(indexPath);
  index.skills[0].description = "tampered";
  writeFileSync(indexPath, `${JSON.stringify(index, null, 2)}\n`);
  assert.throws(() => checkSkillDiscovery({ pluginRoot, outputRoot }), /stale/);

  syncSkillDiscovery({ pluginRoot, outputRoot });
  const badDigestIndex = readJson(indexPath);
  badDigestIndex.skills[0].digest = `sha256:${"0".repeat(64)}`;
  writeFileSync(indexPath, `${JSON.stringify(badDigestIndex, null, 2)}\n`);
  assert.throws(() => checkSkillDiscovery({ pluginRoot, outputRoot }), /stale/);

  syncSkillDiscovery({ pluginRoot, outputRoot });
  const skillPath = path.join(outputRoot, skillNames[0], "SKILL.md");
  writeFileSync(skillPath, "tampered\n");
  assert.throws(() => checkSkillDiscovery({ pluginRoot, outputRoot }), /stale/);

  syncSkillDiscovery({ pluginRoot, outputRoot });
  mkdirSync(path.join(outputRoot, "unexpected"));
  writeFileSync(path.join(outputRoot, "unexpected/SKILL.md"), "unexpected\n");
  assert.throws(
    () => checkSkillDiscovery({ pluginRoot, outputRoot }),
    /generated skill files differ/,
  );
});
