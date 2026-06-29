"use client";

import { useEffect, useState } from "react";

type ThemeChoice = "system" | "light" | "dark";
type ResolvedTheme = "light" | "dark";
type ThemeToggleProps = {
  variant?: "label" | "icon";
};

const storageKey = "stacc-theme";
const choices: ThemeChoice[] = ["system", "light", "dark"];

function getSystemTheme(): ResolvedTheme {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applyTheme(choice: ThemeChoice) {
  const resolved = choice === "system" ? getSystemTheme() : choice;
  document.documentElement.dataset.theme = resolved;
  document.documentElement.dataset.themeChoice = choice;
  document.documentElement.style.colorScheme = resolved;
}

function readStoredChoice(): ThemeChoice {
  const value = window.localStorage.getItem(storageKey);

  return choices.includes(value as ThemeChoice) ? (value as ThemeChoice) : "system";
}

function ThemeIcon({ choice, resolved }: { choice: ThemeChoice; resolved: ResolvedTheme }) {
  if (choice === "system") {
    return (
      <svg viewBox="0 0 24 24" focusable="false">
        <rect x="5" y="6" width="14" height="10" rx="1.5" />
        <path d="M9 20h6M12 16v4" />
      </svg>
    );
  }

  if (resolved === "dark") {
    return (
      <svg viewBox="0 0 24 24" focusable="false">
        <path d="M19 15.5A7.5 7.5 0 0 1 8.5 5 7.8 7.8 0 1 0 19 15.5Z" />
      </svg>
    );
  }

  return (
    <svg viewBox="0 0 24 24" focusable="false">
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2v3M12 19v3M4.9 4.9 7 7M17 17l2.1 2.1M2 12h3M19 12h3M4.9 19.1 7 17M17 7l2.1-2.1" />
    </svg>
  );
}

export function ThemeToggle({ variant = "label" }: ThemeToggleProps) {
  const [choice, setChoice] = useState<ThemeChoice>("system");
  const [resolved, setResolved] = useState<ResolvedTheme>("light");

  useEffect(() => {
    const initialChoice = readStoredChoice();
    const media = window.matchMedia("(prefers-color-scheme: dark)");

    function sync(nextChoice: ThemeChoice) {
      applyTheme(nextChoice);
      setChoice(nextChoice);
      setResolved(document.documentElement.dataset.theme === "dark" ? "dark" : "light");
    }

    function handleSystemChange() {
      if (readStoredChoice() === "system") {
        sync("system");
      }
    }

    sync(initialChoice);
    media.addEventListener("change", handleSystemChange);

    return () => media.removeEventListener("change", handleSystemChange);
  }, []);

  function cycleTheme() {
    const nextChoice = choices[(choices.indexOf(choice) + 1) % choices.length];
    window.localStorage.setItem(storageKey, nextChoice);
    applyTheme(nextChoice);
    setChoice(nextChoice);
    setResolved(document.documentElement.dataset.theme === "dark" ? "dark" : "light");
  }

  const isIconOnly = variant === "icon";

  return (
    <button
      className={`theme-toggle${isIconOnly ? " theme-toggle--icon" : ""}`}
      type="button"
      onClick={cycleTheme}
      aria-label={`Theme: ${choice}. Switch theme`}
      title={`Theme: ${choice}`}
    >
      <span className="theme-toggle-mark" aria-hidden="true">
        <ThemeIcon choice={choice} resolved={resolved} />
      </span>
      {!isIconOnly && <span>{choice}</span>}
    </button>
  );
}
