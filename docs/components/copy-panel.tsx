"use client";

import { useEffect, useState, type ReactNode } from "react";
import { copyText, isCopyFailure } from "@/lib/clipboard";

const copiedResetDelayMs = 1400;
type CopyState = "idle" | "copied" | "error";
type CopyPanelMode = "button" | "overlay";

type CopyPanelProps = {
  value: string;
  className: string;
  ariaLabel: string;
  mode?: CopyPanelMode;
  children: ReactNode;
};

export function CopyPanel({ value, className, ariaLabel, mode = "button", children }: CopyPanelProps) {
  const [copyState, setCopyState] = useState<CopyState>("idle");

  useEffect(() => {
    if (copyState === "idle") {
      return undefined;
    }

    const resetTimer = window.setTimeout(() => setCopyState("idle"), copiedResetDelayMs);

    return () => window.clearTimeout(resetTimer);
  }, [copyState]);

  async function copyPanelValue() {
    try {
      await copyText(value);
      setCopyState("copied");
    } catch (error: unknown) {
      if (!isCopyFailure(error)) {
        throw error;
      }

      setCopyState("error");
    }
  }

  if (mode === "overlay") {
    return (
      <div className={className} data-copy-state={copyState}>
        {children}
        <button
          aria-label={ariaLabel}
          className="copy-panel-control"
          onClick={copyPanelValue}
          type="button"
        >
          <span className="copy-live-region" role="status">
            {copyState === "copied" ? "Copied" : null}
            {copyState === "error" ? "Copy failed" : null}
          </span>
        </button>
      </div>
    );
  }

  return (
    <button
      aria-label={ariaLabel}
      className={className}
      data-copy-state={copyState}
      onClick={copyPanelValue}
      type="button"
    >
      {children}
      <span className="copy-live-region" role="status">
        {copyState === "copied" ? "Copied" : null}
        {copyState === "error" ? "Copy failed" : null}
      </span>
    </button>
  );
}
