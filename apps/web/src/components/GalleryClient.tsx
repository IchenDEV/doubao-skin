"use client";

import { useMemo, useState } from "react";
import type { Theme } from "@/lib/types";
import { filterThemes } from "@/lib/theme-filters";
import ThemeCard from "./ThemeCard";

export default function GalleryClient({
  themes,
  initialType,
  initialSeries,
}: {
  themes: Theme[];
  initialType: string;
  initialSeries: string;
}) {
  const [query, setQuery] = useState("");

  const filtered = useMemo(
    () =>
      filterThemes(
        themes,
        {
          type:
            initialType === "pure" || initialType === "background"
              ? initialType
              : "all",
          series: initialSeries,
        },
        query,
      ),
    [themes, query, initialType, initialSeries],
  );

  return (
    <div className="gallery">
      <label className="search-field">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="11" cy="11" r="7" />
          <path d="M20 20 L16 16" />
        </svg>
        <input
          type="search"
          placeholder="搜索名称、作者或标签"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          aria-label="搜索主题"
        />
      </label>

      <div className="results-meta" aria-live="polite">
        <span>主题</span>
        <span>{filtered.length} 套</span>
      </div>

      {filtered.length > 0 ? (
        <div className="theme-table">
          <div className="theme-table-head" aria-hidden="true">
            <span>预览</span>
            <span>主题</span>
            <span>类型</span>
            <span />
            <span />
          </div>
          {filtered.map((theme) => (
            <ThemeCard key={theme.id} theme={theme} />
          ))}
        </div>
      ) : (
        <div className="empty">
          <strong>没有找到匹配的主题</strong>
          <span>试试清空搜索，或调整左侧的类型和系列。</span>
        </div>
      )}
    </div>
  );
}
