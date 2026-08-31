import { REPO_URL, SITE_NAME } from "@/lib/site";

export default function SiteFooter() {
  return (
    <footer className="site-footer">
      <div className="wrap footer-inner">
        <p>
          {SITE_NAME} · 当前适用于 macOS 版「豆包」「豆包工作」与 WorkBuddy，Windows 版本稍后推出。非官方产品，不修改原始应用安装包。
          主题与背景图遵循仓库 <a href={`${REPO_URL || "#"}`}>MIT License</a>。
        </p>
        <p className="footer-links">
          <a href="/guide">使用与下载</a>
          <a href="/contribute">创作与投稿</a>
          {REPO_URL && (
            <a href={REPO_URL} target="_blank" rel="noreferrer">
              GitHub ↗
            </a>
          )}
          <a href="https://github.com/Fei-Away/Codex-Dream-Skin" target="_blank" rel="noreferrer">
            灵感致谢 Codex Dream Skin ↗
          </a>
        </p>
      </div>
    </footer>
  );
}
