"""Original music and sounds for the Sith maps, synthesized with NumPy.

- music(): about two minutes in D minor for a dark temple: a low drone, brass
  chords, a wordless choir, timpani and, the second time round, a slow horn
  line, all in a long hall's reverb. Its end crossfades into its start, so the
  game's repeat of the track loops without a seam. Encoded to MP3 with `lame`.
- lift(): a platform's ride (a clunk, a motor's hum and whine, a clunk).
- lava_loop(), wind_loop(): seamless ambient loops for looping speakers.

Nothing here is sampled from anything: every sound is sines, noise and envelopes.
"""
import shutil
import subprocess
import wave

import numpy as np

MUSIC_RATE = 32000
SFX_RATE = 22050


def midi(note):
    return 440.0 * 2 ** ((note - 69) / 12)


def lowpass(signal, rate, cutoff, order=4):
    """A gentle FFT low-pass over the whole signal (works along the last axis)."""
    n = signal.shape[-1]
    spectrum = np.fft.rfft(signal, axis=-1)
    f = np.fft.rfftfreq(n, 1 / rate)
    spectrum *= 1 / (1 + (f / cutoff) ** (2 * order)) ** 0.5
    return np.fft.irfft(spectrum, n, axis=-1)


def bandpass(signal, rate, low, high):
    return lowpass(signal, rate, high) - lowpass(signal, rate, low)


def envelope(n, rate, attack, release):
    """Rise over `attack` seconds, fall over the last `release`."""
    e = np.ones(n)
    a, r = int(attack * rate), int(release * rate)
    if a:
        e[:a] = np.linspace(0, 1, a) ** 1.5
    if r:
        e[-r:] *= np.linspace(1, 0, r) ** 1.5
    return e


def harmonic_tone(freq, t, weights, vibrato=0.0, vib_rate=5.0, phase=0.0):
    """A tone of harmonics `weights[k]` (k from 1), with an optional vibrato."""
    wobble = 1 + vibrato * np.sin(2 * np.pi * vib_rate * t + phase)
    base_phase = 2 * np.pi * freq * np.cumsum(wobble) * (t[1] - t[0])
    out = np.zeros_like(t)
    for k, w in enumerate(weights, start=1):
        if w and freq * k < 12000:
            out += w * np.sin(k * base_phase + phase * k)
    return out


def formant_weights(freq, count=14):
    """Harmonic weights shaped like an open 'ah' (formants near 700, 1220, 2600 Hz)."""
    w = []
    for k in range(1, count + 1):
        f = freq * k
        g = (np.exp(-((f - 700) / 180) ** 2) + 0.6 * np.exp(-((f - 1220) / 200) ** 2)
             + 0.25 * np.exp(-((f - 2600) / 300) ** 2) + 0.15 / k)
        w.append(g)
    return w


def reverb(signal, rate, seconds=2.8, seed=3):
    """Stereo hall reverb by convolution with decaying noise; returns (2, n)."""
    rng = np.random.default_rng(seed)
    n_ir = int(seconds * rate)
    t = np.arange(n_ir) / rate
    out = []
    for channel in range(2):
        ir = rng.standard_normal(n_ir) * np.exp(-t / (seconds / 3.5))
        ir = lowpass(ir, rate, 5000 - 1500 * channel)
        ir[0] = 0
        ir /= np.sqrt(np.sum(ir ** 2))
        size = 1 << int(np.ceil(np.log2(len(signal) + n_ir)))
        wet = np.fft.irfft(np.fft.rfft(signal, size) * np.fft.rfft(ir, size), size)[:len(signal) + n_ir]
        out.append(wet)
    return np.array(out)


# ---------------------------------------------------------------- music

# Each chord lasts a bar of eight seconds: Dm, Bb, Gm, A, Dm, Eb, C, A.
CHORDS = [(50, 53, 57), (46, 50, 53), (43, 46, 50), (45, 49, 52),
          (50, 53, 57), (51, 55, 58), (48, 52, 55), (45, 49, 52)]
BAR = 8.0
# The horn's line in the second round: (note, start in seconds from the round's
# start, length in seconds).
MELODY = [(62, 0, 2), (65, 2, 1), (64, 3, 1), (62, 4, 2), (57, 6, 2),
          (58, 8, 2), (57, 10, 1), (55, 11, 1), (57, 12, 4),
          (55, 16, 2), (58, 18, 2), (57, 20, 2), (55, 22, 2),
          (57, 24, 3), (61, 27, 1), (64, 28, 4),
          (62, 32, 2), (65, 34, 1), (64, 35, 1), (62, 36, 2), (69, 38, 2),
          (70, 40, 3), (67, 43, 1), (63, 44, 4),
          (64, 48, 2), (67, 50, 2), (64, 52, 2), (60, 54, 2),
          (61, 56, 2), (64, 58, 2), (62, 60, 4)]


def music(path, rounds=2):
    rate = MUSIC_RATE
    length = rounds * len(CHORDS) * BAR
    n = int(length * rate)
    t = np.arange(n) / rate
    rng = np.random.default_rng(11)
    dry = np.zeros(n)

    # a drone on D, breathing slowly
    breath = 0.75 + 0.25 * np.sin(2 * np.pi * t / 16)
    drone = harmonic_tone(midi(26), t, [1 / k for k in range(1, 9)])
    drone += 0.6 * harmonic_tone(midi(38), t, [1 / k for k in range(1, 7)], 0.002, 0.3)
    dry += 0.10 * lowpass(drone, rate, 260) * breath

    bar_n = int(BAR * rate)
    for r in range(rounds):
        for b, chord in enumerate(CHORDS):
            start = int((r * len(CHORDS) + b) * BAR * rate)
            seg_n = min(bar_n + rate, n - start)  # a second's overlap into the next bar
            ts = np.arange(seg_n) / rate
            env = envelope(seg_n, rate, 1.2, 1.3)
            # brass: two slightly detuned voices per chord tone
            brass = np.zeros(seg_n)
            for note in chord:
                for detune in (0.9975, 1.0025):
                    brass += harmonic_tone(midi(note) * detune, ts, [1 / k ** 0.9 for k in range(1, 11)])
            brass = lowpass(brass, rate, 900 + 500 * r)
            dry[start:start + seg_n] += 0.035 * brass * env
            # choir an octave up, from the second half of the first round on
            if r > 0 or b >= 4:
                choir = np.zeros(seg_n)
                for k, note in enumerate(chord):
                    f = midi(note + 12)
                    choir += harmonic_tone(f, ts, formant_weights(f), 0.006, 4.6 + 0.4 * k, k)
                dry[start:start + seg_n] += 0.02 * (1 + 0.5 * r) * choir * envelope(seg_n, rate, 2.5, 1.5)
            # timpani on the bar and, more often later, on its last beats
            hits = [(0.0, 1.0)] + ([(6.0, 0.5), (7.0, 0.7)] if r > 0 or b % 2 else [(4.0, 0.4)])
            root = midi(chord[0] - 24)
            for at, loud in hits:
                h = start + int(at * rate)
                hn = min(int(1.6 * rate), n - h)
                if hn <= 0:
                    continue
                th = np.arange(hn) / rate
                pitch = root * (1 + 0.15 * np.exp(-th * 18))
                body = np.sin(2 * np.pi * np.cumsum(pitch) / rate) * np.exp(-th * 2.6)
                skin = lowpass(rng.standard_normal(hn), rate, 600) * np.exp(-th * 30) * 0.6
                dry[h:h + hn] += 0.30 * loud * (body + skin)

    # the horn, in the second round
    if rounds > 1:
        offset = len(CHORDS) * BAR
        for note, at, dur in MELODY:
            s = int((offset + at) * rate)
            m_n = min(int((dur + 0.6) * rate), n - s)
            tm = np.arange(m_n) / rate
            horn = harmonic_tone(midi(note), tm, [1, 0.7, 0.45, 0.3, 0.2, 0.12, 0.08], 0.004, 5.2)
            horn = lowpass(horn, rate, 1800)
            dry[s:s + m_n] += 0.06 * horn * envelope(m_n, rate, 0.25, 0.6)

    # a low boom where each round begins
    for r in range(rounds):
        s = int(r * len(CHORDS) * BAR * rate)
        bn = int(4 * rate)
        tb = np.arange(bn) / rate
        dry[s:s + bn] += 0.35 * lowpass(rng.standard_normal(bn), rate, 120) * np.exp(-tb * 1.2)

    wet = reverb(dry, rate)
    mix = np.stack([dry, dry]) * 0.75 + wet[:, :n] * 0.55
    # the reverb's tail after the end goes back over the start, so the loop is seamless
    tail = wet[:, n:]
    mix[:, :tail.shape[1]] += tail * 0.55
    mix = np.tanh(mix / np.max(np.abs(mix)) * 1.4) / np.tanh(1.4) * 0.89
    write_wav_stereo(path.with_suffix(".wav"), mix, rate)
    encode_mp3(path.with_suffix(".wav"), path)
    path.with_suffix(".wav").unlink()


def write_wav_stereo(path, stereo, rate):
    frames = (np.clip(stereo.T, -1, 1) * 32767).astype("<i2").tobytes()
    with wave.open(str(path), "wb") as w:
        w.setnchannels(2)
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes(frames)


def encode_mp3(wav, mp3):
    lame = shutil.which("lame")
    if not lame:
        raise SystemExit("lame is needed to encode the music (or keep the .wav)")
    subprocess.run([lame, "--quiet", "-V", "4", str(wav), str(mp3)], check=True)


# ---------------------------------------------------------------- effects

def write_wav(path, mono, rate=SFX_RATE):
    path.parent.mkdir(parents=True, exist_ok=True)
    data = (np.clip(mono, -1, 1) * 32767).astype("<i2").tobytes()
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes(data)


def clunk(n, rate, rng):
    t = np.arange(n) / rate
    thump = np.sin(2 * np.pi * 55 * t) * np.exp(-t * 14)
    clang = bandpass(rng.standard_normal(n), rate, 900, 3200) * np.exp(-t * 40) * 0.5
    ring = np.sin(2 * np.pi * 410 * t) * np.exp(-t * 9) * 0.15
    return thump + clang + ring


def lift(path, ride=4.6):
    """The lift's whole ride: a clunk, the motor rising, running, slowing, a clunk."""
    rate = SFX_RATE
    rng = np.random.default_rng(5)
    n = int((ride + 0.8) * rate)
    t = np.arange(n) / rate
    speed = np.clip(t / 0.6, 0, 1) * np.clip((ride - t) / 0.5, 0, 1)
    hum_f = 70 + 25 * speed
    hum = np.sin(2 * np.pi * np.cumsum(hum_f) / rate) + 0.5 * np.sin(4 * np.pi * np.cumsum(hum_f) / rate)
    whine = np.sin(2 * np.pi * np.cumsum(520 + 160 * speed) / rate) * 0.18
    rumble = lowpass(rng.standard_normal(n), rate, 300) * 0.5
    motor = (0.5 * hum + whine + rumble) * (0.15 + 0.85 * speed)
    out = motor * 0.5
    c = int(0.4 * rate)
    out[:c] += clunk(c, rate, rng) * 0.8
    end = int(ride * rate)
    out[end:end + c] += clunk(min(c, n - end), rate, rng)[: n - end] * 0.9
    write_wav(path, out / np.max(np.abs(out)) * 0.9)


def loop(signal, rate, fade=1.0):
    """Crossfade the end into the start so the sound repeats without a seam."""
    f = int(fade * rate)
    head, body, tail = signal[:f], signal[f:-f], signal[-f:]
    k = np.linspace(0, 1, f)
    return np.concatenate([tail * (1 - k) + head * k, body])


def lava_loop(path, seconds=8.0):
    rate = SFX_RATE
    rng = np.random.default_rng(7)
    n = int((seconds + 1) * rate)
    t = np.arange(n) / rate
    rumble = lowpass(rng.standard_normal(n), rate, 140) * 1.4
    hiss = bandpass(rng.standard_normal(n), rate, 1500, 5000) * 0.05
    out = rumble + hiss
    for _ in range(int(seconds * 3)):  # bubbles bursting
        s = rng.integers(0, n - rate // 2)
        bn = int(rng.uniform(0.08, 0.25) * rate)
        tb = np.arange(bn) / rate
        f0 = rng.uniform(60, 180)
        out[s:s + bn] += np.sin(2 * np.pi * f0 * (1 + 2 * tb) * tb) * np.exp(-tb * 18) * rng.uniform(0.3, 0.9)
    out = loop(out, rate)
    write_wav(path, out / np.max(np.abs(out)) * 0.8)


def wind_loop(path, seconds=12.0):
    rate = SFX_RATE
    rng = np.random.default_rng(9)
    n = int((seconds + 1) * rate)
    t = np.arange(n) / rate
    gust = 0.55 + 0.45 * np.sin(2 * np.pi * t / 6.5) * np.sin(2 * np.pi * t / 2.9 + 1)
    out = bandpass(rng.standard_normal(n), rate, 180, 900) * gust
    out += bandpass(rng.standard_normal(n), rate, 900, 2400) * gust ** 2 * 0.25
    out = loop(out, rate)
    write_wav(path, out / np.max(np.abs(out)) * 0.7)
