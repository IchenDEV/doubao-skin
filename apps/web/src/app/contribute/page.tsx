import type { Metadata } from "next";
import Link from "next/link";
import CommandRow from "@/components/CommandRow";
import JsonLd from "@/components/JsonLd";
import {
  CODEX_MARKETPLACE_COMMAND,
  CODEX_PLUGIN_INSTALL_COMMAND,
  CONTRIBUTION_DOC_URL,
  REPO_URL,
  SITE_URL,
  SOCIAL_IMAGE,
} from "@/lib/site";

const DESCRIPTION =
  "使用主题创作 Skill 或 Rust CLI 创建、检查并通过 GitHub Pull Request 投稿新的豆皮。";

export const metadata: Metadata = {
  title: "创作与投稿",
  description: DESCRIPTION,
  alternates: { canonical: "/contribute" },
  openGraph: {
    type: "website",
    locale: "zh_CN",
    url: `${SITE_URL}/contribute`,
    title: "创作与投稿 · 豆皮",
    description: DESCRIPTION,
    images: [{ url: SOCIAL_IMAGE, width: 1200, height: 675 }],
  },
  twitter: {
    card: "summary_large_image",
    title: "创作与投稿 · 豆皮",
    description: DESCRIPTION,
    images: [SOCIAL_IMAGE],
  },
};

const STEPS = [
  ["创建主题", "使用 create-doubao-theme Skill 把视觉要求生成到新的 themes/<theme-id>/ 目录，并补齐作者与素材许可。"],
  ["预览与检查", "运行严格检查并重新生成 1200 × 675 预览，确认主题标准、明暗外观和素材引用。"],
  ["同步网站目录", "运行网站 sync 命令，把主题清单、预览和安装包同步到生成目录并一并提交。"],
  ["Fork、分支并推送", "Fork IchenDEV/doubao-skin，从最新主分支建立主题分支，只推送本次主题与同步结果。"],
  ["发起 Pull Request", "说明设计方向、素材来源和实际验证结果；仓库检查与审核通过后，主题才会进入在线主题库。"],
];

export default function ContributePage() {
  return (
    <main className="reading-page">
      <JsonLd
        value={{
          "@context": "https://schema.org",
          "@type": "WebPage",
          name: "豆皮创作与投稿",
          url: `${SITE_URL}/contribute`,
          description: "豆皮的创作、检查、素材许可和 GitHub 投稿流程。",
          inLanguage: "zh-CN",
        }}
      />
      <header className="reading-head">
        <p className="eyebrow">创作者入口</p>
        <h1>创作与投稿</h1>
        <p>主题通过 GitHub Pull Request 投稿。网站暂不接收直接上传，仓库检查通过后才会进入在线主题库。</p>
      </header>

      <section className="doc-section contribution-flow">
        {STEPS.map(([title, description], index) => (
          <article key={title} className="contribution-step">
            <span>{String(index + 1).padStart(2, "0")}</span>
            <div>
              <h2>{title}</h2>
              <p>{description}</p>
            </div>
          </article>
        ))}
      </section>

      <section className="doc-section">
        <h2>用 Skill 开始</h2>
        <p>先安装项目插件，再直接描述想要的颜色、明暗、气质和使用场景。Claude Code 的安装方式见使用指南。</p>
        <CommandRow label="Codex · 添加插件市场" command={CODEX_MARKETPLACE_COMMAND} />
        <CommandRow label="Codex · 安装插件" command={CODEX_PLUGIN_INSTALL_COMMAND} />
        <CommandRow
          label="创作示例"
          command="$create-doubao-theme 做一个墨绿底、低饱和、适合专注写作的豆包工作主题"
        />
      </section>

      <section className="doc-section">
        <h2>投稿前检查</h2>
        <ul className="check-list">
          <li>主题 ID 使用小写 kebab-case，并与目录名一致。</li>
          <li>名称、描述、作者、版本、类型和系列信息完整。</li>
          <li>预览图为 1200 × 675，且与实际主题风格一致。</li>
          <li>没有官方应用资源、私人内容或来源不明的素材。</li>
          <li>只包含主题清单引用的素材与必要许可证文件。</li>
          <li>已运行网站同步命令，并提交本主题对应的生成目录变更。</li>
        </ul>
        <div className="action-row">
          <a className="secondary-button" href={CONTRIBUTION_DOC_URL} target="_blank" rel="noreferrer">
            阅读完整投稿文档 ↗
          </a>
          <a className="secondary-button" href={`${REPO_URL}/pulls`} target="_blank" rel="noreferrer">
            查看 Pull Requests ↗
          </a>
        </div>
      </section>

      <nav className="reading-next">
        <span>只是想安装主题？</span>
        <Link href="/guide">前往使用与下载 →</Link>
      </nav>
    </main>
  );
}
