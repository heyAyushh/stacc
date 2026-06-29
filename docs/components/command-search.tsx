"use client";

import { useEffect, useMemo, useState, useTransition } from "react";
import { useRouter } from "next/navigation";
import { Command } from "cmdk";
import type { SearchItem } from "@/lib/search";

type CommandSearchProps = {
  items: SearchItem[];
};

const keyboardShortcut = "K";
const kindLabels: Record<SearchItem["kind"], string> = {
  doc: "Doc",
  section: "Section",
  skill: "Skill",
};

const searchIcon = (
  <span className="search-icon" aria-hidden="true">
    <svg viewBox="0 0 24 24" focusable="false">
      <circle cx="11" cy="11" r="7" />
      <path d="m16.5 16.5 4 4" />
    </svg>
  </span>
);

function groupItems(items: SearchItem[], kind: SearchItem["kind"]): SearchItem[] {
  return items.filter((item) => item.kind === kind);
}

function normalizeSearchText(value: string): string {
  return value.toLowerCase();
}

function rankSearchResult(value: string, search: string, keywords?: string[]): number {
  const terms = normalizeSearchText(search)
    .split(/\s+/)
    .map((term) => term.trim())
    .filter(Boolean);

  if (terms.length === 0) {
    return 1;
  }

  const normalizedValue = normalizeSearchText(value);
  const normalizedKeywords = (keywords ?? []).map(normalizeSearchText);
  const haystack = [normalizedValue, ...normalizedKeywords].join(" ");

  if (!terms.every((term) => haystack.includes(term))) {
    return 0;
  }

  if (terms.some((term) => normalizedValue.startsWith(term))) {
    return 4;
  }

  if (terms.some((term) => normalizedKeywords.some((keyword) => keyword.startsWith(term)))) {
    return 3;
  }

  return 2;
}

export function CommandSearch({ items }: CommandSearchProps) {
  const router = useRouter();
  const [, startTransition] = useTransition();
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const groups = useMemo(
    () => [
      { label: "Documentation", items: [...groupItems(items, "doc"), ...groupItems(items, "section")] },
      { label: "Skills", items: groupItems(items, "skill") },
    ],
    [items]
  );

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key.toLowerCase() === keyboardShortcut.toLowerCase() && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        setOpen((current) => !current);
      }
    }

    document.addEventListener("keydown", handleKeyDown);

    return () => document.removeEventListener("keydown", handleKeyDown);
  }, []);

  function openPalette() {
    setOpen(true);
  }

  function selectItem(href: string) {
    setOpen(false);
    setSearch("");
    const targetUrl = new URL(href, window.location.origin);

    if (targetUrl.pathname === window.location.pathname && targetUrl.hash.length > 0) {
      window.history.pushState(null, "", `${targetUrl.pathname}${targetUrl.hash}`);
      window.requestAnimationFrame(() => {
        document.getElementById(targetUrl.hash.slice(1))?.scrollIntoView({ block: "start" });
      });
      return;
    }

    startTransition(() => {
      router.push(href);
    });
  }

  return (
    <>
      <button className="search-field" type="button" onClick={openPalette} aria-label="Search documentation and skills">
        {searchIcon}
        <span className="search-label">SEARCH_</span>
        <span className="search-placeholder">FIND DOC, SECTION, OR SKILL...</span>
        <span className="search-key">CMD K</span>
      </button>

      <Command.Dialog
        open={open}
        onOpenChange={setOpen}
        label="Search STACC documentation and skills"
        loop
        shouldFilter
        filter={rankSearchResult}
        className="command-dialog"
      >
        <div className="command-frame">
          <div className="command-input-row">
            <span>SEARCH_</span>
            <Command.Input
              value={search}
              onValueChange={setSearch}
              placeholder="TYPE A DOC, SECTION, OR SKILL"
              autoFocus
            />
            <kbd>ESC</kbd>
          </div>

          <Command.List className="command-list">
            <Command.Empty className="command-empty">NO MATCHES FOUND</Command.Empty>
            {groups.map((group) => (
              <Command.Group className="command-group" heading={group.label} key={group.label}>
                {group.items.map((item) => (
                  <Command.Item
                    className="command-item"
                    key={item.id}
                    value={item.title}
                    keywords={item.keywords}
                    onSelect={() => selectItem(item.href)}
                  >
                    <span className="command-kind">{kindLabels[item.kind]}</span>
                    <span className="command-copy">
                      <strong>{item.title}</strong>
                      <small>{item.eyebrow}</small>
                      <span>{item.description}</span>
                    </span>
                    <span className="command-arrow">ENTER</span>
                  </Command.Item>
                ))}
              </Command.Group>
            ))}
          </Command.List>
        </div>
      </Command.Dialog>
    </>
  );
}
