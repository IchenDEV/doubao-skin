import type { Metadata } from "next";
import Link from "next/link";
import CommandRow from "@/components/CommandRow";
import JsonLd from "@/components/JsonLd";
import {
  CLAUDE_MARKETPLACE_COMMAND,
  CLAUDE_PLUGIN_INSTALL_COMMAND,
  CODEX_MARKETPLACE_COMMAND,
  CODEX_PLUGIN_INSTALL_COMMAND,
  MAC_APP_DOWNLOAD,
  REPO_URL,
  SITE_URL,
  SOCIAL_IMAGE,
  WELL_KNOWN_SKILLS_URL,
} from "@/lib/site";

const DESCRIPTION =
  "下载豆皮 macOS 通用版，完成首次打开、应用与恢复，并在 Codex 或 Claude Code 中安装主题 Skill。";

export const metadata: Metadata = {
  title: "使用与下载",
  description: DESCRIPTION,
  alternates: { canonical: "/guide" },
  openGraph: {
    type: "website",
    locale: "zh_CN",
    url: `${SITE_URL}/guide`,
    title: "使用与下载 · 豆皮",
    description: DESCRIPTION,
    images: [{ url: SOCIAL_IMAGE, width: 1200, height: 675 }],
  },
  twitter: {
    card: "summary_large_image",
    title: "使用与下载 · 豆皮",
    description: DESCRIPTION,
    images: [SOCIAL_IMAGE],
  },
};

export default function GuidePage() {
  return (
    <main className="reading-page">
      <JsonLd
        value={{
          "@context": "https://schema.org",
          "@type": "WebPage",
          name: "豆皮使用与下载",
          url: `${SITE_URL}/guide`,
          description: "豆皮的下载、安装、应用、恢复与 Skill 安装指南。",
          inLanguage: "zh-CN",
        }}
      />
      <header className="reading-head">
        <p className="eyebrow">使用指南</p>
        <h1>使用与下载</h1>
        <p>下载桌面应用，选择一套主题，再应用到 macOS 版「豆包」或「豆包工作」。</p>
      </header>

      <section className="doc-section" id="download">
        <div className="section-title-row">
          <div>
            <p className="step-number">01</p>
            <h2>下载桌面应用</h2>
          </div>
          <span className="coming-soon">Windows · Coming Soon</span>
        </div>
        <p>当前提供 macOS 通用版，同时支持 Apple 芯片和 Intel Mac。安装包由 GitHub Release 提供。</p>
        <a className="download-button" href={MAC_APP_DOWNLOAD}>
          下载 macOS 通用版
        </a>
        <p className="fine-print">下载会前往 GitHub。Windows 版本正在准备中，目前没有可用安装包。</p>
      </section>

      <section className="doc-section">
        <p className="step-number">02</p>
        <h2>首次打开</h2>
        <ol className="numbered-list">
          <li>解压下载的文件，把“豆皮”拖入“应用程序”文件夹。</li>
          <li>第一次打开时，在 Finder 中右键“豆皮”，再选择“打开”。</li>
          <li>如果系统再次询问，确认打开即可；不需要关闭系统安全功能。</li>
        </ol>
      </section>

      <section className="doc-section">
        <p className="step-number">03</p>
        <h2>应用与恢复</h2>
        <ol className="numbered-list">
          <li>先保存「豆包」或「豆包工作」里正在进行的内容。</li>
          <li>在“豆皮”中选择目标应用和喜欢的主题，确认预览。</li>
          <li>点击“应用主题”；需要还原时，点击“恢复默认”。</li>
        </ol>
        <p className="callout">应用过程中目标应用可能重新打开。恢复默认只清理正在运行的主题，不会删除已下载的主题包。</p>
      </section>

      <section className="doc-section" id="skills">
        <div className="section-title-row">
          <div>
            <p className="step-number">04</p>
            <h2>安装创作与应用 Skill</h2>
          </div>
          <span className="availability-note">Codex · Claude Code</span>
        </div>
        <p>添加项目插件市场后，一次安装即可获得主题创作和应用两个 Skill。</p>
        <div className="skill-grid">
          <article>
            <h3>Codex</h3>
            <p>在终端依次运行：</p>
            <CommandRow label="添加插件市场" command={CODEX_MARKETPLACE_COMMAND} />
            <CommandRow label="安装插件" command={CODEX_PLUGIN_INSTALL_COMMAND} />
            <p className="fine-print">
              更新：<code>codex plugin marketplace upgrade doubao-skin</code><br />
              移除：<code>codex plugin remove doubao-skin@doubao-skin</code>
            </p>
          </article>
          <article>
            <h3>Claude Code</h3>
            <p>在 Claude Code 中依次输入：</p>
            <CommandRow label="添加插件市场" command={CLAUDE_MARKETPLACE_COMMAND} />
            <CommandRow label="安装插件" command={CLAUDE_PLUGIN_INSTALL_COMMAND} />
            <p className="fine-print">
              更新：<code>/plugin marketplace update doubao-skin</code><br />
              移除：<code>/plugin uninstall doubao-skin@doubao-skin</code>
            </p>
          </article>
        </div>
        <div className="skill-grid">
          <article>
            <h3>create-doubao-theme</h3>
            <p>根据自然语言需求创建、检查、重绘预览并打包主题。</p>
            <code>$create-doubao-theme 做一个暖黄色、适合夜间阅读的主题</code>
          </article>
          <article>
            <h3>apply-doubao-theme</h3>
            <p>列出、安装、应用或恢复主题；执行有影响的操作前会先征得确认。</p>
            <code>$apply-doubao-theme 列出本机可用主题</code>
          </article>
        </div>
        <p className="fine-print">
          Skill 当前依赖 macOS 版桌面应用；Windows 支持稍后推出。安装第三方插件前请先查看源码和权限。开发者可以从
          {" "}<a href={`${REPO_URL}/tree/main/plugins/doubao-skin/skills`} target="_blank" rel="noreferrer">GitHub 查看 Skill 源文件</a>，也可以读取
          {" "}<a href={WELL_KNOWN_SKILLS_URL}>Agent Skills Discovery Draft 0.2.0 索引</a>。
        </p>
      </section>

      <nav className="reading-next">
        <span>想制作自己的主题？</span>
        <Link href="/contribute">查看创作与投稿流程 →</Link>
      </nav>
    </main>
  );
}
