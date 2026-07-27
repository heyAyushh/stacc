# STACC Design System

## 1. Atmosphere & Identity

STACC feels like a technical poster that became a control surface: stark, monochrome, dense, and exact. The signature is industrial documentation with compressed display type, mono operational copy, hard borders, and responsive layouts that preserve legibility before ornament.

## 2. Color

### Palette

| Role | Token | Light | Dark | Usage |
| --- | --- | --- | --- | --- |
| Surface / page | `--paper` | `#fff` | `#080808` | Main page background |
| Surface / panel | `--panel` | `#fff` | `#0f0f0f` | Cards, ledgers, docs panels |
| Text / primary | `--ink` | `#000` | `#f7f7f7` | Headlines, controls, strong text |
| Text / body | `--body-copy` | `#303030` | `#d4d4d4` | Long-form documentation copy |
| Text / muted | `--muted` | `#6a6a6a` | `#b4b4b4` | Metadata and secondary labels |
| Border / primary | `--line` | `#111` | `#f2f2f2` | Structural borders |
| Border / muted | `--muted-stroke` | `#a5a5a5` | `#6f6f6f` | Pills, minor controls |
| Border / soft | `--soft-line` | `#c9c9c9` | `#343434` | Row dividers |
| Surface / soft | `--soft` | `#f4f4f4` | `#161616` | Inline code and subtle fills |
| Inverse / surface | `--inverse` | `#000` | `#f7f7f7` | Inverted panels and hover states |
| Inverse / text | `--inverse-text` | `#fff` | `#050505` | Text on inverse surfaces |

### Rules

- Keep the interface monochrome unless syntax highlighting or status semantics require color.
- Use borders and tonal inversion for emphasis; do not add decorative gradients.
- Any new color must become a token before use.

## 3. Typography

### Scale

| Level | Size | Weight | Line Height | Tracking | Usage |
| --- | --- | --- | --- | --- | --- |
| Poster | `clamp(48px, 11vw, 150px)` | 400 | 0.76-0.9 | 0 | Landing hero and oversized numerals |
| Display | `clamp(48px, 12vw, 112px)` | 400 | 0.9 | 0 | Docs page titles |
| Section | `clamp(32px, 5vw, 52px)` | 400 | 1 | 0 | Docs section titles |
| Card title | `18px-24px` | 700 | 1.08-1.15 | 0 | Card and matrix labels |
| Body | `16px-18px` | 500-700 | 1.5-1.8 | 0 | Documentation copy |
| Small mono | `12px-14px` | 500-700 | 1.35-1.55 | 0 | Ledgers, cards, metadata |
| Overline | `9px-11px` | 700 | 1 | 0.11em | Small uppercase labels |

### Font Stack

- Display: `var(--font-anton), Impact, Haettenschweiler, "Arial Narrow", sans-serif`
- Mono: `var(--font-ibm-plex-mono), ui-monospace, "SFMono-Regular", Menlo, Consolas, monospace`
- UI sans: `Inter, system-ui, sans-serif`

### Rules

- Never let headings break by individual letters or arbitrary fragments.
- Use mono copy for operational detail; use compressed display only for titles and numerals.
- Body text must stay at or above 14px.

## 4. Spacing & Layout

### Base Unit

All spacing follows a 4px base.

| Token | Value | Usage |
| --- | --- | --- |
| Tight | 8px | Inline groups, small gaps |
| Compact | 12px | Mobile card padding |
| Standard | 16px | Controls and rows |
| Comfortable | 24px | Card groups |
| Section | 48px | Docs section rhythm |
| Page | 64px | Page gutters and major separations |

### Grid

- Max board width: `1280px`.
- Docs shell max width: `1400px`.
- Breakpoints: mobile below `560px`, tablet below `900px`, desktop above `900px`.
- Grids must declare useful minimum track widths; `minmax(0, 1fr)` is not acceptable for content cards.

## 5. Components

### TUI Segment Card

- **Structure**: section number, uppercase title, mono body copy.
- **Spacing**: 12px-18px internal gaps, 14px-18px padding.
- **Responsive rule**: use multi-column only when each card can keep whole words readable; otherwise switch to row cards.
- **Accessibility**: semantic article/card content, no decorative-only text.

### Support Callout

- **Structure**: overline, strong summary, compact metadata.
- **Spacing**: 14px padding, 12px-16px grid gap.
- **Responsive rule**: two columns on wide screens, one column on mobile/tablet.

## 6. Motion & Interaction

| Type | Duration | Easing | Usage |
| --- | --- | --- | --- |
| Press | `140ms` | `var(--ease-out)` | Button active state |
| Hover | `180ms` | `var(--ease-out)` | Links, pills, cards |
| Emphasis | `720ms` | `var(--ease-out)` | Hero/wave motion |

Only animate transform, opacity, color, background, and border-color. Respect `prefers-reduced-motion`.

## 7. Depth & Surface

Depth strategy is borders-only. Structural hierarchy comes from 1px rules, inverted surfaces, and tonal fills. Do not add shadows to docs surfaces unless a modal or overlay requires it.
