//! `r_lampLights`: how strongly the lamps SJK infers light their surroundings in
//! real-time lighting (`r_dayNight 1`).
//!
//! A retail lamp drawn with a `glow` overlay (`byss/lights`, `taspir/light_ceiling`)
//! or a vertex-coloured flare (`flares/solid_blue`) declares no `q3map_surfacelight`
//! power: the map compiler lit its room from point-light entities and stripped them,
//! so in real-time lighting the overlay itself becomes the source
//! ([fixture inference](../../docs/rendering.md#inferring-fixture-light-from-legacy-materials)).
//! Its power is an artistic convention (white overlay radiance 16 times its mean), and
//! Sol's world notes (12-13/10/2026) found these lamps too weak to colour the walls
//! they stand on. This multiplier scales that inferred power, in the lamp's own
//! colour, when a map loads. Declared surface lights, self-lit paint and emission maps
//! (`r_emissiveLights`) keep their own power, and baked lighting is unchanged.
use sjk_shell::{CvarDefinition, CvarError, CvarFlags, CvarRegistry, CvarValue};
use std::sync::atomic::{AtomicU32, Ordering};

/// The cvar's name.
pub(crate) const CVAR: &str = "r_lampLights";
/// The default multiplier. At 1 a `byss/lights` disc on `mp/ffa1` (radiance about
/// 0.1 per unit area over a 48-unit face) reached about 300 units and lit its wall
/// barely above the bounce; 4 lets it cast a visible pool of its blue on the walls
/// under it without whitening them (world shots of the 13/10/2026 notes).
pub(crate) const DEFAULT: f32 = 4.0;
/// Most the inferred power may be multiplied.
pub(crate) const MAX: f32 = 8.0;
/// The current multiplier's bits ([`DEFAULT`] until registered), read when a map loads.
static GAIN: AtomicU32 = AtomicU32::new(0x4080_0000);

/// The multiplier a cvar value asks for: 0 (inferred lamps light nothing) to [`MAX`],
/// [`DEFAULT`] for a non-number.
pub(crate) fn gain_of(value: &CvarValue) -> f32 {
    let gain = match value {
        CvarValue::Float(value) => *value as f32,
        CvarValue::Integer(value) => *value as f32,
        CvarValue::Bool(on) => f32::from(u8::from(*on)),
        CvarValue::Text(_) => DEFAULT,
    };
    if gain.is_finite() {
        gain.clamp(0.0, MAX)
    } else {
        DEFAULT
    }
}

/// The multiplier for the map being loaded.
pub(crate) fn gain() -> f32 {
    f32::from_bits(GAIN.load(Ordering::Relaxed))
}

/// Register the cvar and follow its changes (they apply on the next map load).
pub(crate) fn register(cvars: &mut CvarRegistry) -> Result<(), CvarError> {
    cvars.register(CvarDefinition::new(
        CVAR,
        f64::from(DEFAULT),
        CvarFlags::ARCHIVE,
        "Light cast by lamps that only glow (glow overlays, flares) in real-time \
         lighting (r_dayNight 1): 0 none, 4 default, up to 8; applies when a map loads",
    ))?;
    let set = |value: &CvarValue| GAIN.store(gain_of(value).to_bits(), Ordering::Relaxed);
    set(&cvars.get(CVAR).expect("registered").value);
    cvars.on_change(CVAR, move |change| set(&change.current))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stored_default_is_the_default() {
        assert_eq!(f32::from_bits(0x4080_0000), DEFAULT);
    }

    #[test]
    fn values_are_clamped_and_text_falls_back() {
        assert_eq!(gain_of(&CvarValue::Float(2.5)), 2.5);
        assert_eq!(gain_of(&CvarValue::Float(-1.0)), 0.0);
        assert_eq!(gain_of(&CvarValue::Integer(20)), MAX);
        assert_eq!(gain_of(&CvarValue::Text("bright".into())), DEFAULT);
        assert_eq!(gain_of(&CvarValue::Float(f64::NAN)), DEFAULT);
    }
}
