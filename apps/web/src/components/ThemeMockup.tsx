import type { CSSProperties } from "react";
import type { Theme } from "@/lib/types";

/**
 * A miniature "Doubao Work" chat window rendered with the theme's real
 * design tokens. Atmosphere themes layer their background image beneath
 * the translucent surfaces, mirroring what the injector does in-app.
 */
export default function ThemeMockup({
  theme,
  variant = "card",
}: {
  theme: Theme;
  variant?: "card" | "detail";
}) {
  const generatedPreview = variant === "card"
    ? theme.previewCard ?? theme.previewDetail
    : theme.previewDetail ?? theme.previewCard;
  if (generatedPreview) {
    return (
      <div className={`mock mock-${variant}`} aria-hidden="true">
        <img className="mock-preview-image" src={generatedPreview} alt="" />
      </div>
    );
  }
  const c = theme.colors;
  const bg =
    (variant === "card" ? theme.bgCard ?? theme.bgDetail : theme.bgDetail ?? theme.bgCard) ??
    null;
  const style = {
    "--mk-base": c.base,
    "--mk-base2": c.base2,
    "--mk-primary": c.primary,
    "--mk-float": c.float,
    "--mk-text": c.text,
    "--mk-muted": c.muted,
    "--mk-hairline": c.hairline,
    "--mk-accent": c.accent,
    backgroundImage: bg ? `url(${bg})` : undefined,
  } as CSSProperties;

  return (
    <div className={`mock mock-${variant}`} style={style} aria-hidden="true">
      <div className="mock-window">
        <div className="mock-titlebar">
          <span className="mock-dot" />
          <span className="mock-dot" />
          <span className="mock-dot" />
        </div>
        <div className="mock-main">
          <div className="mock-sidebar">
            <span className="mock-side-logo" />
            <span className="mock-side-item is-active" />
            <span className="mock-side-item" />
            <span className="mock-side-item short" />
            <span className="mock-side-item" />
          </div>
          <div className="mock-chat">
            <div className="mock-msg peer">
              <span className="w55" />
              <span className="w35" />
            </div>
            <div className="mock-msg self">
              <span className="w45" />
            </div>
            <div className="mock-msg peer">
              <span className="w70" />
              <span className="w50" />
            </div>
            {variant === "detail" && (
              <>
                <div className="mock-msg self wide">
                  <span className="w65" />
                  <span className="w30" />
                </div>
                <div className="mock-msg peer">
                  <span className="w40" />
                </div>
              </>
            )}
            <div className="mock-composer">
              <span className="mock-composer-text" />
              <span className="mock-send" />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
