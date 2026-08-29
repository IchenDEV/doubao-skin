import Database from "better-sqlite3";
import path from "node:path";
import type { Theme, ThemeColors } from "./types";

const DB_PATH = path.join(process.cwd(), "data", "themes.db");

export const CATEGORIES: Record<string, string> = {
  pure: "纯色",
  atmosphere: "氛围",
  gallery: "图库",
  codex: "Codex",
  brand: "品牌",
  misc: "其他",
};

export const CATEGORY_ORDER = [
  "pure",
  "atmosphere",
  "gallery",
  "codex",
  "brand",
  "misc",
];

interface ThemeRow {
  id: string;
  name: string;
  description: string;
  version: string;
  author: string;
  category: string;
  tags: string;
  has_background: number;
  veil: number | null;
  colors: string;
  bg_detail: string | null;
  bg_card: string | null;
  preview_detail: string | null;
  preview_card: string | null;
  inspired_by: string | null;
  source_url: string | null;
  source_downloads: number | null;
  source_snapshot: string | null;
  is_default_palette: number;
  sort_order: number;
}

let cached: Database.Database | null = null;

function getDb(): Database.Database {
  if (!cached) {
    cached = new Database(DB_PATH, { readonly: true, fileMustExist: true });
    cached.pragma("query_only = ON");
  }
  return cached;
}

function toTheme(row: ThemeRow): Theme {
  return {
    id: row.id,
    name: row.name,
    description: row.description,
    version: row.version,
    author: row.author,
    category: row.category,
    tags: JSON.parse(row.tags) as string[],
    hasBackground: row.has_background === 1,
    veil: row.veil,
    colors: JSON.parse(row.colors) as ThemeColors,
    bgDetail: row.bg_detail,
    bgCard: row.bg_card,
    previewDetail: row.preview_detail,
    previewCard: row.preview_card,
    inspiredBy: row.inspired_by,
    sourceUrl: row.source_url,
    sourceDownloads: row.source_downloads,
    sourceSnapshot: row.source_snapshot,
    isDefaultPalette: row.is_default_palette === 1,
    sortOrder: row.sort_order,
  };
}

export function getAllThemes(): Theme[] {
  const rows = getDb()
    .prepare("SELECT * FROM themes ORDER BY sort_order, id")
    .all() as ThemeRow[];
  return rows.map(toTheme);
}

export function getTheme(id: string): Theme | null {
  const row = getDb()
    .prepare("SELECT * FROM themes WHERE id = ?")
    .get(id) as ThemeRow | undefined;
  return row ? toTheme(row) : null;
}

export function getAdjacentThemes(id: string): { prev: Theme | null; next: Theme | null } {
  const all = getAllThemes();
  const index = all.findIndex((t) => t.id === id);
  return {
    prev: index > 0 ? all[index - 1] : null,
    next: index >= 0 && index < all.length - 1 ? all[index + 1] : null,
  };
}
