"use client";

import { useEffect, useState } from "react";

type ThemeChoice = "system" | "light" | "dark";
type ResolvedTheme = "light" | "dark";

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

export function ThemeToggle() {
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

  return (
    <button className="theme-toggle" type="button" onClick={cycleTheme} aria-label={`Theme: ${choice}. Switch theme`}>
      <span className="theme-toggle-mark" aria-hidden="true">
        {resolved === "dark" ? "D" : "L"}
      </span>
      <span>{choice}</span>
    </button>
  );
}
