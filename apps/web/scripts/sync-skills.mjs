import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const SCHEMA_URL =
  "https://schemas.agentskills.io/discovery/0.2.0/schema.json";
const SKILL_NAMES = ["apply-doubao-theme", "create-doubao-theme"];
const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const defaultRepoRoot = path.resolve(scriptDirectory, "../../..");
const defaultPluginRoot = path.join(defaultRepoRoot, "plugins/doubao-skin");
const defaultOutputRoot = path.join(
  defaultRepoRoot,
  "apps/web/public/.well-known/agent-skills",
);

function parseFrontmatter(contents, sourcePath) {
  const lines = contents.toString("utf8").split(/\r?\n/);
  if (lines[0] !== "---") {
    throw new Error(`${sourcePath}: missing YAML frontmatter`);
  }

  const end = lines.indexOf("---", 1);
  if (end === -1) {
    throw new Error(`${sourcePath}: unterminated YAML frontmatter`);
  }

  const metadata = {};
  for (const line of lines.slice(1, end)) {
    const separator = line.indexOf(":");
    if (separator === -1) continue;
    const key = line.slice(0, separator).trim();
    let value = line.slice(separator + 1).trim();
    if (
      value.length >= 2 &&
      ((value.startsWith('"') && value.endsWith('"')) ||
        (value.startsWith("'") && value.endsWith("'")))
    ) {
      value = value.slice(1, -1);
    }
    if (key === "name" || key === "description") metadata[key] = value;
  }

  if (!metadata.name || !metadata.description) {
    throw new Error(`${sourcePath}: name and description are required`);
  }
  return metadata;
}

function listFiles(root, prefix = "") {
  if (!existsSync(root)) return [];
  return readdirSync(root, { withFileTypes: true })
    .flatMap((entry) => {
      const relativePath = path.posix.join(prefix, entry.name);
      const absolutePath = path.join(root, entry.name);
      return entry.isDirectory()
        ? listFiles(absolutePath, relativePath)
        : [relativePath];
    })
    .sort();
}

export function buildSkillDiscovery({ pluginRoot = defaultPluginRoot } = {}) {
  const files = new Map();
  const skills = SKILL_NAMES.map((skillName) => {
    const sourcePath = path.join(pluginRoot, "skills", skillName, "SKILL.md");
    const contents = readFileSync(sourcePath);
    const metadata = parseFrontmatter(contents, sourcePath);
    if (metadata.name !== skillName) {
      throw new Error(
        `${sourcePath}: frontmatter name ${metadata.name} does not match directory`,
      );
    }

    const relativePath = `${skillName}/SKILL.md`;
    files.set(relativePath, contents);
    return {
      name: skillName,
      type: "skill-md",
      description: metadata.description,
      url: `/.well-known/agent-skills/${relativePath}`,
      digest: `sha256:${createHash("sha256").update(contents).digest("hex")}`,
    };
  });

  files.set(
    "index.json",
    Buffer.from(`${JSON.stringify({ $schema: SCHEMA_URL, skills }, null, 2)}\n`),
  );
  return files;
}

export function checkSkillDiscovery({
  pluginRoot = defaultPluginRoot,
  outputRoot = defaultOutputRoot,
} = {}) {
  const expected = buildSkillDiscovery({ pluginRoot });
  const actualPaths = listFiles(outputRoot);
  const expectedPaths = [...expected.keys()].sort();
  if (JSON.stringify(actualPaths) !== JSON.stringify(expectedPaths)) {
    throw new Error(
      `generated skill files differ: expected ${expectedPaths.join(", ")}; found ${actualPaths.join(", ") || "none"}`,
    );
  }

  for (const [relativePath, contents] of expected) {
    const actual = readFileSync(path.join(outputRoot, relativePath));
    if (!actual.equals(contents)) {
      throw new Error(`generated skill file is stale: ${relativePath}`);
    }
  }
}

export function syncSkillDiscovery({
  pluginRoot = defaultPluginRoot,
  outputRoot = defaultOutputRoot,
} = {}) {
  const expected = buildSkillDiscovery({ pluginRoot });
  const parent = path.dirname(outputRoot);
  mkdirSync(parent, { recursive: true });
  const temporaryRoot = mkdtempSync(path.join(parent, ".agent-skills-"));
  const backupRoot = `${outputRoot}.previous`;

  try {
    for (const [relativePath, contents] of expected) {
      const destination = path.join(temporaryRoot, relativePath);
      mkdirSync(path.dirname(destination), { recursive: true });
      writeFileSync(destination, contents);
    }

    rmSync(backupRoot, { recursive: true, force: true });
    if (existsSync(outputRoot)) renameSync(outputRoot, backupRoot);
    renameSync(temporaryRoot, outputRoot);
    rmSync(backupRoot, { recursive: true, force: true });
  } catch (error) {
    rmSync(temporaryRoot, { recursive: true, force: true });
    if (!existsSync(outputRoot) && existsSync(backupRoot)) {
      renameSync(backupRoot, outputRoot);
    }
    throw error;
  }
}

const invokedDirectly =
  process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href;
if (invokedDirectly) {
  if (process.argv.includes("--check")) {
    checkSkillDiscovery();
    console.log("skill discovery output is current");
  } else {
    syncSkillDiscovery();
    console.log("skill discovery output synchronized");
  }
}
