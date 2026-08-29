"use client";

import Image from "next/image";
import Link from "next/link";
import { useEffect, useState } from "react";
import { usePathname } from "next/navigation";
import { REPO_URL, SITE_NAME } from "@/lib/site";
import {
  parseThemeFilters,
  themeFilterHref,
  type ThemeFilters,
  type ThemeTypeFilter,
} from "@/lib/theme-filters";

type IconName = "grid" | "book" | "brush" | "image" | "drop" | "folder" | "github";

function NavIcon({ name }: { name: IconName }) {
  const common = {
    width: 19,
    height: 19,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 1.7,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    "aria-hidden": true,
  };
  const paths: Record<IconName, React.ReactNode> = {
    grid: <><rect x="4" y="4" width="6" height="6" rx="1" /><rect x="14" y="4" width="6" height="6" rx="1" /><rect x="4" y="14" width="6" height="6" rx="1" /><rect x="14" y="14" width="6" height="6" rx="1" /></>,
    book: <><path d="M4 5.5A2.5 2.5 0 0 1 6.5 3H11v16H6.5A2.5 2.5 0 0 0 4 21.5v-16Z" /><path d="M20 5.5A2.5 2.5 0 0 0 17.5 3H13v16h4.5a2.5 2.5 0 0 1 2.5 2.5v-16Z" /></>,
    brush: <><path d="m14 5 5 5M13 6l5 5-8.5 8.5-5.5.5.5-5.5L13 6Z" /><path d="M5 15c-1.5 0-2.5 1-2.5 2.5S1.5 20 1.5 20H6" /></>,
    image: <><rect x="3" y="4" width="18" height="16" rx="2" /><circle cx="8.5" cy="9" r="1.5" /><path d="m21 15-4.5-4.5L6 20" /></>,
    drop: <path d="M12 3s6 6.4 6 11a6 6 0 0 1-12 0c0-4.6 6-11 6-11Z" />,
    folder: <path d="M3 7.5h7l2-2h9v13H3v-11Z" />,
    github: <><circle cx="12" cy="12" r="9" /><path d="M8.5 19c.4-1.2.4-2.1 0-2.8-2.1-.3-3.3-1.4-3.3-3.9 0-1 .4-1.9 1.1-2.6-.2-.7-.2-1.6.1-2.5 1.1 0 2 .5 2.6 1a8.8 8.8 0 0 1 6 0c.7-.5 1.5-1 2.6-1 .3.9.3 1.8.1 2.5.7.7 1.1 1.6 1.1 2.6 0 2.5-1.2 3.6-3.3 3.9-.4.7-.4 1.6 0 2.8" /></>,
  };
  return <svg {...common}>{paths[name]}</svg>;
}

function FilterLinks({
  state,
  series,
  onChange,
}: {
  state: ThemeFilters;
  series: { key: string; label: string }[];
  onChange: (state: ThemeFilters) => void;
}) {
  const typeOptions: {
    key: ThemeTypeFilter;
    label: string;
    icon: IconName;
  }[] = [
    { key: "all", label: "全部类型", icon: "grid" },
    { key: "pure", label: "纯色", icon: "drop" },
    { key: "background", label: "有背景", icon: "image" },
  ];
  return (
    <div className="filter-sections">
      <section className="filter-group" aria-labelledby="type-filter-title">
        <h2 id="type-filter-title">类型</h2>
        {typeOptions.map((item) => (
          <Link
            key={item.key}
            href={themeFilterHref({ type: item.key, series: state.series })}
            className={state.type === item.key ? "is-active" : undefined}
            aria-current={state.type === item.key ? "page" : undefined}
            onClick={() => onChange({ ...state, type: item.key })}
          >
            <NavIcon name={item.icon} />{item.label}
          </Link>
        ))}
      </section>
      <section className="filter-group" aria-labelledby="series-filter-title">
        <h2 id="series-filter-title">系列</h2>
        <Link
          href={themeFilterHref({ type: state.type, series: "all" })}
          className={state.series === "all" ? "is-active" : undefined}
          aria-current={state.series === "all" ? "page" : undefined}
          onClick={() => onChange({ ...state, series: "all" })}
        >
          <NavIcon name="folder" />全部系列
        </Link>
        {series.map((item) => (
          <Link
            key={item.key}
            href={themeFilterHref({ type: state.type, series: item.key })}
            className={state.series === item.key ? "is-active" : undefined}
            aria-current={state.series === item.key ? "page" : undefined}
            onClick={() => onChange({ ...state, series: item.key })}
          >
            <span className="series-dot" aria-hidden="true" />{item.label}
          </Link>
        ))}
      </section>
    </div>
  );
}

export default function SiteHeader({
  series,
}: {
  series: { key: string; label: string }[];
}) {
  const pathname = usePathname();
  const [filters, setFilters] = useState<ThemeFilters>({ type: "all", series: "all" });
  const [filtersOpen, setFiltersOpen] = useState(false);

  useEffect(() => {
    const sync = () => {
      setFilters(
        parseThemeFilters(
          new URLSearchParams(window.location.search),
          series.map((item) => item.key),
        ),
      );
      setFiltersOpen(false);
    };
    sync();
    window.addEventListener("popstate", sync);
    return () => window.removeEventListener("popstate", sync);
  }, [pathname, series]);

  const navClass = (path: string) => (pathname === path ? "is-active" : undefined);

  return (
    <header className="site-header" aria-label="网站导航">
      <div className="sidebar-inner">
        <Link href="/" className="brand">
          <Image className="brand-mark" src="/app-icon.png" width={34} height={34} alt="" priority />
          <strong>{SITE_NAME}</strong>
        </Link>

        <nav className="primary-nav" aria-label="主要页面">
          <Link className={navClass("/")} aria-current={pathname === "/" ? "page" : undefined} href="/"><NavIcon name="grid" />主题库</Link>
          <Link className={navClass("/guide")} aria-current={pathname === "/guide" ? "page" : undefined} href="/guide"><NavIcon name="book" />使用与下载</Link>
          <Link className={navClass("/contribute")} aria-current={pathname === "/contribute" ? "page" : undefined} href="/contribute"><NavIcon name="brush" />创作与投稿</Link>
        </nav>

        {pathname === "/" ? (
          <div className={`filters-shell${filtersOpen ? " is-open" : ""}`}>
            <button
              className="mobile-filter-trigger"
              type="button"
              aria-expanded={filtersOpen}
              aria-controls="theme-filter-panel"
              onClick={() => setFiltersOpen((open) => !open)}
            >
              筛选
            </button>
            <div className="filter-panel" id="theme-filter-panel">
              <FilterLinks
                state={filters}
                series={series}
                onChange={(state) => {
                  setFilters(state);
                  setFiltersOpen(false);
                }}
              />
            </div>
          </div>
        ) : null}

        <nav className="sidebar-footer" aria-label="项目链接">
          <a href={REPO_URL} target="_blank" rel="noreferrer"><NavIcon name="github" />GitHub</a>
        </nav>
      </div>
    </header>
  );
}
