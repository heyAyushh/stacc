---
name: audio-math-haptics
description: Derive and implement first-principles audio-coupled haptic and kinetic UI feedback. Use when designing, reviewing, or coding haptics/audio for mobile apps, native iOS/Android, React Native/Expo, SwiftUI, games, detent pickers, toggles, sliders, page curls, scroll gestures, wallet/signature flows, Solana transactions, backend/realtime state changes, confirmations, settlements, and semantic UI events where feedback should follow signal math instead of ad hoc vibration calls.
license: MIT
---

# Audio Math Haptics

## Overview

Use this skill to make feedback systems feel causal: one semantic event should shape the haptic profile, optional audio profile, and UI kinetic behavior from the same clock and confidence model.

Do not start by sprinkling isolated haptic calls through the app. First identify the semantic event, its participants, and its uncertainty/settlement path.

## Workflow

1. Inventory the events.
   - Include local UI controls, navigation transitions, async requests, backend acknowledgements, wallet/signature states, Solana transaction phases, realtime invalidations, and settlement/failure states.
   - For each event, record certainty, uncertainty, salience/event weight, semantic discontinuity, pattern depth, and settlement phase.

2. Choose the primitive.
   - Use detent/picker feedback for indexed motion and step crossings.
   - Use toggle feedback for binary state flips.
   - Use slider feedback for continuous motion with semantic thresholds.
   - Use page/route transition feedback for theatrical but decisive transitions.
   - Use async/transaction feedback for pending, accepted, confirmed, rejected, failed, and settled states.

3. Derive the haptic profile.
   - Map semantic features to intensity, sharpness, duration, transient count, and spacing.
   - Increase intensity with salience, snap energy, confirmation, and edge crossing.
   - Increase sharpness with transient rise, detent alignment, correction force, and semantic discontinuity.
   - Compress duration as certainty rises; increase duration for deeper narrative or unresolved settlement.

4. Couple optional audio from the same event.
   - Higher sharpness raises pitch and shortens attack.
   - Higher salience raises intensity and amplitude together.
   - More transients mean stronger discontinuity.
   - Keep audio optional and subordinate to platform settings, mute state, and accessibility.

5. Implement through a central adapter.
   - Create one semantic feedback module per app, then call it from feature surfaces.
   - Keep platform API calls behind the adapter so iOS, Android, web, simulator, and accessibility fallbacks remain consistent.
   - Rate-limit continuous gestures; threshold crossings should feel meaningful, not noisy.
   - Do not let the backend directly "play haptics"; backend, Solana, and realtime systems emit semantic state changes that the client maps to feedback.

6. Validate the whole participant chain.
   - Check local UI, wallet, backend, realtime, and chain states that can be observed by the user.
   - Verify disabled/reduced-motion/reduced-haptics behavior preserves the same event ordering with lower energy.
   - Run the project typecheck/lint/test path and, for mobile haptics, prefer a real device check when feasible.

## Solana And Backend Events

For crypto or backend-backed apps, model the following separately instead of collapsing them into one "success" pulse:

- User intent captured locally.
- Wallet or passkey prompt opened.
- Signature approved, cancelled, or rejected.
- Transaction created/submitted.
- Backend request accepted, rejected, rate-limited, or timed out.
- Optimistic UI state applied.
- Realtime invalidation or participant update received.
- Chain confirmation/finalization observed.
- Settlement, conflict, rollback, or failure resolved.

Map high-certainty terminal states to crisp, short transients. Map pending/ambiguous states to lower-intensity, longer, non-repeating feedback. Map rejected partial movement or cancelled intent to softer correction feedback.

## Reference Files

- Read `references/canonical-model.md` when exact formulas, coefficients, or primitive laws are needed.
- Read `references/mobile-participants.md` when implementing in a mobile app with backend, realtime, wallet, or Solana participants.
- Run `scripts/derive_feedback.py` when a numeric calibration table or concrete profile is useful:

```bash
python3 scripts/derive_feedback.py '{"energy_density":0.3,"certainty":0.8,"event_weight":0.7,"spectral_centroid_norm":0.5,"transient_rise":0.8,"attack_contrast":0.7,"pattern_depth":0.2,"uncertainty":0.1,"settlement_phase":0.0,"event_importance":0.9}'
```
