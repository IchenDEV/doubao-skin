"use client";

import { useEffect, useState } from "react";
import {
  DESKTOP_DOWNLOADS,
  desktopDownload,
  detectDesktopDownload,
  type DesktopDownloadKey,
} from "@/lib/downloads";

type NavigatorUAData = {
  platform?: string;
  getHighEntropyValues?: (
    hints: string[],
  ) => Promise<{ architecture?: string; bitness?: string }>;
};

export default function DesktopDownloads() {
  const [recommendation, setRecommendation] =
    useState<DesktopDownloadKey | null>(null);
  const [detectionComplete, setDetectionComplete] = useState(false);

  useEffect(() => {
    let active = true;
    const currentNavigator = navigator as Navigator & {
      userAgentData?: NavigatorUAData;
    };

    async function detect() {
      let architecture = "";
      let bitness = "";
      try {
        const details =
          await currentNavigator.userAgentData?.getHighEntropyValues?.([
            "architecture",
            "bitness",
          ]);
        architecture = details?.architecture ?? "";
        bitness = details?.bitness ?? "";
      } catch {
        // Browser privacy settings may decline high-entropy platform details.
      }

      if (!active) return;
      setRecommendation(
        detectDesktopDownload({
          platform:
            currentNavigator.userAgentData?.platform ??
            currentNavigator.platform,
          userAgent: currentNavigator.userAgent,
          architecture,
          bitness,
          maxTouchPoints: currentNavigator.maxTouchPoints,
        }),
      );
      setDetectionComplete(true);
    }

    void detect();
    return () => {
      active = false;
    };
  }, []);

  const recommendedDownload = recommendation
    ? desktopDownload(recommendation)
    : undefined;

  return (
    <div className="download-picker">
      <div className="download-recommendation" aria-live="polite">
        <p className="download-kicker">为此设备推荐</p>
        {recommendedDownload ? (
          <>
            <h3>{recommendedDownload.label}</h3>
            <p>{recommendedDownload.detail}。下载后按本页步骤安装桌面应用。</p>
            <a className="download-button is-primary" href={recommendedDownload.href}>
              下载 {recommendedDownload.shortLabel} 桌面版
            </a>
          </>
        ) : (
          <>
            <h3>{detectionComplete ? "请选择桌面版本" : "正在识别当前设备…"}</h3>
            <p>
              {detectionComplete
                ? "当前设备没有自动匹配的桌面版，仍可从下方手动选择。"
                : "识别结果只用于推荐，不会自动开始下载。"}
            </p>
          </>
        )}
      </div>

      <div className="download-options" aria-label="全部桌面版本">
        {DESKTOP_DOWNLOADS.map((download) => (
          <a
            className={
              download.key === recommendation
                ? "download-option is-recommended"
                : "download-option"
            }
            href={download.href}
            key={download.key}
          >
            <strong>{download.label}</strong>
            <span>{download.detail}</span>
          </a>
        ))}
      </div>
    </div>
  );
}
