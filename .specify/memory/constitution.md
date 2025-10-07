<!-- markdownlint-disable-next-line MD041 -->
<!--
Sync Impact Report
Version change: 0.0.0 → 1.0.0
Modified principles:
- (new) Community-Governed Hosts
- (new) Fluid Classes, Distinct Identities
- (new) Discovery-Led Social Knowledge
- (new) Handcrafted Exploration & Emergent Play
- (new) Player Economy & Fair Monetization
Added sections:
- World Charter and Player Experience Boundaries
- Operational Standards and Delivery Gates
Removed sections:
- None
Templates requiring updates:
- ✅ .specify/templates/plan-template.md
- ✅ .specify/templates/spec-template.md
- ✅ .specify/templates/tasks-template.md
- ⚠ .specify/templates/commands/* (directory absent; no action possible)
Follow-up TODOs:
- None
-->

# Haemwend Constitution

## Core Principles

### I. Community-Governed Hosts

- Every host operates behind feature flags that let the host operator tailor systems, pacing, and events to that community without forking the codebase.
- Any gameplay, economy, or live-ops change that affects a host MUST pass an in-game community vote with a documented supermajority threshold before rollout.
- The Game Improvement Plan (GIP) backlog lives in-game; proposals surface with transparent acceptance criteria and post-vote changelog entries.
- Host-level telemetry dashboards MUST show the impact of enabled flags so communities can self-evaluate.

Rationale: Unique host identities thrive when players are accountable for the worlds they shape, and voting keeps the BDFL grounded in player reality.

### II. Fluid Classes, Distinct Identities

- Characters MAY switch jobs/classes at will, but each class MUST retain exclusive mechanics (e.g., wall-climbing, gliding, shielding) and bespoke UI/UX to preserve identity.
- Shared systems (loot, progression, narrative beats) MUST reinforce why class diversity matters—no homogenized stats or recycled signature abilities.
- Balance reviews focus on preserving unique playstyles before raw numbers; parity is measured by “meaningful choice retention,” not DPS charts alone.

Rationale: Removing reroll friction keeps experimentation high, yet the world only feels alive if every class still plays, looks, and solves problems differently.

### III. Discovery-Led Social Knowledge

- All lore, mechanics, and secrets MUST be discoverable in-game; official wikis or out-of-world guides are forbidden.
- Systems MUST reward mentoring, storytelling, and player-to-player knowledge transfer (e.g., shareable journal entries, brag moments, host spotlights).
- Features are evaluated against the “Fun Tests” list—players should constantly find new stories worth sharing, streaming, or creating around.

Rationale: Curiosity and community storytelling power the brand; frictionless discovery channels would shatter that magic.

### IV. Handcrafted Exploration & Emergent Play

- The world is handcrafted: every zone and landmark requires artist-led intent, bespoke narrative hooks, and traversal variety.
- Quests MUST present branching paths with meaningful consequences; weekly cycles and live events are tuned to encourage surprise and collaboration.
- Systems that encourage emergent gameplay (economy exploits, traversal tricks, social puzzles) are embraced as long as they do not collapse narrative coherence.

Rationale: Exploration feels rewarding when the map is authored with love yet still allows players to invent new stories inside it.

### V. Player Economy & Fair Monetization

- The economy is player-driven: crafting, trade loops, and supply/demand signals originate from players, not NPC vendors.
- Monetization is limited to cosmetics, expansions, and quality-of-life boosts that do not confer competitive power.
- Pay-to-win mechanics, untransparent loot boxes, and predatory retention timers are banned outright.
- Regional pricing MUST honor local purchasing power; parental controls and spend alerts must be available, and refunds are honored for at least 14 days or per stricter local law.

Rationale: Trust is currency—financial systems that respect players keep the community vibrant.

## World Charter and Player Experience Boundaries

### Purpose and Success Criteria

- **Mission:** Deliver a 3D MMORPG that honors classic WoW’s spirit while introducing distinctive Haemwend twists.
- **Target players:** PC players identifying as men aged 18–38 who seek social, persistent worlds.
- **Success metrics:** Reach 100 active players within six months of launch and sustain an average of at least one hour of engagement per player per weekday for six consecutive months (~125 hours each).

### Non-Goals

- No PVP systems, blockchain or NFT integrations, procedurally generated content, single-player experiences, mobile, VR/AR, or console clients.

### Design Pillars and Trade-offs

- Host-specific feature sets managed through feature flags.
- Community voting drives feature adoption.
- Class switching without rerolling, balanced against unique class identity.
- Lore and knowledge distributed socially inside the game world.
- Seven-day character activity cadence to keep worlds feeling inhabited.
- Handcrafted zones, quests with multiple outcomes, and rewarded exploration.
- Player-driven economy and crafting systems.

Trade-offs:

1. **Class Flexibility** ↔ **Character Identity**
2. **Emergent Gameplay** ↔ **Narrative Depth**
3. **Player-Driven Economy** ↔ **Balance and Fairness**

Fun validation checklist:

- Can a player explain the game in five minutes?
- Can they show off something cool they recently accomplished?
- Do they want to return tomorrow and invite friends, stream, create content, art, fiction, mods, guides, lore, music, cosplay, or memes about it?

### Canon and IP

- **Source of truth:** `./docs/lore/`
- **Immutable canon:** Rooted in Germanic mythology (distinct from Norse mythology).
- **Experimental spaces:** Classes, visual style, economy tuning, and UI can iterate within lore boundaries.

### Scope Boundaries

- **Feature floor (MVP):** Single server, three classes, three zones, 30 quests per zone, foundational combat, unique abilities per class.
- **Ceiling (ambitious):** Multiple servers with host-specific flags, expanded class roster, additional zones and quests, richer combat depth, extended ability kits.

### Economy and Monetization Guardrails

- Allowed: cosmetics, expansions, non-power quality-of-life unlocks.
- Banned: power-selling, undisclosed odds loot boxes, psychologically abusive timers.
- Pricing: Apply regional pricing policies informed by purchasing power parity.
- Parental and refunds: Provide parental dashboards, spending caps, and clear refund workflows honoring at least a 14-day window or tighter legal requirements.

### Modding and User-Generated Content

- Modding is not supported; the team invests directly in making the core game better.
- Community content creation (streams, videos, art, fiction) is encouraged provided it respects in-game lore ownership.

### Decision Rights

- The BDFL retains final creative authority for cross-host direction.
- Host communities may veto BDFL decisions with a recorded supermajority vote to prevent runaway changes.
- The in-game GIP enables players to submit, discuss, and vote on improvements with transparent status tracking.

## Operational Standards and Delivery Gates

### Quality Bars

- **Performance:** 120 FPS on low-end CPUs (4 GB RAM, SSD, GTX 1050 Ti), 3 seconds to main menu, 5 seconds to enter the world.
- **Accessibility:** Remappable controls, simultaneous input tolerance, single-hand options, font replacement, customizable captions, toggle/hold parity, replayable or skippable tutorials, TTS chat support, pausable/replayable cutscenes, full color-blind modes.
- **Localization:** English and Futhark text support.
- **UI/UX:** Class-specific interfaces that are context sensitive, resizable, movable, and scalable.
- **Audio:** 3D positional audio, per-channel volume controls, subtitles.
- **Art:** Stylized akin to WoW Vanilla; polycount between Vanilla and WotLK; readability at distance is mandatory.
- **UX:** Intuitive, consistent flows with minimal clicks and unobtrusive tutorials.
- **Testing:** Unit, integration, end-to-end, manual QA, and automated regression suites.
- **Security:** OWASP Top 10 coverage, recurring penetration tests, and a live bug bounty program.
- **Scalability:** Support 10,000 concurrent players on a single server instance.

### Fairness and Safety

- Matchmaking philosophy: No SBMM/ELO because there is no PVP focus.
- Anti-cheat: Combine client-side detection with server validation and escalating penalties.
- Community standards: Tie to the Social Contract with a published penalty matrix and an appeal SLA.

### Data and Experiments

- Telemetry: Capture necessary events with documented lawful basis and retention policies.
- Experimentation: Guardrails on simultaneous tests, maximum player impact bands, and sunset criteria for unfinished experiments.

### Change Control

- RFCs are required for modifications to Sections 1–11 of this constitution.
- Quorum requires named approvers with a cooling period under 48 hours before activation.
- Versioning follows SemVer (`MAJOR.MINOR.PATCH`) with a published changelog for every amendment.

### Shipping Cadence

- Target monthly releases; interim patches may ship as needed when they do not violate gating standards.

### Postmortems

- Conduct blameless reviews with 5 Whys analysis, accountable owners, and dated follow-up actions.

## Governance

The BDFL acts as final arbiter for cross-host direction while honoring Principle I through binding community votes on host-level changes. RFC-backed amendments are mandatory for altering constitutional sections, and each amendment must document anticipated player impact, telemetry coverage, and mitigation plans. Compliance reviews accompany every release plan: producers verify adherence to Core Principles, QA signs off on Quality Bars, and community management validates fairness, safety, and monetization guardrails. The constitution supersedes conflicting docs; exceptions require explicit traceability in release notes and a time-bound plan to return to compliance.

**Version**: 1.0.0 | **Ratified**: 2025-10-08 | **Last Amended**: 2025-10-08
