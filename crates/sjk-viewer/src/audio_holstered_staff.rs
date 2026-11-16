//! Bounded EJK #267 ignition/shutdown synchronization. No decoding in frame paths.
use super::*;
use crate::holstered_sabers::staff::Phase;

const CLIENTS: usize = 64;
const WAIT: i64 = 150;
const HOLD: i64 = 2000;

#[derive(Clone, Copy, Default)]
struct Slot {
    identity: u64,
    sounds: [[Option<SoundHandle>; 3]; 2],
    enabled: bool,
    origin: [f32; 3],
    phase: Phase,
    lit: bool,
    unholster_time: Option<i64>,
    pending: [Option<(SoundHandle, PlayRequest, i64, bool)>; 3],
    /// Weapon switch shutdowns waiting on the pose, with the snapshot server time
    /// whose saber toggle cap they paid at.
    pending_off: [Option<(SoundHandle, PlayRequest, i32)>; 3],
    was_active: bool,
    off_key: Option<(usize, u64, i64)>,
    off_time: Option<i64>,
    seen: bool,
    /// The last reported presentation time, for a release on an unseen frame.
    now: i64,
    /// A late shutdown copy dropped after it paid the toggle cap at this snapshot
    /// server time: the toggle is given back ([`Sync::pay_back`]).
    refund: Option<i32>,
}

impl Slot {
    /// Release the server's deferred copies instead of losing them when the slot
    /// is disabled or reset (death, mount, cloak, an actor no longer submitted):
    /// held ignitions, and shutdowns the reach has not already played.
    fn take_deferred(&mut self, client: usize) -> [Option<(SoundHandle, PlayRequest)>; 6] {
        let covered = self
            .off_time
            .is_some_and(|time| (0..HOLD).contains(&(self.now - time)));
        let ignitions = std::mem::take(&mut self.pending).map(|pending| {
            pending.map(|(handle, mut request, ..)| {
                request.origin = Some(self.origin);
                request.source = SourceId(client as u32);
                (handle, request)
            })
        });
        let offs = std::mem::take(&mut self.pending_off).map(|off| {
            let (handle, request, paid) = off?;
            if covered {
                self.refund = Some(paid);
                return None;
            }
            Some((handle, request))
        });
        std::array::from_fn(|index| {
            if index < 3 {
                ignitions[index]
            } else {
                offs[index - 3]
            }
        })
    }

    fn release_ignitions(
        &mut self,
        client: usize,
        origin: [f32; 3],
        phase: Phase,
        now: i64,
    ) -> [Option<(SoundHandle, PlayRequest)>; 3] {
        std::array::from_fn(|layer| {
            let (handle, mut request, started, mut saw_back) = self.pending[layer]?;
            saw_back |= phase == Phase::OnBack;
            if now - started >= HOLD
                || (phase != Phase::OnBack && (saw_back || now - started >= WAIT))
            {
                request.origin = Some(origin);
                request.source = SourceId(client as u32);
                self.pending[layer] = None;
                Some((handle, request))
            } else {
                self.pending[layer] = Some((handle, request, started, saw_back));
                None
            }
        })
    }
}

pub(super) struct Sync {
    slots: [Slot; CLIENTS],
    time: i64,
    pub(super) enabled: bool,
}
impl Default for Sync {
    fn default() -> Self {
        Self {
            slots: [Slot::default(); CLIENTS],
            time: 0,
            enabled: true,
        }
    }
}
impl Sync {
    /// A new snapshot records the instant the server says the staff lit up.
    pub(super) fn observe(&mut self, snapshot: &Snapshot) {
        let now = i64::from(snapshot.server_time);
        if now < self.time {
            self.slots.fill(Slot::default());
        }
        self.time = now;
        for state in &snapshot.entities {
            let Some(slot) = self.slots.get_mut(usize::from(state.number())) else {
                continue;
            };
            let lit = state.weapon() == 3 && state.saber_holstered() == 0;
            if lit && !slot.lit {
                slot.unholster_time = Some(now);
            }
            slot.lit = lit;
        }
        if let Some(slot) = self
            .slots
            .get_mut(usize::from(snapshot.player.client_num()))
        {
            let lit = snapshot.player.weapon() == 3 && snapshot.player.saber_holstered() == 0;
            if lit && !slot.lit {
                slot.unholster_time = Some(now);
            }
            slot.lit = lit;
        }
    }

    pub(super) fn intercept(
        &mut self,
        handle: SoundHandle,
        request: PlayRequest,
        now: i64,
    ) -> bool {
        if !self.enabled {
            return false;
        }
        for (index, slot) in self.slots.iter_mut().enumerate() {
            if !slot.enabled {
                continue;
            }
            let from_client = request.source.0 == index as u32;
            let near = request.origin.is_some_and(|origin| {
                glam::Vec3::from_array(origin).distance_squared(glam::Vec3::from_array(slot.origin))
                    <= 64.0 * 64.0
            });
            if slot.sounds[1].contains(&Some(handle)) && from_client {
                let paid = i32::try_from(now).unwrap_or(i32::MAX);
                if slot
                    .off_time
                    .is_some_and(|time| (0..HOLD).contains(&(now - time)))
                {
                    // The late copy of a shutdown the reach voiced and paid for.
                    slot.refund = Some(paid);
                } else {
                    let layer = slot.sounds[1]
                        .iter()
                        .position(|h| *h == Some(handle))
                        .expect("matched sound");
                    slot.pending_off[layer] = Some((handle, request, paid));
                }
                return true; // The current pose decides whether this is the late copy.
            }
            if slot.sounds[0].contains(&Some(handle))
                && (from_client || (near && request.source.0 >= CLIENTS as u32))
                && slot.lit
                && (slot.phase == Phase::OnBack
                    || slot
                        .unholster_time
                        .is_some_and(|time| (0..=WAIT).contains(&(now - time))))
            {
                let layer = slot.sounds[0]
                    .iter()
                    .position(|h| *h == Some(handle))
                    .expect("matched sound");
                slot.pending[layer] = Some((handle, request, now, slot.phase == Phase::OnBack));
                return true;
            }
        }
        false
    }

    /// Give back to the saber toggle cap what dropped late copies paid.
    pub(super) fn pay_back(&mut self, adapter: &mut LegacySoundAdapter) {
        for (client, slot) in self.slots.iter_mut().enumerate() {
            if let Some(paid) = slot.refund.take() {
                adapter.refund_saber_toggle(client as u16, paid);
            }
        }
    }
}

/// [`Sync::pay_back`] for one slot, before it is reset or after it dropped a copy.
fn pay_back(legacy: &mut Option<LegacySoundAdapter>, client: usize, slot: &mut Slot) {
    if let Some(paid) = slot.refund.take()
        && let Some(adapter) = legacy
    {
        adapter.refund_saber_toggle(client as u16, paid);
    }
}

impl GameAudio {
    /// Begin a primary actor pass; portal passes must not own audio state.
    pub(crate) fn begin_holsters(&mut self) {
        for slot in &mut self.holster_sync.slots {
            slot.seen = false;
        }
    }

    /// Report a staff carrier whose reach is synchronised this frame; a slot not
    /// reported is released by [`Self::end_holsters`].
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn holstered_staff(
        &mut self,
        client: usize,
        identity: u64,
        origin: [f32; 3],
        phase: Phase,
        putting_away: bool,
        key: Option<(usize, u64, i64)>,
        active: bool,
        now: i64,
    ) {
        let handles = self.legacy.as_ref().map_or([[None; 3]; 2], |adapter| {
            adapter.saber_switch_handles(client as u16)
        });
        let Some(slot) = self.holster_sync.slots.get_mut(client) else {
            return;
        };
        if slot.identity != identity || (slot.sounds != handles && slot.sounds != [[None; 3]; 2]) {
            for (handle, request) in slot.take_deferred(client).into_iter().flatten() {
                self.output.send(AudioCommand::Play(handle, request));
            }
            pay_back(&mut self.legacy, client, slot);
            *slot = Slot::default();
        }
        slot.identity = identity;
        slot.sounds = handles;
        slot.enabled = self.holster_sync.enabled;
        slot.origin = origin;
        slot.phase = phase;
        slot.seen = true;
        slot.now = now;
        if !slot.enabled {
            for (handle, request) in slot.take_deferred(client).into_iter().flatten() {
                self.output.send(AudioCommand::Play(handle, request));
            }
            pay_back(&mut self.legacy, client, slot);
            slot.off_key = None;
            slot.off_time = None;
            return;
        }
        if putting_away
            && phase != Phase::None
            && (active || slot.was_active)
            && key.is_some()
            && slot.off_key != key
        {
            slot.off_key = key;
            // The reach voices this put-away and pays the saber toggle cap for it. A
            // weapon switch copy already waiting paid first and is now the late one
            // (dropped below): its toggle is given back before this one is paid.
            let server_time = i32::try_from(self.holster_sync.time).unwrap_or(i32::MAX);
            let mut allowed = true;
            if let Some(adapter) = &mut self.legacy {
                for &(.., paid) in slot.pending_off.iter().flatten() {
                    adapter.refund_saber_toggle(client as u16, paid);
                }
                if handles[1].iter().any(Option::is_some) {
                    allowed = adapter.allow_saber_toggle(client as u16, server_time);
                }
            }
            for (layer, handle) in handles[1].into_iter().enumerate() {
                let Some(handle) = handle else {
                    continue;
                };
                if handles[1][..layer].contains(&Some(handle)) {
                    continue;
                }
                // Marked as sounded even past the cap, so its late copy stays dropped.
                slot.off_time = Some(now);
                if !allowed {
                    continue;
                }
                self.output.send(AudioCommand::Play(
                    handle,
                    PlayRequest {
                        origin: Some(origin),
                        source: SourceId(client as u32),
                        channel: ChannelId(0),
                        volume: 1.0,
                        attenuation: sjk_client::legacy_sound_attenuation(0),
                    },
                ));
            }
        }
        slot.was_active = active;
        for pending in &mut slot.pending_off {
            let Some((handle, request, paid)) = pending.take() else {
                continue;
            };
            if slot
                .off_time
                .is_some_and(|time| (0..HOLD).contains(&(now - time)))
            {
                slot.refund = Some(paid);
            } else {
                self.output.send(AudioCommand::Play(handle, request));
            }
        }
        pay_back(&mut self.legacy, client, slot);
        for (handle, request) in slot
            .release_ignitions(client, origin, phase, now)
            .into_iter()
            .flatten()
        {
            self.output.send(AudioCommand::Play(handle, request));
        }
    }

    pub(crate) fn end_holsters(&mut self) {
        for (client, slot) in self.holster_sync.slots.iter_mut().enumerate() {
            if !slot.seen {
                for (handle, request) in slot.take_deferred(client).into_iter().flatten() {
                    self.output.send(AudioCommand::Play(handle, request));
                }
                pay_back(&mut self.legacy, client, slot);
                *slot = Slot::default();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ignition_layers_wait_for_reach_and_have_a_bounded_fallback() {
        let request = PlayRequest {
            source: SourceId(3),
            origin: None,
            channel: ChannelId(0),
            volume: 1.,
            attenuation: sjk_audio::Attenuation::None,
        };
        for step in [8, 7, 4, 3] {
            let mut slot = Slot {
                pending: [
                    Some((SoundHandle(4), request, 0, false)),
                    Some((SoundHandle(6), request, 0, false)),
                    Some((SoundHandle(7), request, 0, false)),
                ],
                ..Slot::default()
            };
            for time in (0..600).step_by(step) {
                assert!(
                    slot.release_ignitions(3, [1., 2., 3.], Phase::OnBack, time)
                        .iter()
                        .all(Option::is_none)
                );
            }
            let starts = slot.release_ignitions(3, [1., 2., 3.], Phase::InHand, 600);
            assert_eq!(
                starts.map(|s| s.map(|s| s.0)),
                [
                    Some(SoundHandle(4)),
                    Some(SoundHandle(6)),
                    Some(SoundHandle(7))
                ]
            );
            assert_eq!(starts[0].unwrap().1.origin, Some([1., 2., 3.]));
            assert!(
                slot.release_ignitions(3, [0.; 3], Phase::InHand, 601)
                    .iter()
                    .all(Option::is_none)
            );
        }
        let mut slot = Slot {
            pending: [Some((SoundHandle(4), request, 0, false)), None, None],
            ..Slot::default()
        };
        assert!(slot.release_ignitions(3, [0.; 3], Phase::None, 149)[0].is_none());
        assert!(slot.release_ignitions(3, [0.; 3], Phase::None, 150)[0].is_some());
        slot.pending[0] = Some((SoundHandle(4), request, 0, true));
        assert!(slot.release_ignitions(3, [0.; 3], Phase::OnBack, 1999)[0].is_none());
        assert!(slot.release_ignitions(3, [0.; 3], Phase::OnBack, 2000)[0].is_some());
    }
    #[test]
    fn a_released_slot_plays_its_deferred_sounds_instead_of_dropping_them() {
        let request = PlayRequest {
            source: SourceId(40),
            origin: None,
            channel: ChannelId(0),
            volume: 1.,
            attenuation: sjk_audio::Attenuation::None,
        };
        let mut slot = Slot {
            origin: [1., 2., 3.],
            pending: [Some((SoundHandle(4), request, 0, true)), None, None],
            pending_off: [None, Some((SoundHandle(5), request, 0)), None],
            now: 5000,
            ..Slot::default()
        };
        let released = slot.take_deferred(3);
        let ignition = released[0].expect("held ignition");
        assert_eq!(ignition.0, SoundHandle(4));
        assert_eq!(ignition.1.origin, Some([1., 2., 3.]));
        assert_eq!(ignition.1.source, SourceId(3));
        assert_eq!(released[4].map(|s| s.0), Some(SoundHandle(5)));
        assert_eq!(released.iter().flatten().count(), 2);
        assert!(slot.take_deferred(3).iter().all(Option::is_none));
        // A shutdown the reach already played stays the late copy and is dropped.
        // It paid the saber toggle cap at 4100, which is given back.
        let mut slot = Slot {
            pending_off: [Some((SoundHandle(5), request, 4100)), None, None],
            off_time: Some(4000),
            now: 5000,
            ..Slot::default()
        };
        assert!(slot.take_deferred(3).iter().all(Option::is_none));
        assert_eq!(slot.refund, Some(4100));
    }
    #[test]
    fn ignition_interception_requires_ownership_handle_and_recent_draw() {
        let mut sync = Sync::default();
        sync.slots[3] = Slot {
            enabled: true,
            lit: true,
            sounds: [
                [Some(SoundHandle(4)), Some(SoundHandle(6)), None],
                [Some(SoundHandle(5)), None, None],
            ],
            unholster_time: Some(100),
            ..Slot::default()
        };
        let request = PlayRequest {
            source: SourceId(3),
            origin: None,
            channel: ChannelId(0),
            volume: 1.,
            attenuation: sjk_audio::Attenuation::None,
        };
        assert!(!sync.intercept(SoundHandle(8), request, 120));
        assert!(sync.intercept(SoundHandle(4), request, 120));
        assert!(!sync.intercept(SoundHandle(4), request, 300));
        assert!(!sync.intercept(
            SoundHandle(4),
            PlayRequest {
                source: SourceId(9),
                origin: Some([0.; 3]),
                ..request
            },
            120
        ));
        assert!(!sync.intercept(
            SoundHandle(4),
            PlayRequest {
                source: SourceId(9),
                ..request
            },
            120
        ));
        sync.slots[3].off_time = Some(100);
        assert!(sync.intercept(SoundHandle(5), request, 120));
        assert!(sync.slots[3].pending_off[0].is_none());
        // The late copy paid the saber toggle cap at its snapshot: given back.
        assert_eq!(sync.slots[3].refund.take(), Some(120));
        assert!(sync.intercept(SoundHandle(5), request, 2100));
        assert!(sync.slots[3].pending_off[0].is_some());
        assert_eq!(sync.slots[3].pending_off[0].map(|off| off.2), Some(2100));
        assert_eq!(sync.slots[3].refund, None);
        sync.enabled = false;
        assert!(!sync.intercept(SoundHandle(4), request, 120));
    }
}
