import type { Metadata } from "next";
import { getAllThemes } from "@/lib/db";
import { SITE_NAME, SITE_TAGLINE, SITE_URL } from "@/lib/site";
import GalleryClient from "@/components/GalleryClient";
import JsonLd from "@/components/JsonLd";
import { parseThemeFilters } from "@/lib/theme-filters";

export const metadata: Metadata = {
  alternates: { canonical: "/" },
};

export default async function HomePage({
  searchParams,
}: {
  searchParams: Promise<{ type?: string; series?: string; target?: string; view?: string }>;
}) {
  const themes = getAllThemes();
  const params = await searchParams;
  const filters = parseThemeFilters(
    params,
    themes
      .map((theme) => theme.category)
      .filter((category) => category !== "pure"),
  );

  return (
    <main className="library-page" id="gallery">
      <JsonLd
        value={{
          "@context": "https://schema.org",
          "@type": "CollectionPage",
          name: "豆皮库",
          description: SITE_TAGLINE,
          url: SITE_URL,
          mainEntity: {
            "@type": "ItemList",
            numberOfItems: themes.length,
            itemListElement: themes.map((theme, index) => ({
              "@type": "ListItem",
              position: index + 1,
              url: `${SITE_URL}/themes/${theme.id}`,
              name: theme.name,
            })),
          },
        }}
      />
      <section className="library-heading">
        <h1>主题库</h1>
        <p>{SITE_TAGLINE}</p>
      </section>

      <GalleryClient
        key={`${filters.type}:${filters.series}:${filters.target}`}
        themes={themes}
        initialType={filters.type}
        initialSeries={filters.series}
        initialTarget={filters.target}
      />
    </main>
  );
}
