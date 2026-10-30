"""Make the mystery boxes' sounds (`docs/mystery-boxes.md`, SJK's own).

An opened box plays a little music box tune while what is inside rolls over it, slowing
like a wound-down spring, and ends on a chord as the roll stops on what came out. Nothing
here comes from another game: the tune is SJK's own and the sounds are synthesized. This
script writes, into the output folder:

- `roll.wav`: the tune, a music box's plucked tines over a soft bass; its last chord
  starts at `REVEAL` seconds (the client's `mystery_box::ROLL` must match);
- `take.wav`: what the opener hears taking what came out, a quick rising sparkle;

both 22050 Hz mono 16-bit.

    python scripts/mystery_box_assets.py crates/sjk-viewer/assets/mystery_box

The output is deterministic. Needs numpy (pip install numpy).
"""

import sys
import wave
from pathlib import Path

import numpy as np

RATE = 22050
# One eighth of the tune before it slows, in seconds: the last chord at 5.2 s (Sol,
# 30/10/2026: the extras pack's roll stops there; 0.19 before, 5.5 s).
STEP = 0.1796
# How much longer the last eighth lasts than the first (the spring winding down).
SLOWING = 0.9

# The tune in eighths: (MIDI note, eighths held); None is a rest. A minor, a waltz's lilt.
MELODY = [
    (76, 1), (81, 1), (84, 1), (83, 1), (81, 1), (76, 1),
    (77, 1), (81, 1), (86, 1), (84, 1), (83, 1), (80, 1),
    (81, 1), (84, 1), (88, 1), (86, 1), (84, 1), (83, 1),
    (84, 1), (83, 1), (81, 1), (80, 1), (76, 1), (None, 1),
]
# The bass under it, one note a bar of six eighths.
BASS = [57, 62, 57, 52]
# The last chord: A major, the box's "ta-da".
CHORD = [69, 73, 76, 81, 85]


def hz(note: int) -> float:
    return 440.0 * 2 ** ((note - 69) / 12)


def tine(note: int, length: float, loud: float = 1.0) -> np.ndarray:
    """One plucked music box tine: a bright attack and a ring that dies away, its
    overtones out of tune like a steel comb's."""
    t = np.arange(int(length * RATE)) / RATE
    f = hz(note)
    out = np.zeros_like(t)
    for ratio, gain, decay in ((1.0, 1.0, 2.2), (2.0, 0.25, 4.0), (5.4, 0.12, 9.0), (8.9, 0.05, 16.0)):
        out += gain * np.sin(2 * np.pi * f * ratio * t) * np.exp(-decay * t)
    attack = np.minimum(t / 0.002, 1.0)
    return out * attack * loud


def soft_bass(note: int, length: float) -> np.ndarray:
    t = np.arange(int(length * RATE)) / RATE
    f = hz(note)
    out = np.sin(2 * np.pi * f * t) + 0.3 * np.sin(4 * np.pi * f * t)
    return out * np.minimum(t / 0.01, 1.0) * np.exp(-1.6 * t) * 0.35


def mix(into: np.ndarray, sound: np.ndarray, at: float) -> None:
    start = int(at * RATE)
    end = min(len(into), start + len(sound))
    into[start:end] += sound[: end - start]


def onsets() -> tuple[list[float], float]:
    """When each eighth starts, the spring slowing toward the end, and when the chord
    starts."""
    times, now = [], 0.0
    total = sum(held for _, held in MELODY)
    for step in range(total + 1):
        times.append(now)
        now += STEP * (1 + SLOWING * (step / total) ** 3)
    return times[:total], times[total]


def roll() -> tuple[np.ndarray, float]:
    times, reveal = onsets()
    out = np.zeros(int((reveal + 2.0) * RATE))
    step = 0
    for note, held in MELODY:
        if note is not None:
            mix(out, tine(note, 1.6, 0.9 if step % 6 else 1.0), times[step])
        step += held
    for bar, note in enumerate(BASS):
        mix(out, soft_bass(note, 1.4), times[bar * 6])
    for i, note in enumerate(CHORD):
        # Strummed a little, low to high.
        mix(out, tine(note, 2.0, 1.1), reveal + i * 0.025)
    mix(out, soft_bass(45, 2.0) * 1.3, reveal)
    return out, reveal


def take() -> np.ndarray:
    out = np.zeros(int(0.9 * RATE))
    for i, note in enumerate([81, 85, 88, 93, 97]):
        mix(out, tine(note, 0.6, 0.8 + 0.05 * i), i * 0.045)
    return out


def write_wav(path: Path, samples: np.ndarray) -> None:
    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.9
    # A short fade so the file ends in silence.
    fade = min(len(samples), int(0.05 * RATE))
    samples[-fade:] *= np.linspace(1, 0, fade)
    data = (np.clip(samples, -1, 1) * 32767).astype("<i2").tobytes()
    with wave.open(str(path), "wb") as out:
        out.setnchannels(1)
        out.setsampwidth(2)
        out.setframerate(RATE)
        out.writeframes(data)


def main() -> None:
    folder = Path(sys.argv[1])
    folder.mkdir(parents=True, exist_ok=True)
    tune, reveal = roll()
    write_wav(folder / "roll.wav", tune)
    write_wav(folder / "take.wav", take())
    print(f"REVEAL = {reveal:.3f} s")


if __name__ == "__main__":
    main()
