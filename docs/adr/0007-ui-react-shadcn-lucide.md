# ADR-0007: UI stack — React + shadcn/ui + Lucide

- **Status:** Accepted
- **Date:** 2026-10-23
- **Related:** ADR-0001, NFR-001, NFR-005, [STYLE_RULE.md §3–4](../STYLE_RULE.md#3-typescript-ui)

## Context

The UI must look polished and professional, and stay consistent as features grow. The UI covers settings panels, dialogs, sliders, a crop editor and a large thumbnail grid. The 2026-09 plan chose vanilla TypeScript for simplicity. The project owner wants **shadcn/ui** components and **Lucide** icons. shadcn/ui is built on React and Radix UI primitives, styled with Tailwind CSS.

## Options considered

1. **Vanilla TS:** smallest runtime, but every accessible component (dialog, select, slider, tooltip) has to be built by hand.
2. **Svelte + shadcn-svelte:** smaller runtime, but a community port with a smaller ecosystem.
3. **React + shadcn/ui + Lucide:** the canonical shadcn implementation. Accessible Radix primitives, copy-in components that we own, and the largest ecosystem.

## Decision

Use **Vite + React + TypeScript + Tailwind CSS + shadcn/ui**, with **`lucide-react`** for icons.

## Consequences

- React adds about 45 KB gzipped to the UI. This is negligible for a desktop app, and all heavy work stays in Rust ([ARCHITECTURE.md §1](../ARCHITECTURE.md#1-overview)).
- Performance rules: the thumbnail grid must be virtualized (`@tanstack/react-virtual`), and image bytes never go into React state ([STYLE_RULE.md §3](../STYLE_RULE.md#3-typescript-ui)).
- XSS rules are adapted for React: `dangerouslySetInnerHTML` is banned and checked by CI ([SECURITY.md §7](../SECURITY.md#7-ci-security-gates)).
- shadcn components are copied into `ui/src/components/ui/`. We own and maintain them, and they have no runtime dependency on a component package.
- This supersedes the 2026-09 "vanilla TS, no React" decision.
