import type { Theme, ThemeTargetId, ThemeTargetSupport } from "./types";

export const THEME_TARGETS: readonly { id: ThemeTargetId; label: string }[] = [
  { id: "doubao", label: "豆包" },
  { id: "doubao-work", label: "豆包工作" },
  { id: "workbuddy", label: "WorkBuddy" },
];

export function supportedTargets(theme: Theme) {
  return THEME_TARGETS.flatMap((target) => {
    const support = theme.targets[target.id];
    return support && support.supportLevel !== "unsupported"
      ? [{ ...target, support }]
      : [];
  });
}

export function supportCopy(support: ThemeTargetSupport): string {
  if (support.declaration === "legacy-inferred") return "兼容模式";
  return support.supportLevel === "tailored" ? "专属适配" : "支持";
}
