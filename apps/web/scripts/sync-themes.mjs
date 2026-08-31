import { execFileSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import Database from "better-sqlite3";
import sharp from "sharp";

const siteDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repoDir = path.resolve(siteDir, "../..");
const themesDir = path.join(repoDir, "themes");
const databasePath = path.join(siteDir, "data/themes.db");
const publicThemesDir = path.join(siteDir, "public/themes");
const publicPackagesDir = path.join(publicThemesDir, "packages");
const catalogPath = path.join(publicThemesDir, "catalog.json");

function newestSourceTimestamp(directory) {
  let newest = 0;
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (entry.name === ".DS_Store") continue;
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      newest = Math.max(newest, newestSourceTimestamp(entryPath));
    } else if (entry.isFile()) {
      newest = Math.max(newest, fs.statSync(entryPath).mtimeMs);
    }
  }
  return newest;
}

const tokenMap = {
  base: "--s-color-bg-base", base2: "--dbx-bg-base-2", primary: "--s-color-bg-primary",
  float: "--s-color-bg-float", text: "--N900", muted: "--N500", hairline: "--N100",
  accent: "--semi-color-primary", accentHover: "--semi-color-primary-hover", brand: "--B500",
};
const defaultColors = {
  base: "#1b1c20", base2: "#141519", primary: "#24262b", float: "#27292e",
  text: "#f2f3f5", muted: "#9a9ba3", hairline: "#2c2e34",
  accent: "rgba(76,110,245,1)", accentHover: "rgba(98,128,247,1)", brand: "#4c6ef5",
};

function parseColors(css) {
  const tokens = new Map([...css.matchAll(/--([A-Za-z0-9_-]+)\s*:\s*([^;]+);/g)].map((match) => [match[1], match[2]]));
  const colors = {};
  for (const [key, token] of Object.entries(tokenMap)) {
    const value = tokens.get(token.slice(2));
    if (!value) return null;
    colors[key] = value.trim();
  }
  return colors;
}

function structuredColors(info) {
  const visual = info.schemaVersion === 3 ? info.shared : info;
  const appearance = info.preview?.appearance === "light" ? "light" : "dark";
  const variant = visual?.variants?.[appearance] ?? {};
  const content = { ...visual?.content, ...variant.content };
  const composer = { ...visual?.composer, ...variant.composer };
  const accent = info.preview?.accent ?? defaultColors.accent;
  return {
    base: content.chatBackground ?? defaultColors.base,
    base2: composer.background ?? defaultColors.base2,
    primary: content.assistantMessageBackground ?? defaultColors.primary,
    float: composer.background ?? defaultColors.float,
    text: content.assistantMessageText ?? defaultColors.text,
    muted: composer.placeholderColor ?? defaultColors.muted,
    hairline: content.scrollbarColor ?? defaultColors.hairline,
    accent,
    accentHover: accent,
    brand: accent,
  };
}

function runThemeCli(args) {
  let output;
  try {
    output = execFileSync(
      "cargo",
      ["run", "-q", "-p", "skin-core", "--bin", "doubao-theme", "--", ...args, "--json"],
      { cwd: repoDir, encoding: "utf8" },
    );
  } catch (error) {
    const response = JSON.parse(String(error.stdout ?? "{}"));
    throw new Error(response.error?.message ?? `doubao-theme ${args[0]} failed`);
  }
  const response = JSON.parse(output);
  if (!response.ok) throw new Error(response.error?.message ?? `doubao-theme ${args[0]} failed`);
  return response.result;
}

async function exportImage(source, id) {
  fs.mkdirSync(publicThemesDir, { recursive: true });
  const detail = path.join(publicThemesDir, `${id}.jpg`);
  const card = path.join(publicThemesDir, `${id}.card.jpg`);
  await sharp(source).resize({ width: 1600, withoutEnlargement: true }).jpeg({ quality: 86 }).toFile(detail);
  await sharp(source).resize({ width: 800, withoutEnlargement: true }).jpeg({ quality: 82 }).toFile(card);
  return [`/themes/${id}.jpg`, `/themes/${id}.card.jpg`];
}

function safeRelativeAsset(directory, relative, label) {
  if (typeof relative !== "string" || relative.length === 0 || path.isAbsolute(relative)) {
    throw new Error(`${label} must be a relative file path`);
  }
  const normalized = path.normalize(relative);
  if (normalized === ".." || normalized.startsWith(`..${path.sep}`)) {
    throw new Error(`${label} must stay inside the theme directory`);
  }
  return path.join(directory, normalized);
}

async function exportPreview(source, id) {
  if (!source) return [null, null];
  const detail = path.join(publicThemesDir, `${id}.preview.jpg`);
  const card = path.join(publicThemesDir, `${id}.preview.card.jpg`);
  await sharp(source).resize({ width: 1600, withoutEnlargement: true }).jpeg({ quality: 86 }).toFile(detail);
  await sharp(source).resize({ width: 800, withoutEnlargement: true }).jpeg({ quality: 82 }).toFile(card);
  return [`/themes/${id}.preview.jpg`, `/themes/${id}.preview.card.jpg`];
}

function exportIcon(directory, info, id) {
  const relative = info.icons?.main
    ?? info.variants?.light?.icons?.main
    ?? info.variants?.dark?.icons?.main;
  if (!relative) return null;
  const source = path.join(directory, relative);
  if (!fs.existsSync(source) || !fs.statSync(source).isFile()) return null;
  const extension = path.extname(source).toLowerCase() || ".svg";
  const destination = path.join(publicThemesDir, `${id}.icon${extension}`);
  fs.copyFileSync(source, destination);
  return `/themes/${path.basename(destination)}`;
}

function exportPackage(directory, id) {
  fs.mkdirSync(publicPackagesDir, { recursive: true });
  const destination = path.join(publicPackagesDir, `${id}.doubao-skin.zip`);
  fs.rmSync(destination, { force: true });
  runThemeCli(["pack", directory, destination]);
  const bytes = fs.readFileSync(destination);
  return {
    packageUrl: `/themes/packages/${path.basename(destination)}`,
    sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
    packageSize: bytes.length,
  };
}

const requestedThemes = new Set(
  (process.env.DOUBAO_SKIN_SYNC_THEME_IDS ?? "")
    .split(",")
    .map((id) => id.trim())
    .filter(Boolean),
);
const themeDirectories = fs.readdirSync(themesDir, { withFileTypes: true })
  .filter((entry) => entry.isDirectory() && fs.existsSync(path.join(themesDir, entry.name, "theme.json")))
  .map((entry) => entry.name)
  .filter((id) => requestedThemes.size === 0 || requestedThemes.has(id))
  .sort();
if (themeDirectories.length === 0) throw new Error(`No themes found under ${themesDir}`);
if (requestedThemes.size > 0 && themeDirectories.length !== requestedThemes.size) {
  throw new Error("DOUBAO_SKIN_SYNC_THEME_IDS contains an unknown theme id");
}

fs.mkdirSync(path.dirname(databasePath), { recursive: true });
fs.mkdirSync(publicThemesDir, { recursive: true });
fs.rmSync(publicPackagesDir, { recursive: true, force: true });
fs.mkdirSync(publicPackagesDir, { recursive: true });
const preparedThemes = [];
const catalogThemes = [];

for (const [index, directoryName] of themeDirectories.entries()) {
  const directory = path.join(themesDir, directoryName);
  const info = JSON.parse(fs.readFileSync(path.join(directory, "theme.json"), "utf8"));
  const validation = runThemeCli(["check", directory]).validation;
  const id = info.id || directoryName;
  if (id !== directoryName) throw new Error(`${directoryName}: theme id must match the directory name`);
  if (!info.version || !info.author || !info.preview?.image || !info.preview?.accent) {
    throw new Error(`${id}: version, author and preview metadata are required for store packages`);
  }
  if (!info.store?.category || !Array.isArray(info.store.tags) || !Number.isInteger(info.store.sortOrder)) {
    throw new Error(`${id}: store.category, store.tags and store.sortOrder are required`);
  }
  const cssPath = path.join(directory, "theme.css");
  const parsedColors = parseColors(fs.existsSync(cssPath) ? fs.readFileSync(cssPath, "utf8") : "");
  const colors = parsedColors || structuredColors(info);
  const visual = info.schemaVersion === 3 ? info.shared : info;
  const previewInfo = info.schemaVersion === 3 ? { ...info, ...info.shared } : info;
  let bgDetail = null;
  let bgCard = null;
  const backgroundName = typeof visual.background === "string"
    ? visual.background
    : visual.background?.src ?? visual.background?.source;
  if (backgroundName && fs.existsSync(path.join(directory, backgroundName))) {
    [bgDetail, bgCard] = await exportImage(path.join(directory, backgroundName), id);
  }
  const previewSource = safeRelativeAsset(directory, info.preview.image, "preview.image");
  if (!fs.existsSync(previewSource) || !fs.statSync(previewSource).isFile()) {
    throw new Error(`${id}: preview image is missing`);
  }
  const [previewDetail, previewCard] = await exportPreview(previewSource, id);
  const iconUrl = exportIcon(directory, previewInfo, id);
  const packageInfo = exportPackage(directory, id);
  const sortOrder = info.store.sortOrder ?? 900 + index;
  preparedThemes.push({
    id, name: info.name || id, description: info.description || "", version: info.version,
    author: info.author, category: info.store.category, tags: JSON.stringify(info.store.tags),
    hasBackground: bgDetail ? 1 : 0, veil: visual.veil ?? visual.background?.veil ?? null,
    colors: JSON.stringify(colors), bgDetail, bgCard, previewDetail, previewCard,
    inspiredBy: info.provenance?.inspiredBy ?? info.inspiredBy ?? null,
    sourceUrl: info.provenance?.sourceUrl ?? info.sourceUrl ?? null,
    sourceDownloads: info.provenance?.sourceDownloads ?? info.sourceDownloads ?? null,
    sourceSnapshot: info.provenance?.sourceVersion ?? info.sourceSnapshot ?? null,
    isDefaultPalette: parsedColors ? 0 : 1,
    schemaVersion: validation.schemaVersion,
    targets: JSON.stringify(validation.targets),
    sortOrder,
  });
  catalogThemes.push({
    id,
    name: info.name || id,
    description: info.description || "",
    version: info.version,
    author: info.author,
    category: info.store.category,
    tags: info.store.tags,
    schemaVersion: validation.schemaVersion,
    targets: validation.targets,
    previewUrl: previewDetail,
    thumbnailUrl: previewCard ?? bgCard,
    iconUrl,
    accent: info.preview.accent || colors.accent,
    ...packageInfo,
    sortOrder,
  });
  console.log(`  ${id}`);
}

const generatedAt = new Date(newestSourceTimestamp(themesDir)).toISOString();
fs.rmSync(databasePath, { force: true });
const database = new Database(databasePath);
database.exec(`
  CREATE TABLE themes (
    id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
    version TEXT NOT NULL, author TEXT NOT NULL, category TEXT NOT NULL DEFAULT 'misc',
    tags TEXT NOT NULL DEFAULT '[]', has_background INTEGER NOT NULL DEFAULT 0,
    schema_version INTEGER NOT NULL DEFAULT 1, targets TEXT NOT NULL DEFAULT '{}',
    veil REAL, colors TEXT NOT NULL, bg_detail TEXT, bg_card TEXT,
    preview_detail TEXT, preview_card TEXT, inspired_by TEXT,
    source_url TEXT, source_downloads INTEGER, source_snapshot TEXT,
    is_default_palette INTEGER NOT NULL DEFAULT 0, sort_order INTEGER NOT NULL DEFAULT 999
  );
  CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
`);
const insertTheme = database.prepare(`
  INSERT INTO themes (id, name, description, version, author, category, tags,
    has_background, schema_version, targets, veil, colors, bg_detail, bg_card, preview_detail, preview_card,
    inspired_by, source_url, source_downloads, source_snapshot, is_default_palette, sort_order)
  VALUES (@id, @name, @description, @version, @author, @category, @tags,
    @hasBackground, @schemaVersion, @targets, @veil, @colors, @bgDetail, @bgCard, @previewDetail, @previewCard,
    @inspiredBy, @sourceUrl, @sourceDownloads, @sourceSnapshot, @isDefaultPalette, @sortOrder)
`);

database.transaction(() => {
  preparedThemes.forEach((theme) => insertTheme.run(theme));
  database.prepare("INSERT INTO meta (key, value) VALUES (?, ?)").run("built_at", generatedAt);
  database.prepare("INSERT INTO meta (key, value) VALUES (?, ?)").run("theme_count", String(themeDirectories.length));
})();
database.close();
const themes = catalogThemes
  .sort((left, right) => left.sortOrder - right.sortOrder)
  .map(({ sortOrder: _sortOrder, ...theme }) => theme);
fs.writeFileSync(
  catalogPath,
  `${JSON.stringify({ schemaVersion: 1, generatedAt, themes }, null, 2)}\n`,
);
console.log(`\n${themeDirectories.length} themes -> ${path.relative(repoDir, databasePath)}`);
console.log(`${themes.length} installable packages -> ${path.relative(repoDir, catalogPath)}`);
