#!/usr/bin/env python3
"""Derive an audio-haptic feedback profile from normalized event features."""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path
from typing import Any


DEFAULT_COEFFICIENTS: dict[str, float] = {
    "alpha_intensity": 0.38,
    "beta_confidence": 0.32,
    "gamma_salience": 0.30,
    "alpha_sharpness": 0.30,
    "beta_transient": 0.40,
    "gamma_edges": 0.30,
    "d_min": 18.0,
    "k_depth": 42.0,
    "k_ambiguity": 56.0,
    "k_resolution": 24.0,
    "f_base": 180.0,
    "k_s": 520.0,
    "k_i": 180.0,
    "f_min": 80.0,
    "f_max": 1200.0,
    "a0": 4.0,
    "a1": 22.0,
    "r0": 18.0,
    "r1": 70.0,
    "transient_weight": 3.0,
    "t_min": 18.0,
    "t_gap_base": 96.0,
    "t_gap_slope": 58.0,
}


FEATURE_DEFAULTS: dict[str, float] = {
    "energy_density": 0.0,
    "certainty": 0.0,
    "event_weight": 0.0,
    "spectral_centroid_norm": 0.0,
    "transient_rise": 0.0,
    "attack_contrast": 0.0,
    "pattern_depth": 0.0,
    "uncertainty": 0.0,
    "settlement_phase": 0.0,
    "event_importance": 0.0,
}


def clamp(value: float, low: float, high: float) -> float:
    return max(low, min(high, value))


def number_from(mapping: dict[str, Any], key: str, fallback: float) -> float:
    raw = mapping.get(key, fallback)
    if isinstance(raw, bool):
        raise TypeError(f"{key} must be numeric, got bool")
    if not isinstance(raw, (int, float)):
        raise TypeError(f"{key} must be numeric, got {type(raw).__name__}")
    return float(raw)


def normalized_features(payload: dict[str, Any]) -> dict[str, float]:
    features: dict[str, float] = {}
    source = payload.get("features", payload)
    if not isinstance(source, dict):
        raise TypeError("features must be an object")

    for key, fallback in FEATURE_DEFAULTS.items():
        value = number_from(source, key, fallback)
        features[key] = clamp(value, 0.0, 1.0)
    return features


def coefficients(payload: dict[str, Any]) -> dict[str, float]:
    merged = dict(DEFAULT_COEFFICIENTS)
    raw = payload.get("coefficients", {})
    if raw is None:
        return merged
    if not isinstance(raw, dict):
        raise TypeError("coefficients must be an object")
    for key, value in raw.items():
        if key not in merged:
            raise KeyError(f"unknown coefficient: {key}")
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            raise TypeError(f"coefficient {key} must be numeric")
        merged[key] = float(value)
    return merged


def derive_profile(payload: dict[str, Any]) -> dict[str, Any]:
    f = normalized_features(payload)
    c = coefficients(payload)

    intensity = clamp(
        c["alpha_intensity"] * f["energy_density"]
        + c["beta_confidence"] * f["certainty"]
        + c["gamma_salience"] * f["event_weight"],
        0.0,
        1.0,
    )
    sharpness = clamp(
        c["alpha_sharpness"] * f["spectral_centroid_norm"]
        + c["beta_transient"] * f["transient_rise"]
        + c["gamma_edges"] * f["attack_contrast"],
        0.0,
        1.0,
    )
    duration_ms = round(
        c["d_min"]
        + c["k_depth"] * f["pattern_depth"]
        + c["k_ambiguity"] * f["uncertainty"]
        + c["k_resolution"] * f["settlement_phase"]
    )
    frequency_hz = clamp(
        c["f_base"] + c["k_s"] * sharpness + c["k_i"] * intensity,
        c["f_min"],
        c["f_max"],
    )
    attack_ms = round(c["a0"] + c["a1"] * (1.0 - sharpness))
    release_ms = round(c["r0"] + c["r1"] * (1.0 - intensity))
    transient_count = 1 + math.floor(c["transient_weight"] * f["event_importance"])
    transient_spacing_ms = max(
        c["t_min"],
        round(c["t_gap_base"] - c["t_gap_slope"] * f["certainty"]),
    )

    return {
        "features": f,
        "haptic": {
            "intensity": round(intensity, 4),
            "sharpness": round(sharpness, 4),
            "duration_ms": int(duration_ms),
            "transient_count": int(transient_count),
            "transient_spacing_ms": int(transient_spacing_ms),
        },
        "audio": {
            "frequency_hz": round(frequency_hz, 2),
            "attack_ms": int(attack_ms),
            "release_ms": int(release_ms),
        },
    }


def load_payload(args: argparse.Namespace) -> dict[str, Any]:
    if args.file:
        text = Path(args.file).read_text(encoding="utf-8")
    elif args.json:
        text = args.json
    else:
        text = sys.stdin.read()

    payload = json.loads(text)
    if not isinstance(payload, dict):
        raise TypeError("input JSON must be an object")
    return payload


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Derive an audio-haptic profile from normalized event features."
    )
    parser.add_argument("json", nargs="?", help="JSON object with event features")
    parser.add_argument("--file", help="Read JSON object from a file")
    parser.add_argument("--compact", action="store_true", help="Emit compact JSON")
    args = parser.parse_args()

    try:
        profile = derive_profile(load_payload(args))
    except Exception as error:
        print(f"derive_feedback.py: {error}", file=sys.stderr)
        return 1

    indent = None if args.compact else 2
    print(json.dumps(profile, indent=indent, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
