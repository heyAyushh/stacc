"use client";

import { createContext, use, useCallback, useEffect, useMemo, useState, useTransition, type ReactNode } from "react";
import { useRouter } from "next/navigation";
import { Command } from "cmdk";
import type { SearchItem } from "@/lib/search";

type CommandSearchProviderProps = {
  items: SearchItem[];
  children: ReactNode;
};

type CommandSearchContextValue = {
  openPalette: () => void;
};

type CommandSearchProps = {
  variant?: "docs" | "compact";
};

const keyboardShortcut = "K";

const searchIcon = (
  <span className="search-icon" aria-hidden="true">
    <svg viewBox="0 0 24 24" focusable="false">
      <circle cx="11" cy="11" r="7" />
      <path d="m16.5 16.5 4 4" />
    </svg>
  </span>
);

const CommandSearchContext = createContext<CommandSearchContextValue | null>(null);

function groupItems(items: SearchItem[], kind: SearchItem["kind"]): SearchItem[] {
  return items.filter((item) => item.kind === kind);
}

function normalizeSearchText(value: string): string {
  return value.toLowerCase();
}

function rankSearchResult(value: string, search: string, keywords?: string[]): number {
  const terms = normalizeSearchText(search)
    .split(/\s+/)
    .flatMap((term) => {
      const trimmedTerm = term.trim();

      return trimmedTerm ? [trimmedTerm] : [];
    });

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

function useCommandSearch() {
  const context = use(CommandSearchContext);

  if (!context) {
    throw new Error("CommandSearch must be rendered inside CommandSearchProvider");
  }

  return context;
}

export function CommandSearchProvider({ items, children }: CommandSearchProviderProps) {
  const router = useRouter();
  const [, startTransition] = useTransition();
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const groups = useMemo(
    () => [
      { label: "Documentation", items: [...groupItems(items, "doc"), ...groupItems(items, "section")] },
      { label: "Skills", items: groupItems(items, "skill") },
      { label: "MCP Servers", items: groupItems(items, "mcp") },
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

  const openPalette = useCallback(function openPalette() {
    setOpen(true);
  }, []);

  const contextValue = useMemo(() => ({ openPalette }), [openPalette]);

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
    <CommandSearchContext.Provider value={contextValue}>
      {children}
      <Command.Dialog
        open={open}
        onOpenChange={setOpen}
        label="Search STACC documentation, skills, and MCP servers"
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
              placeholder="TYPE A DOC, SECTION, SKILL, OR MCP…"
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
                    <span className="command-copy">
                      <strong>{item.title}</strong>
                      <small>
                        <span>{item.eyebrow}</span>
                        <span>{item.detail}</span>
                      </small>
                      <span>{item.description}</span>
                    </span>
                  </Command.Item>
                ))}
              </Command.Group>
            ))}
          </Command.List>
        </div>
      </Command.Dialog>
    </CommandSearchContext.Provider>
  );
}

export function CommandSearch({ variant = "docs" }: CommandSearchProps) {
  const { openPalette } = useCommandSearch();
  const isCompact = variant === "compact";

  return (
    <button
      className={isCompact ? "command-search-compact" : "search-field"}
      type="button"
      onClick={openPalette}
      aria-label="Search documentation, skills, and MCP servers with Command or Control K"
    >
      {searchIcon}
      {isCompact ? (
        <>
          <span>SEARCH</span>
          <kbd>CMD K</kbd>
        </>
      ) : (
        <>
          <span className="search-label">SEARCH_</span>
          <span className="search-placeholder">FIND DOC, SECTION, SKILL, OR MCP…</span>
          <span className="search-key">CMD K</span>
        </>
      )}
    </button>
  );
}
