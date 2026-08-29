import type { MetadataRoute } from "next";
import { getAllThemes } from "@/lib/db";
import { SITE_URL } from "@/lib/site";

export default function sitemap(): MetadataRoute.Sitemap {
  const topLevel: MetadataRoute.Sitemap = ["", "/guide", "/contribute"].map(
    (path) => ({ url: `${SITE_URL}${path}`, changeFrequency: "weekly" })
  );
  return [
    ...topLevel,
    ...getAllThemes().map((theme) => ({
      url: `${SITE_URL}/themes/${theme.id}`,
      changeFrequency: "monthly" as const,
    })),
  ];
}
