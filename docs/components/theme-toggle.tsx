"use client";

import { useSyncExternalStore } from "react";

type ThemeChoice = "system" | "light" | "dark";
type ResolvedTheme = "light" | "dark";
type ThemeToggleProps = {
  variant?: "label" | "icon";
};

const storageKey = "stacc-theme";
const themeChangeEvent = "stacc-theme-change";
const choices: ThemeChoice[] = ["system", "light", "dark"];
const themeColors: Record<ResolvedTheme, { background: string; text: string }> = {
  dark: {
    background: "#080808",
    text: "#f7f7f7",
  },
  light: {
    background: "#ffffff",
    text: "#000000",
  },
};

type ThemeSnapshot = {
  choice: ThemeChoice;
  resolved: ResolvedTheme;
};

function getSystemTheme(): ResolvedTheme {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applyTheme(choice: ThemeChoice) {
  const resolved = choice === "system" ? getSystemTheme() : choice;
  document.documentElement.dataset.theme = resolved;
  document.documentElement.dataset.themeChoice = choice;
  document.documentElement.style.colorScheme = resolved;
  document.documentElement.style.backgroundColor = themeColors[resolved].background;
  document.documentElement.style.color = themeColors[resolved].text;
}

function readStoredChoice(): ThemeChoice {
  const value = window.localStorage.getItem(storageKey);

  return choices.includes(value as ThemeChoice) ? (value as ThemeChoice) : "system";
}

function readThemeSnapshot(): ThemeSnapshot {
  const choice = readStoredChoice();
  const resolved = choice === "system" ? getSystemTheme() : choice;

  return { choice, resolved };
}

function getThemeSnapshot(): string {
  const { choice, resolved } = readThemeSnapshot();

  return `${choice}:${resolved}`;
}

function getServerThemeSnapshot(): string {
  return "system:light";
}

function parseThemeSnapshot(snapshot: string): ThemeSnapshot {
  const [choice, resolved] = snapshot.split(":");

  return {
    choice: choices.includes(choice as ThemeChoice) ? (choice as ThemeChoice) : "system",
    resolved: resolved === "dark" ? "dark" : "light",
  };
}

function subscribeToThemeChanges(onStoreChange: () => void): () => void {
  const media = window.matchMedia("(prefers-color-scheme: dark)");

  function handleSystemChange() {
    if (readStoredChoice() === "system") {
      applyTheme("system");
      onStoreChange();
    }
  }

  media.addEventListener("change", handleSystemChange);
  window.addEventListener(themeChangeEvent, onStoreChange);

  return () => {
    media.removeEventListener("change", handleSystemChange);
    window.removeEventListener(themeChangeEvent, onStoreChange);
  };
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
  const { choice, resolved } = parseThemeSnapshot(
    useSyncExternalStore(subscribeToThemeChanges, getThemeSnapshot, getServerThemeSnapshot)
  );

  function cycleTheme() {
    const nextChoice = choices[(choices.indexOf(choice) + 1) % choices.length];
    window.localStorage.setItem(storageKey, nextChoice);
    applyTheme(nextChoice);
    window.dispatchEvent(new Event(themeChangeEvent));
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
