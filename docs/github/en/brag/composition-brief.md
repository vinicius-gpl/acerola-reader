# Hyperframes Composition Brief: Acerola

## Objective
Create a short, polished launch-style brag video for Acerola, a local-first comic/manga reader that syncs libraries directly between a user's own Android and Desktop devices over P2P — no cloud, no account.

## Output
- Composition directory: `brag-output-2026-09-28-213229/composition/`
- Rendered video: `brag-output-2026-09-28-213229/brag.mp4`
- Format: landscape — 1920x1080
- Duration: ~23.5 seconds (hard range 15-25s)

## Source Material
- Project root: `C:\Users\vinicius\Desktop\acerola-reader`
- Primary files read: `README.md`, `docs/web/messages/en.json` (landing page copy — hero, facts, how-it-works), `docs/web/src/theme/colors/catppuccin.css` (actual app color theme), the pre-made banner/hero images below.
- Product name: Acerola
- Tagline / strongest claim: "Read your comics and manga on any device. No account, no cloud." (site hero_title) — supporting line: "Your library stays on your devices and syncs directly between them over encrypted P2P — no server stores or sees your data."
- Key UI or visual moment to recreate: none needs recreating — every screen is a real, already-produced screenshot/banner (see Source Assets). Do not rebuild any of these scenes from scratch in HTML/CSS; composite and animate the provided PNGs.
- Copy that must appear verbatim (or near-verbatim, trimmed for reading time — see storyboard):
  - "No cloud" / "No account" (`landing.facts.no_cloud` / `no_account`)
  - "Direct P2P sync" (`landing.features.p2p_sync.title`)
  - "Open source" (`landing.features.open_source.title`)
  - "From install to synced, in three steps" (`landing.how_it_works.title`) — used as the organizing idea for Scene 3+4, not necessarily on screen verbatim
  - Step copy to paraphrase tightly: install → "Install Acerola on the devices you want to sync"; pair → "scan the QR code shown on one with the other"; confirm → "your library and reading progress sync automatically, device to device"

## Creative Direction
- Tone preset: polished
- Creative direction: quiet, premium product film — restrained motion, confident typography, the product speaks for itself. No jokes, no exclamation points, no hype language.
- Interpretation: slow crossfades (0.6-0.8s), generous holds, never more than one short line of new copy on screen at a time. Six narrative beats are required (hook, two-platform establish, two flow steps, the sync payoff, three facts, outro) instead of the usual 3-4 polished scenes — restraint is preserved by grouping the flow steps and the facts into two internally-sequential scenes rather than six separate hard cuts, so the edit still reads as a small number of confident movements.
- Angle: Don't sell an idea — walk the site's own three-step flow (Install → Pair via QR → Confirm/sync) as a real device-to-device demo using only real screenshots, then land the site's own three facts as the payoff. The single hero shot (`acerola-duo-overlap.png`) is the entire proof: one unmodified screenshot of a phone and a desktop window overlapping, showing the exact same synced library, with the app's own real "Scan concluído!" toast still on screen.
- Hook: Acerola mark settles on near-black, then "No cloud. No account." lands beneath it — the claim is the hook, no further setup.
- Outro / punchline: Acerola mark returns; "Acerola. Open source. No cloud." Hold. Silence.
- Avoid:
  - Generic SaaS language ("streamline," "seamless," "unlock")
  - Abstract filler visuals, particles, gradients standing in for content
  - Rebuilding any screen that already exists as a finished asset below
  - Showing `acerola-duo-overlap.png` more than once, or before Scene 4
  - Showing `docs/github/acerola-duo-sidebyside.png` at all (already used elsewhere)
  - Pulling from `docs/github/*/prints/*` for anything a banner already covers (home/reader/history/customization) — those must come from `banner/`

## Visual Identity
- Background: `#11111b` (Catppuccin Mocha "crust") for text-only frames; `#1e1e2e` (Mocha "base") acceptable as a secondary panel tone
- Text: `#cdd6f4` (Mocha foreground)
- Accent: `#cba6f7` (Mocha mauve/lavender — the exact purple already visible on every "CONTINUAR LENDO" / primary button in the screenshots)
- Secondary accent: `#f5c2e7` (Mocha pink — matches the rounded icon chips visible in the app's own screens)
- Success accent: `#a6e3a1` (Mocha green — matches the real "Scan concluído!" toast already baked into the hero screenshot; do not invent a different success color or re-draw that toast)
- Display font: bold, rounded geometric sans matching the weight/feel already baked into the banner headlines (e.g. "Sua biblioteca no desktop") — do not use the doc site's serif heading font, it doesn't match the shipped marketing art
- Body font: DM Sans (the doc site's real body font), for any small supporting/chip label text
- Visual references from the project: the banner images already establish the full visual language (dark theme, rounded window/phone chrome, mauve accent) — match new on-screen text to that language rather than introducing a new visual system

## Source Assets (already produced — composite/animate, do not regenerate)
All copied into `composition/assets/images/`:
- `acerola-desktop-solo.png` — desktop window, logo only, no baked caption. Canvas for Scene 2 and Scene 3's "Install" beat.
- `acerola-android-solo.png` — phone, logo only, no baked caption. Canvas for Scene 2 and Scene 3's "Install" beat.
- `desktop-network-screen.png` (from `docs/github/desktop/prints/network-screen.png`) — real desktop "Rede" screen with the live QR pairing code, already has real window chrome. This is the only correct source for the "Pair via QR" beat; no polished banner exists for this screen, so use this real screenshot directly (crop/frame it to sit consistently alongside the solo banners' window treatment — don't restage the QR code itself).
- `acerola-duo-overlap.png` — the hero/closing shot for Scene 4 only. Phone overlapping desktop window, identical synced library visible on both, real "Scan concluído!" toast still in frame.

Do not touch `docs/github/*/prints/*` beyond `network-screen.png` (no banner exists for that step). Do not use `docs/github/desktop/banner/01-home.png`, `02-reader.png`, `03-history.png`, or the Android `01-home.png`/`02-reader.png`/`03-customization.png`, or `docs/github/acerola-duo-sidebyside.png` — none of these are needed for this storyboard and the numbered ones already carry their own baked Portuguese captions that would visually compete with any new overlay text.

## Storyboard
Use the full storyboard in `brag-output-2026-09-28-213229/brag-plan.md` as the creative contract. Scene summary:

1. **Hook** — 2.5s — Acerola mark settles on `#11111b`; "No cloud. No account." settles beneath it.
2. **Two platforms, one library** — 3s — `acerola-desktop-solo.png` + `acerola-android-solo.png` composed together (both visible, same comics), kicker: "Same library. Every device."
3. **The flow: Install → Pair** — 5.5s (2 sequential sub-beats, ~2.75s each) — "01 — Install" over `acerola-android-solo.png`; "02 — Pair via QR" over `desktop-network-screen.png` with a single scanning-line sweep across the real QR code (the one simulated interaction in the video).
4. **Confirm & synced (hero payoff)** — 5.5s — full-bleed `acerola-duo-overlap.png`, restrained scale-in (0.96→1.0) then hold; line: "From then on, it syncs automatically, device to device." This is both step 3 of the flow and the video's emotional peak — do not add a second success graphic, the real toast in the image already sells it.
5. **Three facts** — 4.5s — three chips arrive one by one, staying on screen together at the end: "No cloud, no account" → "Direct P2P sync" → "Open source" (verbatim from the landing page, in that order).
6. **Outro** — 2.5s — back to the Scene 1 frame; Acerola mark + "Acerola. Open source. No cloud." Hold, no CTA/URL.

## Audio
- Audio role: warm, restrained ambient/electronic bed with sparse, professional, motion-matched accents
- Audio arc: fade in under the hook → steady low presence (~0.30-0.35 volume) through the establish and flow scenes → one deliberate swell exactly as the hero shot (Scene 4) settles → gentle resolve through the facts → fade to silence under the outro
- Music: `assets/music/happy-beats-business-moves-vol-12-by-ende-dot-app.mp3` (already copied into `composition/assets/music/`) — "steady and clean," ~110 BPM, 117s source (video uses only the first ~23.5s)
- Music treatment: fade-in 0-2.5s, steady bed through Scenes 2-3, swell into Scene 4, slow fade-out starting in Scene 6
- Music cue guidance: preset at `assets/music/cues/happy-beats-business-moves-vol-12-by-ende-dot-app.music-cues.json` (+ `.md`), already copied in. Scene 4 (hero) starts at **11.0s** in the planned timeline, and the preset has a strong cue at **10.93s (intensity 0.97)** — lock the hero entrance there (`// beat-locked: 10.93s`). Scene 5 (facts) starts at 16.5s; snap the three chip arrivals to the beat grid near there (16.38 / 16.93 / 17.47 / 18.02 / 18.56 …), skipping beats so arrivals stay ≥~1.0-1.3s apart — do not put a new line of text on every beat. No strong cue sits near the 21.0s outro; use natural fade timing there instead of forcing one.
- Audio-reactive treatment: subtle only — a light glow/presence pulse on the Scene 4 hero shot tied to the music swell is welcome; extract via the current `hyperframes-creative` audio-reactive workflow. No waveform/equalizer visuals, no strobing.
- Audio-coupled moments:
  - Scene 3 — soft UI tap when each step chip ("01 — Install", "02 — Pair via QR") arrives
  - Scene 3 — a subtle scan/sweep sound timed to the scanning-line crossing the real QR code
  - Scene 4 — one clean, quiet confirm chime exactly as the hero shot settles into its hold (in addition to/aligned with the music's beat-locked swell, not a second competing sound)
  - Scene 5 — a soft, low "pop" for each of the three fact chips as they arrive
- SFX selection guidance: polished tone → minimal but present, 2-3 subtle SFX total beyond the per-item taps/pops above; nothing aggressive. `interface/drop_001` or `_002` fits the chip/step arrivals well; a soft `impact/impactSoft_medium_*` or `interface/bong_001` fits the Scene 4 confirm chime. Choose exact files after the real animation timing exists.
- SFX analysis guidance: read `sfx-analysis.md`/`.json` next to the SFX library (see `hyperframes-cli`/skill audio reference) before choosing files; prefer low/medium high-frequency-risk files since this is a polished, repeated-motif video.
- Exact SFX choice: Hyperframes chooses exact filenames, timestamps, density, and volume based on the implemented animation. Copy any chosen SFX into `composition/assets/sfx/...` before referencing them.
- Audio files: music is already copied into `composition/assets/music/` (with its cue preset in `composition/assets/music/cues/`); copy any selected SFX into `composition/assets/sfx/` following the same convention.

## Hyperframes Instructions
Load the composition-building Hyperframes domain skills — `hyperframes-core` (composition contract + `data-*` timing), `hyperframes-animation` (motion), `hyperframes-creative` (design spec, beats, audio-reactive), `hyperframes-keyframes` (seek-safe keyframes), and `hyperframes-cli` (lint/check/render). `/brag` is its own workflow: do not enter the `hyperframes` entry-point intent interview and do not route into its generic promo/launch-video workflow. Prefer native Hyperframes conventions over anything in `/brag`.

Requirements:
- Show at least one real UI, copy, or visual element from the source project (this brief provides four already-produced screenshots — use them; do not rebuild from scratch).
- Keep all text readable in the final render — respect the per-scene reading-time floors called out in `brag-plan.md`.
- Keep the video within 15-25 seconds (target ~23.5s).
- Include the planned music/SFX layer — audio was not disabled and silence was not chosen as the concept.
- Treat `/brag` audio notes as guidance, not a fixed cue sheet. Choose SFX after the visual animation exists.
- Treat music cue metadata as optional timing hints; ignore cues that hurt readability, scene pacing, or the product story.
- Use only 1-3 strong cue locks in this 23.5s video (one is already identified at 10.93s for the hero entrance) unless the edit clearly benefits from more.
- Use SFX to support motion and interaction, with restraint appropriate to `polished`.
- When wiring the music, respect the fade-in/fade-out and swell notes above using the best Hyperframes-supported implementation.
- Use local assets (already copied into `composition/assets/`) for audio and images.
- Run `hyperframes check` before render — it is `/brag`'s single gate.
