# ADR-0013: Allow inline styles in the CSP, keep scripts strict

- **Status:** Accepted
- **Date:** 2026-10-23
- **Related:** ADR-0007, [SECURITY.md §2](../SECURITY.md#2-tauri-hardening)

## Context

The UI uses shadcn/ui on Radix primitives ([ADR-0007](0007-ui-react-shadcn-lucide.md)). Radix **Select**, **Dialog** and **Popover** lock page scrolling through `react-remove-scroll`. That library injects `<style>` elements at runtime, and their content (for example the scrollbar gap) is computed. Our previous CSP, `style-src 'self'` plus hashes for two static Radix styles, would block these tags in release builds. Hashes cannot cover computed content, and Tauri's CSP nonces are not exposed to runtime code.

## Options considered

1. **Keep a strict `style-src` and avoid Radix overlays:** we would hand-build dialogs and selects and lose accessible, maintained components. This contradicts ADR-0007.
2. **Nonces for runtime styles:** not available to the app (Tauri generates nonces only for tags in `index.html`).
3. **`style-src 'self' 'unsafe-inline'`, with every other directive strict:** inline styles work, and scripts stay locked down.

## Decision

Use option 3. The CSP is:

- `script-src 'self'`, with no inline scripts and no eval.
- `style-src 'self' 'unsafe-inline'`.
- `img-src`, `connect-src` and `font-src` restricted to the app, the IPC channel and the `fotothumb` protocol.

## Consequences

- **Risk:** CSS injection could alter how the page looks. It cannot run code. It also cannot exfiltrate data: there are no remote origins in `img-src`, `font-src` or `connect-src`. The app never renders untrusted HTML ([SECURITY.md §3](../SECURITY.md#3-input-and-metadata-defense)).
- The two static style hashes and `scripts/csp-style-hashes.sh` are removed.
- Libraries that inject styles (`sonner`, Radix overlays) are acceptable again. Script-injecting libraries remain forbidden.
- Revisit this if Tauri exposes per-page style nonces to the frontend.
