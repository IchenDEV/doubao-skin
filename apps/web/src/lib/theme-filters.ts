export type ThemeTypeFilter = "all" | "pure" | "background";
export type ThemeTargetFilter = "all" | "doubao" | "doubao-work" | "workbuddy";

export interface ThemeFilters {
  type: ThemeTypeFilter;
  series: string;
  target: ThemeTargetFilter;
}

export interface FilterableTheme {
  id: string;
  name: string;
  description: string;
  author: string;
  category: string;
  tags: string[];
  hasBackground: boolean;
  targets: Record<string, { supportLevel: "unsupported" | "shared" | "tailored" }>;
}

type SearchParameters =
  | URLSearchParams
  | Readonly<Record<string, string | undefined>>;

const TYPE_FILTERS = new Set<ThemeTypeFilter>([
  "all",
  "pure",
  "background",
]);
const TARGET_FILTERS = new Set<ThemeTargetFilter>([
  "all",
  "doubao",
  "doubao-work",
  "workbuddy",
]);

function parameter(params: SearchParameters, key: string): string | undefined {
  if (params instanceof URLSearchParams) return params.get(key) ?? undefined;
  return params[key];
}

export function parseThemeFilters(
  params: SearchParameters,
  availableSeries: Iterable<string>,
): ThemeFilters {
  const seriesOptions = new Set(["all", ...availableSeries]);
  const legacy = parameter(params, "view") ?? "";
  const requestedType = parameter(params, "type");
  const requestedSeries = parameter(params, "series");
  const requestedTarget = parameter(params, "target") ?? "all";

  const typeCandidate =
    requestedType ??
    (legacy === "pure" || legacy === "background" ? legacy : "all");
  const seriesCandidate =
    requestedSeries ??
    (legacy && !["all", "pure", "background"].includes(legacy)
      ? legacy
      : "all");

  return {
    type: TYPE_FILTERS.has(typeCandidate as ThemeTypeFilter)
      ? (typeCandidate as ThemeTypeFilter)
      : "all",
    series: seriesOptions.has(seriesCandidate) ? seriesCandidate : "all",
    target: TARGET_FILTERS.has(requestedTarget as ThemeTargetFilter)
      ? (requestedTarget as ThemeTargetFilter)
      : "all",
  };
}

export function themeFilterHref(filters: ThemeFilters): string {
  const params = new URLSearchParams();
  if (filters.type !== "all") params.set("type", filters.type);
  if (filters.series !== "all") params.set("series", filters.series);
  if (filters.target !== "all") params.set("target", filters.target);
  const query = params.toString();
  return `/${query ? `?${query}` : ""}#gallery`;
}

export function filterThemes<T extends FilterableTheme>(
  themes: readonly T[],
  filters: ThemeFilters,
  query = "",
): T[] {
  const normalized = query.trim().toLocaleLowerCase();
  return themes.filter((theme) => {
    if (filters.type === "background" && !theme.hasBackground) return false;
    if (filters.type === "pure" && theme.hasBackground) return false;
    if (filters.series !== "all" && theme.category !== filters.series) {
      return false;
    }
    if (
      filters.target !== "all" &&
      theme.targets[filters.target]?.supportLevel !== "shared" &&
      theme.targets[filters.target]?.supportLevel !== "tailored"
    ) {
      return false;
    }
    if (!normalized) return true;
    return [
      theme.name,
      theme.id,
      theme.description,
      theme.author,
      ...theme.tags,
    ].some((value) => value.toLocaleLowerCase().includes(normalized));
  });
}
