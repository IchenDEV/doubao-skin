export interface ThemeColors {
  base: string;
  base2: string;
  primary: string;
  float: string;
  text: string;
  muted: string;
  hairline: string;
  accent: string;
  accentHover: string;
  brand: string;
}

export type ThemeTargetId = "doubao" | "doubao-work" | "workbuddy";
export type ThemeSupportLevel = "unsupported" | "shared" | "tailored";

export interface ThemeTargetSupport {
  supportLevel: ThemeSupportLevel;
  declaration: "explicit" | "legacy-inferred";
  appearances: ("light" | "dark")[];
}

export interface Theme {
  id: string;
  name: string;
  description: string;
  version: string;
  author: string;
  category: string;
  tags: string[];
  schemaVersion: number;
  targets: Record<ThemeTargetId, ThemeTargetSupport>;
  hasBackground: boolean;
  veil: number | null;
  colors: ThemeColors;
  bgDetail: string | null;
  bgCard: string | null;
  previewDetail: string | null;
  previewCard: string | null;
  inspiredBy: string | null;
  sourceUrl: string | null;
  sourceDownloads: number | null;
  sourceSnapshot: string | null;
  isDefaultPalette: boolean;
  sortOrder: number;
}
