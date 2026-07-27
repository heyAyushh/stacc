# Canonical Audio Math Haptics Model

Use this reference when exact mapping laws are needed. Keep implementation pages subordinate to this model.

## Contents

- Abstract Domains
- Canonical Mapping Functions
- Audio-Haptic Coupling Law
- UI Kinetic Primitives
- Design Notes

## Abstract Domains

Audio signal space:

```text
A(t) = {f(t), e(t), x(t)}
```

- `f(t)`: dominant frequency trajectory
- `e(t)`: amplitude envelope
- `x(t)`: transient impulse structure

Haptic primitive space:

```text
H(t) = {i(t), s(t), d(t)}
```

- `i(t)`: intensity
- `s(t)`: sharpness
- `d(t)`: duration

UI kinetic space:

```text
U(t) = {x(t), theta(t), v(t), omega(t), mu, J, K_d}
```

- `x(t)`: linear position
- `theta(t)`: rotational angle for wheel-like inputs
- `v(t)`: linear velocity
- `omega(t)`: angular velocity
- `mu`: friction coefficient
- `J`: rotational inertia
- `K_d`: detent stiffness

## Canonical Mapping Functions

Intensity:

```text
i = clamp(
  alpha_intensity * energy_density
  + beta_confidence * certainty
  + gamma_salience * event_weight,
  0,
  1
)

energy_density = integral(e(t)^2 dt) over the response window
```

Sharpness:

```text
s = clamp(
  alpha_sharpness * spectral_centroid_norm
  + beta_transient * transient_rise
  + gamma_edges * attack_contrast,
  0,
  1
)

spectral_centroid_norm = normalized spectral centroid of the audio band
```

Duration:

```text
d_ms = round(
  d_min
  + k_depth * pattern_depth
  + k_ambiguity * uncertainty
  + k_resolution * settlement_phase
)
```

Duration decreases as ambiguity collapses and increases as narrative depth rises.

Frequency:

```text
f0_hz = clamp(f_base + k_s * s + k_i * i, f_min, f_max)
```

Higher sharpness drives higher pitch. Higher intensity may increase harmonic brightness.

Envelope:

```text
attack_ms = round(a0 + a1 * (1 - s))
release_ms = round(r0 + r1 * (1 - i))
```

Envelope slope should remain monotonic with perceived urgency.

Transient:

```text
transient_count = 1 + floor(transient_weight * event_importance)
transient_spacing_ms = max(t_min, round(t_gap_base - t_gap_slope * certainty))
```

Transient energy should be concentrated earlier for high-certainty events.

## Audio-Haptic Coupling Law

- More certainty compresses time.
- More salience increases intensity and amplitude together.
- More sharpness increases pitch, edge, and attack speed.
- More transients imply stronger semantic discontinuity.
- One event should shape both modalities with one shared causal clock.

## UI Kinetic Primitives

### Scroll Dynamics

```text
J * theta_ddot + mu * theta_dot + dV_detent/dtheta = tau_input
V_detent(theta) = D * (1 - cos(n * theta))
detent_spacing_rad = 2*pi / n
snap_energy = 0.5 * J * omega^2 - V_detent(theta)
```

If snap energy exceeds threshold, the picker crosses to the nearest stable notch.

### Date/Time Picker Mapping

```text
omega_abs = abs(theta_dot)
detent_alignment = 1 - min(abs(phase_to_nearest_notch) / detent_half_width, 1)
friction_loss = mu * omega_abs + nonlinear_drag(omega_abs)
haptic_sharpness rises with detent alignment, snap energy, and angular velocity
haptic_intensity rises with snap energy, edge crossing, and correction force
audio_pitch_hz rises with angular velocity, detent alignment, and step crossings
audio_pitch_delta = sign(delta_detent) * (k_step + k_velocity * omega_abs)
audio_volume = clamp(v_base + v_i * haptic_intensity, 0, 1)
```

### Detent Law

- Each notch is a local minimum in the UI potential field.
- Higher inertia increases overshoot probability.
- Friction increases dwell time near the notch and lowers pitch glide rate.
- A successful snap should produce a crisp haptic transient plus a short pitch rise.
- A rejected partial move should produce a weaker transient and a damped pitch correction.

### Toggle Primitive

```text
transition_energy_toggle = edge_crossing + state_flip_cost
```

- Toggle feedback should be instantaneous, binary, and non-oscillatory.
- On-state transitions should feel like a clean circuit closing.
- Off-state transitions should feel slightly softer but still decisive.

### Slider Primitive

```text
slider_response = f(position, velocity, acceleration)
```

- Haptic intensity should increase with velocity near semantic thresholds.
- Haptic sharpness should increase at labeled breakpoints and snap regions.
- Audio pitch should rise slightly with positive motion and fall with reverse motion.
- Continuous drag should remain continuous and low-amplitude rather than spiking repeatedly.

### Page Curl Primitive

```text
curl_tension = bend_angle * resistance_coefficient
curl_release = integrated_tension - return_loss
```

- Haptic sharpness should spike at the release threshold.
- Audio should form a brief ascending edge during lift and a soft landing tone on completion.
- Page curl completion should resolve as a single, theatrical transition.

## Design Notes

- Date/time pickers should feel like matter moving through a gravitational field of detents.
- Toggles should feel like a clean circuit closing.
- Sliders should feel like controlled momentum, never like a broken loop.
- Page curls should feel theatrical, but the snap must remain mathematically decisive.
- Accessibility and fallback behavior can reduce output energy, but should preserve the same underlying mapping structure.
