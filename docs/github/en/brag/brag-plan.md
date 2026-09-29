# Brag Plan: Acerola

## What is this app?
Acerola is a local-first comic/manga reader for Android and Desktop whose libraries sync straight between your own devices over encrypted P2P — no cloud, no account, no central server.

## The angle
Don't sell an idea — walk the site's own three-step flow (Install → Pair via QR → Confirm/sync) as a real device-to-device demo, then land the three facts (No cloud/no account, Direct P2P sync, Open source) as the payoff. The proof is a single unmodified screenshot: a phone and a desktop window overlapping, showing the *exact same* synced library, with the app's own "Scan concluído!" toast still on screen. Restraint is the joke there is no joke, just two platforms actually staying in sync.

## Hook (first 2-3 seconds)
Acerola's logo mark settles on a near-black frame, then two short facts land beneath it, verbatim from the landing page: "No cloud. No account." No further setup — the claim is the hook.

## Key moments (the middle)
- Both platforms established up front (desktop + Android solo device shots), before any flow talk — this is a two-platform product, not a phone app with a desktop afterthought.
- The real three-step flow, using real screens: Install → open Network and scan the QR code (real pairing screen, real QR) → confirmed.
- The `acerola-duo-overlap.png` hero shot as the "confirmed/synced" payoff: phone in front, desktop behind, identical library on both, the app's own green "Scan concluído!" toast still visible. This single frame is both step 3's proof and the video's emotional peak.
- The three landing-page facts land as a clean trio, in the site's own order: No cloud/no account → Direct P2P sync → Open source.

## Outro / punchline
Logo returns, paired with a short closing line that echoes the hook: "Acerola. Open source. No cloud." Hold. Silence.

## User flow worth showing
1. **Entry:** install Acerola on the devices you want to sync (Android APK, desktop build).
2. **Key action:** open Network on both devices, scan the QR code shown on one with the other.
3. **Result:** confirm the pairing — from then on the library and reading progress sync automatically, device to device (proven on screen, not claimed in text).

## Tone
- Preset: polished
- Creative direction: quiet, premium product film — restrained motion, confident typography, the product speaks for itself
- Interpretation: slow crossfades (0.6-0.8s), generous holds, minimal on-screen copy (never more than one short line at a time), no jokes, no exclamation points. The brief needs six distinct beats (hook, two-platform establish, two flow steps, the sync payoff, three facts, outro) instead of the usual 3-4 polished scenes — restraint is kept by grouping the flow steps and the facts into two sequential-reveal scenes rather than six separate hard cuts, so the film still reads as a small number of confident movements.

## Format: landscape — 1920x1080
## Duration: ~23.5s target

## Visual identity (from the project)
- Background: `#11111b` (crust) / `#1e1e2e` (base) — Catppuccin Mocha, the app's actual dark theme
- Accent: `#cba6f7` (mauve/lavender — matches the "CONTINUAR LENDO" / primary buttons in every screenshot)
- Secondary accent: `#f5c2e7` (pink — matches the rounded icon chips in the app's own screens)
- Success accent: `#a6e3a1` (green — matches the real "Scan concluído!" toast in the hero shot; do not invent a different success color)
- Text: `#cdd6f4` (foreground)
- Display font: bold, rounded geometric sans — match the weight/feel already baked into the banner headlines (e.g. "Sua biblioteca no desktop"), not the doc site's serif heading font
- Body font: DM Sans (the doc site's actual body font) for any smaller supporting line/chip label
- Strongest visual element: `docs/github/acerola-duo-overlap.png` — phone overlapping desktop, identical synced library, real "Scan concluído!" toast still on screen. This is the hero shot and must not be used anywhere earlier in the video.

## Source assets — use as-is, do not rebuild
- `docs/github/desktop/banner/acerola-desktop-solo.png` — desktop window, logo only, no baked caption. Use as canvas for our own step/fact captions.
- `docs/github/android/banner/acerola-android-solo.png` — phone, logo only, no baked caption. Same use as above.
- `docs/github/desktop/prints/network-screen.png` — raw (but already has real window chrome) desktop "Rede" screen with the live QR pairing code. No banner exists for this step, so this is the correct source for "Pair via QR" — frame/crop it consistently with the banners' window treatment, do not restage the QR itself.
- `docs/github/acerola-duo-overlap.png` — the hero/closing shot. Reserved for the sync payoff only.
- Do NOT use `docs/github/*/prints/*` for anything a banner already covers (home, reader, history/customization) — those already exist as polished, captioned banners (`01-home`, `02-reader`, `03-history`/`03-customization`) and should be pulled from `banner/`, not rebuilt from `prints/`.
- Do NOT reuse `docs/github/acerola-duo-sidebyside.png` (already used elsewhere) or `01-home`/`02-reader`/`03-*` banners' baked captions as a base for new overlay text — if any of those numbered banners appear at all, show them as-is and do not add a second competing headline on top.

## Audio direction
- Role: warm, restrained ambient/electronic bed with sparse, professional, motion-matched accents — this should feel like a real product film, not a meme edit
- Music: `happy-beats-business-moves-vol-12-by-ende-dot-app.mp3` (bundled) — "steady and clean," the skill's own pick for `polished`/`cinematic`. Tempo ~110 BPM, duration 117s (video only uses the first ~23.5s).
- Music treatment: soft fade-in under the hook (0-2.5s), steady low presence (~0.30-0.35 volume) through the establish/flow scenes, a gentle one-time swell at the hero/confirm moment, slow fade-out under the outro
- Music cue guidance: bundled preset at `assets/music/cues/happy-beats-business-moves-vol-12-by-ende-dot-app.music-cues.md` (+ `.json`). Scene timeline lands almost exactly on a strong cue already: Scene 4 (hero) starts at **11.0s**, and the preset lists a strong cue at **10.93s (0.97)** — lock the hero entrance there (`// beat-locked: 10.93s`). Scene 5 (facts) starts at 16.5s; use the beat grid (16.38s / 16.93s / 17.47s / 18.02s / 18.56s …) to snap each of the three chip arrivals to a nearby beat, spaced at least ~1.0-1.3s apart (i.e. skip beats rather than hit every one). No strong cue sits near the 21.0s outro — let it resolve on natural fade timing rather than forcing a cue.
- Audio-reactive treatment: subtle only — a very light glow/presence pulse on the hero shot tied to the music swell is acceptable; no waveform bars, no beat-flashing text
- SFX posture: sparse, motion-matched, restrained
- Audio-coupled moments:
  - Soft UI tap when each step chip ("01 Install", "02 Pair via QR") arrives
  - A subtle scan/sweep sound while a scanning-line animation crosses the QR code
  - One clean, quiet confirm chime exactly as the hero shot settles (in sync with the sync payoff, not the pre-existing toast graphic)
  - A soft, low "pop" for each of the three fact chips as they arrive
- Restraint rule: never more than one SFX layer at once; no cartoonish or loud sounds; silence is allowed and preferred over filler

## Storyboard

### Scene 1 — Hook — 2.5s
Near-black frame (`#11111b`). The Acerola mark (fruit icon + wordmark) fades/settles in center-frame, small scale-in only (0.95→1.0). Beneath it, one line settles: "No cloud. No account." (verbatim from the landing page's own fact labels).
Sequential/interaction: none.
Audio intent: quiet, confident opening — the claim needs no music swell yet, just presence.
Audio-coupled idea: a soft, single low chime exactly as the wordmark settles.
Music: ambient pad fades in under this scene.
Transition mood: soft crossfade → Scene 2.

### Scene 2 — Two platforms, one library — 3s
Full-bleed crossfade/slow pan across `acerola-desktop-solo.png` and `acerola-android-solo.png` — desktop window and phone both present in frame (side-by-side composition built from the two solo shots, not the already-used sidebyside asset), both showing the same comic covers. One small kicker line: "Same library. Every device."
Sequential/interaction: none (a restrained pan/reveal, not a cut between the two images).
Audio intent: calm establishing confidence.
Audio-coupled idea: none.
Music: pad continues, steady.
Transition mood: clean slide → Scene 3.

### Scene 3 — The flow: Install → Pair — 5.5s
Two sequential sub-beats, ~2.75s each, same visual grammar (a small numbered step chip, top-left, plus one short line):
- **01 — Install** (2.75s): `acerola-android-solo.png` (or a tight crop on the phone). Chip: "01 — Install". Line: "Install on the devices you want to sync." (trim to fit the read-time floor comfortably; prefer the shorter "Install on both devices." if the fuller line crowds the hold.)
- **02 — Pair via QR** (2.75s): `desktop/prints/network-screen.png`, framed/cropped consistently with the banner window chrome. Chip: "02 — Pair via QR". Line: "Scan to pair." A thin scanning-line sweeps once across the real QR code to imply active pairing — this is the one simulated interaction in the scene.
Sequential/interaction: yes — the two step sub-beats arrive one after another, chip label first, then the line; the QR sub-beat additionally simulates a scan sweep across the real QR code.
Audio intent: procedural, confident, unhurried — this is the "watch it happen" section.
Audio-coupled idea: soft tap on each chip's arrival; a subtle scan/sweep sound timed to the scan-line crossing the QR code.
Music: pad steady, no swell yet.
Transition mood: soft crossfade → Scene 4.

### Scene 4 — Confirm & synced (hero payoff) — 5.5s
Full-bleed `acerola-duo-overlap.png`, restrained scale-in (0.96→1.0), then hold. This is step 3 of the flow ("Confirm — it syncs automatically") and the video's hero shot in one: the phone and desktop already show the identical library, and the real "Scan concluído!" toast is visible in the source image — do not add a competing success graphic. One line settles over/beside it, close to the site's own words: "From then on, it syncs automatically, device to device."
Sequential/interaction: none — this is the long confident hold, not another reveal.
Audio intent: the emotional peak of the video — quiet satisfaction, not triumph.
Audio-coupled idea: one clean, quiet confirm chime exactly as the shot settles into its hold.
Music: the one deliberate swell in the whole piece, timed to this scene's entrance.
Transition mood: slow crossfade → Scene 5.

### Scene 5 — Three facts — 4.5s
Clean `#1e1e2e` background. Three short chips arrive one by one, in the landing page's own order and words, each staying on screen once it arrives so all three are visible together for the final ~1.3s:
1. "No cloud, no account"
2. "Direct P2P sync"
3. "Open source"
Sequential/interaction: yes — three chips, one by one, ~1.0-1.3s apart, each with its own small icon (lock/shield, device-link, code-branch — Hyperframes/registry to choose consistent, simple line icons); all three remain visible together at the end.
Audio intent: light, deliberate confirmation — each fact lands like a small, certain fact, not a hype beat.
Audio-coupled idea: a soft, low "pop" on each chip's arrival.
Music: pad begins its slow resolve toward the outro.
Transition mood: soft crossfade → Scene 6.

### Scene 6 — Outro — 2.5s
Back to the Scene 1 frame (`#11111b`), Acerola mark returns, paired with: "Acerola. Open source. No cloud." Hold. No CTA button, no URL — let it sit.
Sequential/interaction: none.
Audio intent: quiet close, mirroring the hook.
Audio-coupled idea: none — let the pad fade out under silence.
Music: fades out fully by the last frame.
Transition mood: hold to end.

**Music mood for this video:** calm, premium, ambient-electronic — quiet confidence throughout with exactly one gentle swell at the hero shot.
**Audio summary:** a restrained ambient bed carries the whole video at low presence, accented only by soft UI taps and a QR scan sweep during the flow, resolving into a single quiet confirm chime at the hero payoff, then fading fully under a silent outro.
