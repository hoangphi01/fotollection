# ADR-0053: The roadmap by quarter, two years ahead

- **Status:** Accepted
- **Date:** 2026-11-16
- **Related:** ADR-0006, ADR-0021, ROADMAP.md, STATUS.md

## Context

The roadmap ran in four phases by theme (0 foundation, 1 MVP, 2 smart features, 3 pro and growth) since 2026-10-23. By v0.14.3 (2026-11-16) the phases no longer said where the project stood: Phase 1 was still open (14 P0 requirements not Done, no `REF-WIN` machine ever defined, so no NFR Done), while much of Phase 2 and some of Phase 3 had shipped (postcards, looks, Japanese and Korean). Release targets drifted the same way: work moved from v0.14.1 to v0.14.2, v0.14.3 and v0.14.4, and the docs kept the old numbers (the audit of 2026-11-16 found about 110 stale statements).

The owner set the goal (2026-11-16): Fotollection is maintained as open source for at least two years, to 2028-11, by one person at about 20 hours a week; v1.0.0 in December 2026; every requirement kept, none dropped.

## Options considered

1. **Keep the phases, add a list for v1.0.** Little churn, but the phases still would not say when anything comes.
2. **Redefine the phases** (declare the MVP done, start Phase 2). A label change; the open P0 work and the dates stay unclear.
3. **A roadmap by quarter.** Each requirement in one quarter or *Shipped*; one release theme per MINOR; a maintenance calendar first. Answers "when" and "how long", and shows the load of each quarter.

## Decision

Option 3. ROADMAP.md lists the quarters 2026-Q4 to 2028-Q4 and *Shipped*; STATUS.md's *When* column holds the same value for every FR, checked by `scripts/check-docs.sh`. v1.0.0 keeps the old Phase 1 exit criteria (every P0 FR Done, NFR-001 to NFR-013 within budget on `REF-MAC` and `REF-WIN`), listed in STATUS as *Road to v1.0*. The release once called v0.15.0 "Core 2" becomes v1.1 (2027-Q1). After v1.0.0, one MINOR a month and PATCH releases when needed. PRD priorities (P0–P3) stay: a priority says how much a requirement matters, a quarter says when it comes.

## Consequences

- A requirement added later goes into a quarter in the same commit (DOCS_GUIDE, *A new requirement*).
- A slip moves a release to the next quarter, written in ROADMAP the same day, rather than leaving a past version number in the docs.
- Maintenance (dependencies, advisories, new OS versions, the domain and signing keys) is planned work, ahead of features.
- The roadmap is reviewed every year and at 2028-Q4 for the next two years.
