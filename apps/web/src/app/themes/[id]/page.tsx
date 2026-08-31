import Link from "next/link";
import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { CATEGORIES, getAdjacentThemes, getAllThemes, getTheme } from "@/lib/db";
import ThemeMockup from "@/components/ThemeMockup";
import CopyButton from "@/components/CopyButton";
import JsonLd from "@/components/JsonLd";
import type { Theme } from "@/lib/types";
import { supportedTargets, supportCopy } from "@/lib/theme-targets";
import { SITE_URL, SOCIAL_IMAGE } from "@/lib/site";

export const dynamic = "force-static";

export function generateStaticParams() {
  return getAllThemes().map((t) => ({ id: t.id }));
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ id: string }>;
}): Promise<Metadata> {
  const { id } = await params;
  const theme = getTheme(id);
  if (!theme) return {};
  const title = `${theme.name} · 豆皮`;
  const url = `${SITE_URL}/themes/${theme.id}`;
  const image = theme.previewDetail ?? theme.bgDetail ?? SOCIAL_IMAGE;
  return {
    title: theme.name,
    description: theme.description,
    alternates: { canonical: `/themes/${theme.id}` },
    openGraph: {
      type: "article",
      locale: "zh_CN",
      url,
      title,
      description: theme.description,
      images: [image],
    },
    twitter: {
      card: "summary_large_image",
      title,
      description: theme.description,
      images: [image],
    },
  };
}

const SWATCH_LABELS: [keyof Theme["colors"], string][] = [
  ["base", "底色"],
  ["base2", "侧栏"],
  ["primary", "卡片"],
  ["float", "浮层"],
  ["text", "正文"],
  ["muted", "弱文"],
  ["accent", "强调"],
  ["brand", "品牌"],
];

export default async function ThemePage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const theme = getTheme(id);
  if (!theme) return notFound();
  const { prev, next } = getAdjacentThemes(id);
  const c = theme.colors;
  const categoryLabel = CATEGORIES[theme.category] ?? theme.category;

  return (
    <main className="wrap detail">
      <JsonLd
        value={{
          "@context": "https://schema.org",
          "@type": "CreativeWork",
          name: theme.name,
          description: theme.description,
          url: `${SITE_URL}/themes/${theme.id}`,
          author: { "@type": "Person", name: theme.author },
          version: theme.version,
          image: theme.previewDetail ? `${SITE_URL}${theme.previewDetail}` : undefined,
          genre: categoryLabel,
          keywords: theme.tags.join(", "),
          inLanguage: "zh-CN",
        }}
      />
      <Link href="/#gallery" className="back">
        ← 返回主题索引
      </Link>

      <header className="detail-head">
        <p className="micro">
          {categoryLabel} · No. {String(theme.sortOrder + 1).padStart(2, "0")}
        </p>
        <h1>{theme.name}</h1>
        <p className="detail-id mono">{theme.id}</p>
        <p className="lede">{theme.description}</p>
        <div className="detail-chips">
          <span className="tag-chip">
            <i style={{ background: c.brand }} />
            {categoryLabel}
          </span>
          {theme.hasBackground ? (
            <span className="tag-chip">氛围背景 · veil {theme.veil ?? "-"}</span>
          ) : (
            <span className="tag-chip">纯色</span>
          )}
          {theme.isDefaultPalette && <span className="tag-chip">应用默认色板</span>}
          {supportedTargets(theme).map((target) => (
            <span className="tag-chip" key={target.id}>
              {target.label} · {supportCopy(target.support)}
            </span>
          ))}
        </div>
      </header>

      <div className="detail-grid">
        <ThemeMockup theme={theme} variant="detail" />
        <aside>
          <section className="panel">
            <h2>安装</h2>
            <p className="panel-note">
              下载原生 macOS 应用，打开后选择“{theme.name}”并点击“应用主题”。无需额外运行环境。
            </p>
            <a
              className="download-button is-full is-primary"
              href={`doubao-skin://apply/${theme.id}`}
            >
              在本地应用中打开
            </a>
            <Link className="download-button is-full is-secondary" href="/guide#download">
              前往使用与下载
            </Link>
          </section>

          <section className="panel">
            <h2>色板</h2>
            <div className="swatch-list">
              {SWATCH_LABELS.map(([key, label]) => (
                <CopyButton key={key} text={c[key]} className="swatch-row">
                  <span className="swatch-chip">
                    <i style={{ background: c[key] }} />
                  </span>
                  <span className="swatch-name">{label}</span>
                  <span className="swatch-value">{c[key]}</span>
                </CopyButton>
              ))}
            </div>
            <p className="panel-note">点击任意一行复制色值。</p>
          </section>

          <section className="panel">
            <h2>档案</h2>
            <dl className="archive">
              <dt>版本</dt>
              <dd>{theme.version}</dd>
              <dt>作者</dt>
              <dd>{theme.author}</dd>
              <dt>系列</dt>
              <dd>{categoryLabel}</dd>
              <dt>标签</dt>
              <dd>{theme.tags.join(" · ")}</dd>
              <dt>适用应用</dt>
              <dd>
                {supportedTargets(theme)
                  .map((target) => `${target.label}（${supportCopy(target.support)}）`)
                  .join(" · ")}
              </dd>
              <dt>背景图</dt>
              <dd>
                {theme.hasBackground
                  ? `原创生成 · veil ${theme.veil ?? "-"}`
                  : "无"}
              </dd>
              {theme.inspiredBy && (
                <>
                  <dt>灵感来源</dt>
                  <dd>{theme.inspiredBy}</dd>
                </>
              )}
              {theme.sourceDownloads != null && (
                <>
                  <dt>原站热度</dt>
                  <dd>
                    {theme.sourceDownloads.toLocaleString()} 次下载（
                    {theme.sourceSnapshot} 快照）
                  </dd>
                </>
              )}
              {theme.sourceUrl && (
                <>
                  <dt>参考链接</dt>
                  <dd>
                    <a href={theme.sourceUrl} target="_blank" rel="noreferrer">
                      查看来源 ↗
                    </a>
                  </dd>
                </>
              )}
            </dl>
          </section>
        </aside>
      </div>

      <nav className="prevnext">
        {prev ? (
          <Link className="pn" href={`/themes/${prev.id}`}>
            <span className="arrow">←</span>
            <span>
              <small>上一套</small>
              <strong>{prev.name}</strong>
            </span>
          </Link>
        ) : (
          <span />
        )}
        {next ? (
          <Link className="pn next" href={`/themes/${next.id}`}>
            <span className="arrow">→</span>
            <span>
              <small>下一套</small>
              <strong>{next.name}</strong>
            </span>
          </Link>
        ) : (
          <span />
        )}
      </nav>
    </main>
  );
}
