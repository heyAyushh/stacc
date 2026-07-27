# Mobile Participant Checklist

Use this reference when implementing haptics in a mobile app with local UI, backend, realtime, wallet, or Solana participants.

## Event Inventory

Build a table before coding:

```text
event | participant | user-visible state | certainty | salience | uncertainty | primitive | fallback
```

Participants to check:

- Local UI: taps, toggles, segmented controls, tabs, sheet open/close, form validation, scroll detents, sliders, pull-to-refresh, route transitions.
- Gesture system: drag start, threshold approach, threshold crossing, snap completion, rejected snap, fling/overshoot, release/cancel.
- Auth and wallet: auth prompt opened, passkey request, wallet connect, signature request, approval, cancellation, rejection.
- Backend: request queued, accepted, validation failed, rate-limited, timed out, stale cache, success payload received.
- Realtime: websocket connected, reconnected, joined room, market invalidated, chat sent, chat failed, participant state changed.
- Solana: simulation/preflight result, transaction built, signature obtained, submission accepted, confirmation observed, finalization observed, failure/revert/rollback.
- Settlement: optimistic state committed, optimistic state rolled back, conflict resolved, market/account state refreshed.

## Mapping Guidance

- Local direct manipulation can be immediate because certainty is high.
- Backend accepted states should be lower energy than final settlement unless the user only needs request acknowledgement.
- Wallet approvals should feel decisive; wallet cancellations should be softer and shorter than failures.
- Solana submission is not finality. Use a moderate transient for submitted and a cleaner terminal transient for confirmed/finalized.
- Realtime updates not caused by the current user should be subtle or silent unless they change the active task.
- Errors should be sharp enough to be noticed but not theatrical for background work.

## Implementation Pattern

Centralize the API:

```text
semantic event -> feature vector -> derived profile -> platform adapter
```

Recommended module responsibilities:

- `events`: stable semantic event names and participants.
- `model`: mapping from event features to intensity, sharpness, duration, transients, and optional audio.
- `adapter`: platform calls such as iOS Core Haptics, UIKit feedback generators, Expo Haptics, Android vibration primitives, or no-op fallbacks.
- `guards`: accessibility, reduced motion, device support, foreground state, and rate limits.
- `tests`: deterministic mapping tests and callsite coverage for important user-visible states.

Avoid:

- Direct haptic calls scattered across feature files.
- Repeating pulses for every frame of a continuous gesture.
- Treating optimistic UI, backend success, and on-chain finality as the same event.
- Inventing backend endpoints only to drive haptics. Use existing semantic state where possible.

## Calibration Defaults

Use conservative defaults first:

- Tap or low-salience acknowledgement: low intensity, medium sharpness, short duration.
- Toggle on: medium intensity, high sharpness, short duration.
- Toggle off: low-medium intensity, medium sharpness, short duration.
- Snap-to-detent success: medium-high intensity, high sharpness, very short transient.
- Rejected partial move: low intensity, medium-low sharpness, short correction.
- Backend pending: usually no haptic or one low transient if the user initiated a visible operation.
- Backend accepted: low-medium intensity, medium sharpness.
- Wallet approved: medium-high intensity, high sharpness.
- Wallet cancelled: low intensity, soft sharpness.
- Solana submitted: medium intensity, medium-high sharpness.
- Solana confirmed/finalized: high confidence, crisp transient, short duration.
- Rollback/failure: medium intensity, high sharpness, short but not repetitive.

## Verification

Verify with:

- Static checks for the app language and framework.
- Unit tests for deterministic event-to-profile mapping.
- Search confirming important backend, realtime, wallet, and Solana callsites use the central module.
- Real device checks for iOS or Android haptics when possible, because simulators and web fallbacks can lie.
