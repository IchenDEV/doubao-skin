import Link from "next/link";
import type { Theme } from "@/lib/types";
import ThemeMockup from "./ThemeMockup";

export default function ThemeCard({ theme }: { theme: Theme }) {
  return (
    <article className="theme-row" data-id={theme.id}>
      <Link
        href={`/themes/${theme.id}`}
        className="theme-row-preview"
        aria-label={`查看${theme.name}`}
      >
        <ThemeMockup theme={theme} variant="card" />
      </Link>
      <Link href={`/themes/${theme.id}`} className="theme-row-copy">
        <strong>{theme.name}</strong>
        <span>{theme.description}</span>
      </Link>
      <span className="theme-row-type">
        {theme.hasBackground ? "有背景" : "纯色"}
      </span>
      <Link href={`/themes/${theme.id}`} className="view-button">查看</Link>
      <details className="row-menu">
        <summary aria-label={`${theme.name}更多操作`}>•••</summary>
        <div className="row-menu-popover">
          <Link href={`/themes/${theme.id}`} className="row-menu-item">查看详情</Link>
          <Link className="row-menu-item" href="/guide#download">使用与下载</Link>
        </div>
      </details>
    </article>
  );
}
