export type DesktopDownloadKey =
  | "macos"
  | "windows-x64"
  | "windows-x86"
  | "windows-arm64";

export type DesktopDownload = {
  key: DesktopDownloadKey;
  label: string;
  shortLabel: string;
  detail: string;
  href: string;
};

const RELEASE_DOWNLOAD =
  "https://github.com/IchenDEV/doubao-skin/releases/latest/download";

export const DESKTOP_DOWNLOADS: readonly DesktopDownload[] = [
  {
    key: "macos",
    label: "macOS 通用版",
    shortLabel: "macOS",
    detail: "Apple 芯片与 Intel",
    href: `${RELEASE_DOWNLOAD}/Doubao-Skin-macOS-universal.dmg`,
  },
  {
    key: "windows-x64",
    label: "Windows x64",
    shortLabel: "Windows x64",
    detail: "大多数 Windows 电脑",
    href: `${RELEASE_DOWNLOAD}/Doubao-Skin-Windows-x64.zip`,
  },
  {
    key: "windows-arm64",
    label: "Windows ARM64",
    shortLabel: "Windows ARM64",
    detail: "骁龙与其他 ARM 电脑",
    href: `${RELEASE_DOWNLOAD}/Doubao-Skin-Windows-arm64.zip`,
  },
  {
    key: "windows-x86",
    label: "Windows x86",
    shortLabel: "Windows x86",
    detail: "仅限 32 位 Windows",
    href: `${RELEASE_DOWNLOAD}/Doubao-Skin-Windows-x86.zip`,
  },
];

export type PlatformEvidence = {
  platform?: string;
  userAgent?: string;
  architecture?: string;
  bitness?: string;
  maxTouchPoints?: number;
};

export function detectDesktopDownload({
  platform = "",
  userAgent = "",
  architecture = "",
  bitness = "",
  maxTouchPoints = 0,
}: PlatformEvidence): DesktopDownloadKey | null {
  const identity = `${platform} ${userAgent}`.toLowerCase();
  if (/iphone|ipad|android/.test(identity)) return null;
  if (platform === "MacIntel" && maxTouchPoints > 1) return null;
  if (/mac|darwin/.test(identity)) return "macos";
  if (!/win/.test(identity)) return null;

  const normalizedArchitecture = architecture.toLowerCase();
  if (normalizedArchitecture.includes("arm") || /arm64/.test(identity)) {
    return "windows-arm64";
  }
  if (bitness === "32" || /windows[^;)]*;[^;)]*i[3-6]86/.test(identity)) {
    return "windows-x86";
  }
  return "windows-x64";
}

export function desktopDownload(key: DesktopDownloadKey) {
  return DESKTOP_DOWNLOADS.find((download) => download.key === key);
}
