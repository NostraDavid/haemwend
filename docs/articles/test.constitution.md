---
changelog:
  2025-10-08: Initial version.
about: this is the original constitution for the game.
---

# Game Constitution

## 1. Purpose

- **Mission:** To create a 3D MMORPG that captures the essence of classic WoW while introducing unique gameplay elements.
- **Target player:** Men, 18-38, PC.
- **Success criteria:**
  - 100 active players within 6 months of launch.
  - When we're able to keep each player entertained for 1 hour per day, 5 days a week, continuously for 6 months (~125 hours)

## 2. Non-Goals

- PVP
- blockchain or NFT
- procedurally generated content
- single player
- mobile
- VR/AR
- console

## 3. Design Pillars

- Each host gets their own features (via feature flags)
- The community of each host gets to vote on features.
- Each character can switch between jobs/classes. No more rerolling or altoholics.
- Each class must be unique and have a unique playstyle. Some can climb walls, others can glide, others can hold a shield above their head to block projectiles (or let people climb on top of them).
- Lore should be discovereable in-game, not in a wiki.
- Game information should be shared by people, not wikis or guides.
- 7 day week cycle of characters that do things.
- Handcrafted world with unique zones and landmarks.
- Quests with branching paths and multiple solutions.
- Exploration and discovery are rewarded.
- Massively multiplayer
- Player-driven economy and crafting systems
- Generative testing, which includes property-based testing and fuzz testing, but also AI-driven testing.

1. **Class Flexibility** — tradeoff vs **Character Identity**.
2. **Emergent Gameplay** — tradeoff vs **Narrative Depth**.
3. **Player-Driven Economy** — tradeoff vs **Balance and Fairness**.

- **Fun tests:**
  - Can I explain the game to a friend in 5 minutes?
  - Can I show off something cool I did in the game?
  - Do I want to come back tomorrow?
  - Do I want to tell my friends about this game?
  - Do I want to stream this game?
  - Do I want to make videos about this game?
  - Do I want to write fan fiction about this game?
  - Do I want to create art about this game?
  - Do I want to create mods for this game?
  - Do I want to create guides for this game?
  - Do I want to create lore for this game?
  - Do I want to create music for this game?
  - Do I want to create cosplay for this game?
  - Do I want to create memes for this game?

## 4. Canon and IP

- **Sources of truth:** ./docs/lore/
- **Immutable:** Based on Germanic mythology (not to be confused with Norse mythology).
- **Experimental:** Classes, visual style, economy, UI.

## 5. Scope Boundaries

- **Feature floor (MVP):** Single server, 3 classes, 3 zones, 30 quests per zone, basic combat, basic (yet unique) abilities per class.
- **Ceiling (nice-to-have):** More servers (each with feature flags), more classes, more zones, more quests, fuller combat, more abilities (?).

## 6. Quality Bars

- **Performance:** 120 FPS on low-end CPU, 4GB RAM, SSD, GTX 1050 Ti; in 3 seconds to the main menu; in 5s in the game.
- **Accessibility:** Remappable keys, simultaneous-input tolerance, single-hand play-options, font replacement, captions
  (customizable), toggle vs hold options, replayable/skippable tutorials, TTS for chat, pausable/replayable cutscenes,
  full color-blind modes.
- **Localization:** English, Futhark text.
- **UI:** Unique per class, context sensitive, resizable, movable, scalable.
- **Audio:** 3D positional audio, customizable volume per channel, subtitles.
- **Art:** Stylized a la WoW Vanilla, poly-count between Vanilla and WotLK, consistent, readable at a distance.
- **UX:** Intuitive (familiar), consistent, minimal clicks, out-of-the-way tutorials.
- **Testing:** Unit tests, integration tests, E2E tests, manual QA, automated regression tests.
- **Security:** OWASP top 10, regular pentests, bug bounty program.
- **Scalability:** Support 10000 concurrent players on a single server.

## 7. Economy and Monetization

- **Allow:** cosmetics, expansions, QoL non-power boosts.
- **Ban:** pay-to-win, loot boxes without odds, predatory timers.
- **Pricing guardrails:** regional pricing policy.
- **Parental and refund policy:** [DEFINE LATER].

## 8. Fairness and Safety

- **Matchmaking philosophy:** No SBMM/ELO rules, as there's no PVP.
- **Anti-cheat:** Client-side detection, server-side validation, penalties for cheating.
- **Community standards:** link to Social Contract; penalty matrix; appeal SLA.

## 9. Data and Experiments

- **Telemetry:** events, lawful basis, retention policy.
- **A/B testing:** guardrails, max concurrent tests, player impact limits.

## 10. Modding and UGC

- **Allowed:** Nothing, I think? We should be pushed to make the game better, not let people change it.
- **Prohibited:** Everything, I think? See above.
- **License:** Allow people to make content from the game? Though I want to keep lore in-game.

## 11. Decision Rights

- **RACI:** Me. The BDFL.
- **Vetoes:** Let supermajority of community veto, in case of me making a mistake.
- **Game Improvement Plan (GIP):** Users should be able to suggest features/improvements, and vote on them (in-game!).

## 12. Change Control

- **RFC required** for changes to §§ 1–11.
- **Quorum:** n approvers; cooling period <48h>.
- **Versioning:** SemVer-style `MAJOR.MINOR.PATCH`; changelog required.

## 13. Shipping Gates

- Monthly releases, unless it's a patch.

## 14. Postmortems

- **Process:** blameless, 5 whys, owners, deadlines.
