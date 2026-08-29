"use client";

import { useEffect, useRef, useState, type ReactNode } from "react";

export default function CopyButton({
  text,
  label = "复制",
  className = "",
  children,
}: {
  text: string;
  label?: string;
  className?: string;
  children?: ReactNode;
}) {
  const [copied, setCopied] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
    },
    []
  );

  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = text;
      ta.style.position = "fixed";
      ta.style.opacity = "0";
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
    }
    setCopied(true);
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => setCopied(false), 1600);
  }

  return (
    <button
      type="button"
      className={`copy-btn ${copied ? "is-copied" : ""} ${className}`}
      onClick={copy}
    >
      {children ?? (copied ? "已复制" : label)}
      {children && copied && <span className="copy-badge">已复制</span>}
    </button>
  );
}
