import type { Metadata } from "next";
import Link from "next/link";
import CommandRow from "@/components/CommandRow";
import DesktopDownloads from "@/components/DesktopDownloads";
import JsonLd from "@/components/JsonLd";
import {
  CLAUDE_MARKETPLACE_COMMAND,
  CLAUDE_PLUGIN_INSTALL_COMMAND,
  CODEX_MARKETPLACE_COMMAND,
  CODEX_PLUGIN_INSTALL_COMMAND,
  REPO_URL,
  SITE_URL,
  SOCIAL_IMAGE,
  WELL_KNOWN_SKILLS_URL,
} from "@/lib/site";

const DESCRIPTION =
  "自动识别 macOS 或 Windows 桌面版本，独立安装豆皮 CLI，并在 Codex 或 Claude Code 中使用主题 Skill。";

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
        <p>下载与你当前系统匹配的桌面应用，或单独安装 CLI；两条安装链互不依赖。</p>
      </header>

      <section className="doc-section" id="download">
        <div className="section-title-row">
          <div>
            <p className="step-number">01</p>
            <h2>下载桌面应用</h2>
          </div>
          <span className="availability-note">macOS · Windows</span>
        </div>
        <p>浏览器会在本地识别操作系统和可用的处理器信息，优先推荐正确版本；所有下载都来自同一 GitHub Release。</p>
        <DesktopDownloads />
        <p className="fine-print">识别结果只改变推荐项，不会自动下载。无法识别或使用兼容模式时，请手动选择对应版本。</p>
      </section>

      <section className="doc-section" id="cli">
        <p className="step-number">02</p>
        <div className="section-title-row">
          <h2>独立安装 CLI</h2>
          <span className="availability-note">可选 · 不安装桌面应用</span>
        </div>
        <p>CLI 是给自动化、主题创作和 Agent 使用的独立工具。主题创作、检查、预览、打包和安装可跨平台使用；实时应用只支持 macOS/Windows，离线克隆只支持 macOS。安装 CLI 不会安装或修改桌面应用，安装桌面应用也不会写入 CLI。</p>
        <div className="skill-grid">
          <article>
            <h3>Windows · Scoop</h3>
            <p>Scoop 会选择 x64、x86 或 ARM64 包，并把命令加入当前用户的 PATH。</p>
            <CommandRow
              label="安装 CLI"
              command="scoop install https://github.com/IchenDEV/doubao-skin/releases/latest/download/doubao-skin.json"
            />
          </article>
          <article>
            <h3>macOS / Linux</h3>
            <p>安装脚本识别 macOS 通用版或 Linux x64/ARM64，并验证 SHA-256。</p>
            <CommandRow
              label="安装 CLI"
              command="curl -fsSL https://raw.githubusercontent.com/IchenDEV/doubao-skin/main/scripts/install-cli.sh | sh"
            />
          </article>
        </div>
        <p className="fine-print">Agent 可以先检查 `doubao-skin` 是否在 PATH；缺失时再按当前系统选择上述安装命令。</p>
      </section>

      <section className="doc-section">
        <p className="step-number">03</p>
        <h2>首次打开</h2>
        <ol className="numbered-list">
          <li>macOS：打开 DMG，把“豆皮”拖入“应用程序”；首次被拦截时前往「系统设置 → 隐私与安全性」选择“仍要打开”。</li>
          <li>Windows：解压 ZIP，保持 `doubao-skin.exe`、`themes` 与 `licenses` 位于同一目录，再运行桌面程序。</li>
          <li>两边都不需要关闭系统安全功能；如果下载被阻止，请先确认文件来自本项目的 GitHub Release。</li>
        </ol>
      </section>

      <section className="doc-section">
        <p className="step-number">04</p>
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
            <p className="step-number">05</p>
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
          Skill 会优先发现 PATH 中的独立 CLI；桌面应用与 CLI 可以只安装其中一个。安装第三方插件前请先查看源码和权限。开发者可以从
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
