import assert from "node:assert/strict";
import test from "node:test";

import {
  filterThemes,
  parseThemeFilters,
  themeFilterHref,
} from "../src/lib/theme-filters.ts";

const themes = [
  {
    id: "codex-night",
    name: "Codex 夜色",
    description: "沉静的纯色开发主题",
    author: "豆皮",
    category: "codex",
    tags: ["深色"],
    hasBackground: false,
    targets: {
      doubao: { supportLevel: "tailored" },
      "doubao-work": { supportLevel: "tailored" },
      workbuddy: { supportLevel: "unsupported" },
    },
  },
  {
    id: "codex-room",
    name: "Codex 书房",
    description: "带有暖光背景",
    author: "豆皮",
    category: "codex",
    tags: ["暖色"],
    hasBackground: true,
    targets: {
      doubao: { supportLevel: "shared" },
      "doubao-work": { supportLevel: "shared" },
      workbuddy: { supportLevel: "shared" },
    },
  },
  {
    id: "gallery-rain",
    name: "雨夜画廊",
    description: "雨夜背景",
    author: "主题作者",
    category: "gallery",
    tags: ["氛围"],
    hasBackground: true,
    targets: {
      doubao: { supportLevel: "unsupported" },
      "doubao-work": { supportLevel: "unsupported" },
      workbuddy: { supportLevel: "tailored" },
    },
  },
];

const series = ["codex", "gallery", "atmosphere"];

test("type and series filters compose with AND semantics", () => {
  assert.deepEqual(
    filterThemes(themes, { type: "pure", series: "codex", target: "all" }).map(
      (theme) => theme.id,
    ),
    ["codex-night"],
  );
  assert.deepEqual(
    filterThemes(themes, { type: "background", series: "gallery", target: "all" }).map(
      (theme) => theme.id,
    ),
    ["gallery-rain"],
  );
});

test("search narrows the current facet result", () => {
  assert.deepEqual(
    filterThemes(themes, { type: "background", series: "all", target: "all" }, "暖").map(
      (theme) => theme.id,
    ),
    ["codex-room"],
  );
  assert.deepEqual(
    filterThemes(themes, { type: "pure", series: "gallery", target: "all" }, "雨"),
    [],
  );
});

test("invalid parameters fall back and legacy view remains readable", () => {
  assert.deepEqual(
    parseThemeFilters(new URLSearchParams("type=unknown&series=missing"), series),
    { type: "all", series: "all", target: "all" },
  );
  assert.deepEqual(parseThemeFilters(new URLSearchParams("view=codex"), series), {
    type: "all",
    series: "codex",
    target: "all",
  });
  assert.deepEqual(
    parseThemeFilters(new URLSearchParams("view=background"), series),
    { type: "background", series: "all", target: "all" },
  );
});

test("filter links preserve the other dimension and omit defaults", () => {
  assert.equal(
    themeFilterHref({ type: "background", series: "codex", target: "workbuddy" }),
    "/?type=background&series=codex&target=workbuddy#gallery",
  );
  assert.equal(themeFilterHref({ type: "all", series: "all", target: "all" }), "/#gallery");
});

test("target support composes with type, series and search", () => {
  assert.deepEqual(
    filterThemes(themes, { type: "background", series: "all", target: "workbuddy" }).map(
      (theme) => theme.id,
    ),
    ["codex-room", "gallery-rain"],
  );
  assert.deepEqual(
    filterThemes(themes, { type: "all", series: "gallery", target: "doubao" }),
    [],
  );
});
