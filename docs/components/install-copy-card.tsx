"use client";

import { useEffect, useState } from "react";

const installCommand = "curl -fsSL ay.dog | bash";
const copiedResetDelayMs = 1400;
const clipboardWriteTimeoutMs = 800;

function timeoutAfter(delayMs: number): Promise<never> {
  return new Promise((_, reject) => {
    window.setTimeout(() => reject(new Error("Clipboard write timed out")), delayMs);
  });
}

async function copyWithFallback(value: string): Promise<void> {
  if (navigator.clipboard) {
    try {
      await Promise.race([navigator.clipboard.writeText(value), timeoutAfter(clipboardWriteTimeoutMs)]);
      return;
    } catch {
    }
  }

  const textarea = document.createElement("textarea");
  textarea.value = value;
  textarea.setAttribute("readonly", "true");
  textarea.style.position = "fixed";
  textarea.style.left = "-9999px";
  document.body.appendChild(textarea);
  textarea.select();

  try {
    document.execCommand("copy");
  } finally {
    document.body.removeChild(textarea);
  }
}

export function InstallCopyCard() {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!copied) {
      return undefined;
    }

    const resetTimer = window.setTimeout(() => setCopied(false), copiedResetDelayMs);

    return () => window.clearTimeout(resetTimer);
  }, [copied]);

  async function copyInstallCommand() {
    await copyWithFallback(installCommand);
    setCopied(true);
  }

  return (
    <button
      className="install-card"
      type="button"
      onClick={copyInstallCommand}
      aria-label={`Copy quick install command: ${installCommand}`}
    >
      <span className="install-label">{copied ? "COPIED" : "QUICK INSTALL"}</span>
      <span className="install-command">
        {installCommand} <span className="command-dot" />
      </span>
    </button>
  );
}
