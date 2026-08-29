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

function svgPaint(value, fallback) {
  const paint = typeof value === "string" && value.trim() ? value.trim() : fallback;
  return paint.replaceAll("&", "&amp;").replaceAll('"', "&quot;").replaceAll("<", "&lt;");
}

async function renderPackagePreview(directory, info, colors, backgroundName) {
  const relative = info.preview?.image;
  if (!relative) return null;
  const destination = safeRelativeAsset(directory, relative, "preview.image");
  const width = 1200;
  const height = 675;
  const backgroundPath = backgroundName
    ? safeRelativeAsset(directory, backgroundName, "background")
    : null;
  const hasBackground = backgroundPath && fs.existsSync(backgroundPath);
  const previewAppearance = info.preview?.appearance === "light" ? "light" : "dark";
  const variant = info.variants?.[previewAppearance] ?? {};
  const previewContent = { ...info.content, ...variant.content };
  const previewComposer = { ...info.composer, ...variant.composer };
  const neutral = previewAppearance === "light" ? "#f5f5f3" : "#121318";
  const neutral2 = previewAppearance === "light" ? "#ebecea" : "#1a1c22";
  const accent = svgPaint(info.preview?.accent, colors.accent);
  const base = svgPaint(previewContent.chatBackground, colors.base);
  const base2 = svgPaint(previewComposer.background, colors.base2);
  const primary = svgPaint(previewContent.chatBackground, colors.primary);
  const floating = svgPaint(previewContent.assistantMessageBackground, colors.float);
  const text = svgPaint(previewContent.assistantMessageText, colors.text);
  const muted = svgPaint(colors.muted, "#9a9ba3");
  const hairline = svgPaint(colors.hairline, "#2c2e34");
  const assistant = svgPaint(previewContent.assistantMessageBackground, floating);
  const assistantText = svgPaint(previewContent.assistantMessageText, text);
  const user = svgPaint(previewContent.userMessageBackground, colors.accent);
  const userText = svgPaint(previewContent.userMessageText, "#ffffff");
  const composer = svgPaint(previewComposer.background, floating);
  const surface = hasBackground ? Math.max(0.58, info.surfaceOpacity ?? 0.68) : 1;

  const baseLayer = hasBackground
    ? await sharp(backgroundPath).resize(width, height, { fit: "cover" }).toBuffer()
    : Buffer.from(`<svg width="${width}" height="${height}" xmlns="http://www.w3.org/2000/svg">
        <defs><radialGradient id="g" cx="82%" cy="8%" r="90%">
          <stop offset="0" stop-color="${accent}" stop-opacity=".32"/>
          <stop offset=".48" stop-color="${base2}" stop-opacity=".22"/>
          <stop offset="1" stop-color="${base}"/>
        </radialGradient></defs>
        <rect width="100%" height="100%" fill="${neutral}"/>
        <rect width="100%" height="100%" fill="url(#g)"/>
      </svg>`);

  const overlay = Buffer.from(`<svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" xmlns="http://www.w3.org/2000/svg">
    <rect width="${width}" height="${height}" fill="${hasBackground ? neutral2 : base}" fill-opacity="${hasBackground ? 0.16 : 0}"/>
    <rect x="24" y="24" width="1152" height="627" rx="22" fill="${primary}" fill-opacity="${surface}" stroke="${hairline}" stroke-width="2"/>
    <path d="M24 70H1176" stroke="${hairline}" stroke-width="2"/>
    <circle cx="56" cy="47" r="8" fill="#ff5f57"/><circle cx="82" cy="47" r="8" fill="#febc2e"/><circle cx="108" cy="47" r="8" fill="#28c840"/>
    <path d="M248 70V651" stroke="${hairline}" stroke-width="2"/>
    <rect x="25" y="71" width="222" height="579" fill="${base2}" fill-opacity="${Math.max(0.7, surface)}"/>
    <rect x="48" y="102" width="42" height="42" rx="11" fill="${accent}" fill-opacity=".22"/>
    <circle cx="69" cy="123" r="9" fill="${accent}"/>
    <rect x="104" y="108" width="96" height="11" rx="5.5" fill="${text}" fill-opacity=".82"/>
    <rect x="104" y="128" width="68" height="8" rx="4" fill="${muted}" fill-opacity=".7"/>
    <rect x="44" y="178" width="160" height="12" rx="6" fill="${muted}" fill-opacity=".46"/>
    <rect x="44" y="217" width="176" height="42" rx="10" fill="${accent}" fill-opacity=".17"/>
    <rect x="60" y="232" width="104" height="11" rx="5.5" fill="${text}" fill-opacity=".82"/>
    <rect x="44" y="282" width="142" height="11" rx="5.5" fill="${muted}" fill-opacity=".58"/>
    <rect x="44" y="320" width="168" height="11" rx="5.5" fill="${muted}" fill-opacity=".48"/>
    <rect x="44" y="358" width="126" height="11" rx="5.5" fill="${muted}" fill-opacity=".48"/>
    <rect x="299" y="105" width="198" height="13" rx="6.5" fill="${text}" fill-opacity=".76"/>
    <rect x="299" y="130" width="122" height="9" rx="4.5" fill="${muted}" fill-opacity=".6"/>
    <rect x="300" y="188" width="466" height="104" rx="18" fill="${assistant}" fill-opacity="${surface}" stroke="${hairline}" stroke-width="1.5"/>
    <rect x="326" y="216" width="312" height="11" rx="5.5" fill="${assistantText}" fill-opacity=".82"/>
    <rect x="326" y="239" width="366" height="10" rx="5" fill="${assistantText}" fill-opacity=".56"/>
    <rect x="326" y="261" width="218" height="10" rx="5" fill="${assistantText}" fill-opacity=".56"/>
    <rect x="759" y="329" width="360" height="72" rx="18" fill="${user}"/>
    <rect x="790" y="355" width="246" height="11" rx="5.5" fill="${userText}" fill-opacity=".9"/>
    <rect x="300" y="438" width="402" height="74" rx="18" fill="${assistant}" fill-opacity="${surface}" stroke="${hairline}" stroke-width="1.5"/>
    <rect x="326" y="463" width="272" height="11" rx="5.5" fill="${assistantText}" fill-opacity=".72"/>
    <rect x="326" y="486" width="198" height="10" rx="5" fill="${assistantText}" fill-opacity=".52"/>
    <rect x="300" y="556" width="819" height="65" rx="24" fill="${composer}" fill-opacity="${Math.max(0.82, surface)}" stroke="${accent}" stroke-opacity=".42" stroke-width="2"/>
    <rect x="330" y="583" width="156" height="10" rx="5" fill="${muted}" fill-opacity=".64"/>
    <circle cx="1084" cy="588" r="21" fill="${accent}"/>
    <path d="M1084 598V579M1076 587L1084 579L1092 587" fill="none" stroke="${userText}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
  </svg>`);

  fs.mkdirSync(path.dirname(destination), { recursive: true });
  const rendered = await sharp(baseLayer)
    .composite([{ input: overlay }])
    .jpeg({ quality: 86, chromaSubsampling: "4:4:4" })
    .toBuffer();
  if (!fs.existsSync(destination) || !fs.readFileSync(destination).equals(rendered)) {
    fs.writeFileSync(destination, rendered);
  }
  return destination;
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

function exportPackage(directoryName, id) {
  fs.mkdirSync(publicPackagesDir, { recursive: true });
  const destination = path.join(publicPackagesDir, `${id}.doubao-skin.zip`);
  fs.rmSync(destination, { force: true });
  execFileSync(
    "zip",
    ["-X", "-q", "-r", destination, directoryName, "-x", "*/.DS_Store"],
    { cwd: themesDir },
  );
  const bytes = fs.readFileSync(destination);
  return {
    packageUrl: `/themes/packages/${path.basename(destination)}`,
    sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
    packageSize: bytes.length,
  };
}

const themeDirectories = fs.readdirSync(themesDir, { withFileTypes: true })
  .filter((entry) => entry.isDirectory() && fs.existsSync(path.join(themesDir, entry.name, "theme.json")))
  .map((entry) => entry.name)
  .sort();
if (themeDirectories.length === 0) throw new Error(`No themes found under ${themesDir}`);

fs.mkdirSync(path.dirname(databasePath), { recursive: true });
fs.mkdirSync(publicThemesDir, { recursive: true });
fs.rmSync(publicPackagesDir, { recursive: true, force: true });
fs.mkdirSync(publicPackagesDir, { recursive: true });
const preparedThemes = [];
const catalogThemes = [];

for (const [index, directoryName] of themeDirectories.entries()) {
  const directory = path.join(themesDir, directoryName);
  const info = JSON.parse(fs.readFileSync(path.join(directory, "theme.json"), "utf8"));
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
  const colors = parsedColors || defaultColors;
  let bgDetail = null;
  let bgCard = null;
  const backgroundName = typeof info.background === "string"
    ? info.background
    : info.background?.src ?? info.background?.source;
  if (backgroundName && fs.existsSync(path.join(directory, backgroundName))) {
    [bgDetail, bgCard] = await exportImage(path.join(directory, backgroundName), id);
  }
  const previewSource = await renderPackagePreview(directory, info, colors, backgroundName);
  const [previewDetail, previewCard] = await exportPreview(previewSource, id);
  const iconUrl = exportIcon(directory, info, id);
  const packageInfo = exportPackage(directoryName, id);
  const sortOrder = info.store.sortOrder ?? 900 + index;
  preparedThemes.push({
    id, name: info.name || id, description: info.description || "", version: info.version,
    author: info.author, category: info.store.category, tags: JSON.stringify(info.store.tags),
    hasBackground: bgDetail ? 1 : 0, veil: info.veil ?? info.background?.veil ?? null,
    colors: JSON.stringify(colors), bgDetail, bgCard, previewDetail, previewCard,
    inspiredBy: info.inspiredBy ?? null, sourceUrl: info.sourceUrl ?? null,
    sourceDownloads: info.sourceDownloads ?? null, sourceSnapshot: info.sourceSnapshot ?? null,
    isDefaultPalette: parsedColors ? 0 : 1,
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
    veil REAL, colors TEXT NOT NULL, bg_detail TEXT, bg_card TEXT,
    preview_detail TEXT, preview_card TEXT, inspired_by TEXT,
    source_url TEXT, source_downloads INTEGER, source_snapshot TEXT,
    is_default_palette INTEGER NOT NULL DEFAULT 0, sort_order INTEGER NOT NULL DEFAULT 999
  );
  CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
`);
const insertTheme = database.prepare(`
  INSERT INTO themes (id, name, description, version, author, category, tags,
    has_background, veil, colors, bg_detail, bg_card, preview_detail, preview_card,
    inspired_by, source_url, source_downloads, source_snapshot, is_default_palette, sort_order)
  VALUES (@id, @name, @description, @version, @author, @category, @tags,
    @hasBackground, @veil, @colors, @bgDetail, @bgCard, @previewDetail, @previewCard,
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
