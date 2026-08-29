import type { Metadata, Viewport } from "next";
import "./globals.css";
import SiteHeader from "@/components/SiteHeader";
import SiteFooter from "@/components/SiteFooter";
import JsonLd from "@/components/JsonLd";
import { CATEGORY_ORDER, CATEGORIES, getAllThemes } from "@/lib/db";
import { SITE_NAME, SITE_TAGLINE, SITE_URL, SOCIAL_IMAGE } from "@/lib/site";

export const metadata: Metadata = {
  metadataBase: new URL(SITE_URL),
  title: {
    default: `${SITE_NAME} · 豆包与豆包工作主题库`,
    template: `%s · ${SITE_NAME}`,
  },
  description:
    "同时支持 macOS 版「豆包」与「豆包工作」的第三方主题工具：浏览、预览并安装纯色、氛围背景与编辑器配色主题。",
  alternates: { canonical: "/" },
  openGraph: {
    type: "website",
    locale: "zh_CN",
    url: SITE_URL,
    siteName: SITE_NAME,
    title: `${SITE_NAME} · 豆包与豆包工作主题库`,
    description: "浏览、预览并安装适用于 macOS 豆包与豆包工作的主题。",
    images: [{ url: SOCIAL_IMAGE, width: 1200, height: 675 }],
  },
  twitter: {
    card: "summary_large_image",
    title: `${SITE_NAME} · 豆包与豆包工作主题库`,
    description: "浏览、预览并安装适用于 macOS 豆包与豆包工作的主题。",
    images: [SOCIAL_IMAGE],
  },
};

export const viewport: Viewport = {
  colorScheme: "light dark",
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#ffffff" },
    { media: "(prefers-color-scheme: dark)", color: "#111210" },
  ],
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  const categories = new Set(getAllThemes().map((theme) => theme.category));
  const series = CATEGORY_ORDER.filter(
    (category) => category !== "pure" && categories.has(category)
  ).map((key) => ({ key, label: CATEGORIES[key] }));
  return (
    <html lang="zh-CN" data-scroll-behavior="smooth">
      <body>
        <JsonLd
          value={{
            "@context": "https://schema.org",
            "@type": "WebSite",
            name: SITE_NAME,
            url: SITE_URL,
            description: SITE_TAGLINE,
            inLanguage: "zh-CN",
          }}
        />
        <div className="site-shell">
          <SiteHeader series={series} />
          <div className="site-content">
            {children}
            <SiteFooter />
          </div>
        </div>
      </body>
    </html>
  );
}
