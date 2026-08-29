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
  },
  {
    id: "codex-room",
    name: "Codex 书房",
    description: "带有暖光背景",
    author: "豆皮",
    category: "codex",
    tags: ["暖色"],
    hasBackground: true,
  },
  {
    id: "gallery-rain",
    name: "雨夜画廊",
    description: "雨夜背景",
    author: "主题作者",
    category: "gallery",
    tags: ["氛围"],
    hasBackground: true,
  },
];

const series = ["codex", "gallery", "atmosphere"];

test("type and series filters compose with AND semantics", () => {
  assert.deepEqual(
    filterThemes(themes, { type: "pure", series: "codex" }).map(
      (theme) => theme.id,
    ),
    ["codex-night"],
  );
  assert.deepEqual(
    filterThemes(themes, { type: "background", series: "gallery" }).map(
      (theme) => theme.id,
    ),
    ["gallery-rain"],
  );
});

test("search narrows the current facet result", () => {
  assert.deepEqual(
    filterThemes(themes, { type: "background", series: "all" }, "暖").map(
      (theme) => theme.id,
    ),
    ["codex-room"],
  );
  assert.deepEqual(
    filterThemes(themes, { type: "pure", series: "gallery" }, "雨"),
    [],
  );
});

test("invalid parameters fall back and legacy view remains readable", () => {
  assert.deepEqual(
    parseThemeFilters(new URLSearchParams("type=unknown&series=missing"), series),
    { type: "all", series: "all" },
  );
  assert.deepEqual(parseThemeFilters(new URLSearchParams("view=codex"), series), {
    type: "all",
    series: "codex",
  });
  assert.deepEqual(
    parseThemeFilters(new URLSearchParams("view=background"), series),
    { type: "background", series: "all" },
  );
});

test("filter links preserve the other dimension and omit defaults", () => {
  assert.equal(
    themeFilterHref({ type: "background", series: "codex" }),
    "/?type=background&series=codex#gallery",
  );
  assert.equal(themeFilterHref({ type: "all", series: "all" }), "/#gallery");
});
