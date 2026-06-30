"use client";

import { useEffect, useState } from "react";
import { copyText, isCopyFailure } from "@/lib/clipboard";

const installCommand = "curl -fsSL https://stacc.fyi/install.sh | bash";
const copiedResetDelayMs = 1400;
type CopyState = "idle" | "copied" | "error";

export function InstallCopyCard() {
  const [copyState, setCopyState] = useState<CopyState>("idle");

  useEffect(() => {
    if (copyState === "idle") {
      return undefined;
    }

    const resetTimer = window.setTimeout(() => setCopyState("idle"), copiedResetDelayMs);

    return () => window.clearTimeout(resetTimer);
  }, [copyState]);

  async function copyInstallCommand() {
    try {
      await copyText(installCommand);
      setCopyState("copied");
    } catch (error: unknown) {
      if (!isCopyFailure(error)) {
        throw error;
      }

      setCopyState("error");
    }
  }

  return (
    <button
      className="install-card"
      type="button"
      onClick={copyInstallCommand}
      aria-label={`Copy quick install command: ${installCommand}`}
      data-copy-state={copyState}
    >
      <span className="install-label">
        {copyState === "copied" ? "COPIED" : null}
        {copyState === "error" ? "COPY FAILED" : null}
        {copyState === "idle" ? "QUICK INSTALL" : null}
      </span>
      <span className="install-command">
        <span className="install-command-text">{installCommand}</span>
        <span className="command-dot" />
      </span>
      <output className="copy-live-region">
        {copyState === "copied" ? "Copied" : null}
        {copyState === "error" ? "Copy failed" : null}
      </output>
    </button>
  );
}
