//! The card faces' pictures (`crate::cards::faces`): one texture each, for the faces a
//! screen draws. A face drawn without one is asked for; its picture, decoded on a worker,
//! is uploaded a couple a frame. At most [`KEPT`] stay on the card, the least recently
//! drawn going first, and one not drawn for [`UNUSED`] is dropped, so a screen of cards
//! costs what it shows rather than the whole catalogue. All are made again when the packs
//! are mounted anew.

use crate::cards::faces;
use std::time::{Duration, Instant};

/// Pictures uploaded in one frame at most, so a screen of new cards does not stall a frame.
const PER_FRAME: usize = 2;
/// Pictures kept on the card at most (64 of 1024 pixels, about 340 MiB; of 512, 85 MiB),
/// more only while a screen draws more at once.
const KEPT: usize = 64;
/// A picture not drawn for this long is dropped.
const UNUSED: Duration = Duration::from_secs(60);

pub(super) struct CardTextures {
    /// The faces' generation the groups belong to.
    generation: u64,
    /// Each face's uploaded picture, with when a frame last drew it.
    groups: Vec<Option<(wgpu::BindGroup, Instant)>>,
    /// Faces drawn without a picture since the last [`Self::install`].
    wanted: Vec<usize>,
    /// Whether the last install asked for pictures, so the next one says when none are
    /// wanted any more.
    asked: bool,
    /// The time of the last install, which the frame's draws are stamped with.
    now: Instant,
    /// The time of the install before: the stamp of the faces the last frame drew.
    previous: Instant,
    sampler: Option<wgpu::Sampler>,
}

impl CardTextures {
    pub(super) fn new() -> Self {
        Self {
            generation: 0,
            groups: Vec::new(),
            wanted: Vec::new(),
            asked: false,
            now: Instant::now(),
            previous: Instant::now(),
            sampler: None,
        }
    }

    /// The bind group sampling face `index`'s picture, once uploaded.
    pub(super) fn group(&self, index: usize) -> Option<&wgpu::BindGroup> {
        self.groups.get(index)?.as_ref().map(|(group, _)| group)
    }

    /// Whether face `index`'s picture can be drawn this frame. It is stamped as drawn, or
    /// asked for at the next install when it is not uploaded.
    pub(super) fn draw(&mut self, index: usize) -> bool {
        if let Some(Some((_, drawn))) = self.groups.get_mut(index) {
            *drawn = self.now;
            return true;
        }
        if !self.wanted.contains(&index) {
            self.wanted.push(index);
        }
        false
    }

    /// Upload the pictures decoded for the faces last drawn without one (a few a frame),
    /// ask for the rest, and drop pictures left undrawn; forget them all when the faces
    /// changed.
    pub(super) fn install(
        &mut self,
        device: &wgpu::Device,
        queue: &crate::frame_queue::FrameQueue,
        layout: &wgpu::BindGroupLayout,
    ) {
        let faces = faces::current();
        if faces.generation != self.generation || self.groups.len() != faces.list.len() {
            self.generation = faces.generation;
            self.groups = (0..faces.list.len()).map(|_| None).collect();
            self.wanted.clear();
        }
        // A card's scene just drawn (`cards::scene`): its picture is uploaded again, now
        // from the drawing.
        for id in crate::cards::scene::take_drawn() {
            if let Some((index, _)) = faces.find(&id) {
                faces.retry(index);
                if let Some(slot) = self.groups.get_mut(index) {
                    *slot = None;
                }
            }
        }
        self.previous = self.now;
        self.now = Instant::now();
        let now = self.now;
        for slot in &mut self.groups {
            if slot
                .as_ref()
                .is_some_and(|(_, drawn)| now.duration_since(*drawn) > UNUSED)
            {
                *slot = None;
            }
        }
        let wanted = std::mem::take(&mut self.wanted);
        if wanted.is_empty() && !self.asked {
            return;
        }
        faces::request(&faces, &wanted);
        self.asked = !wanted.is_empty();
        #[cfg(test)]
        let per_frame = if faces::inline_for_shots() {
            usize::MAX
        } else {
            PER_FRAME
        };
        #[cfg(not(test))]
        let per_frame = PER_FRAME;
        let mut uploaded = 0;
        for index in wanted {
            if uploaded == per_frame {
                break;
            }
            let Some(chain) = faces.take_art(index) else {
                continue;
            };
            self.make_room();
            let sampler = self.sampler.get_or_insert_with(|| {
                device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("SJK card art sampler"),
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::MipmapFilterMode::Linear,
                    ..Default::default()
                })
            });
            let view = super::emblem::upload_labelled(device, queue, &chain, "SJK card art");
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("SJK card art bind group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            });
            if let Some(slot) = self.groups.get_mut(index) {
                *slot = Some((group, now));
            }
            uploaded += 1;
        }
    }

    /// At [`KEPT`] pictures, drop the least recently drawn one that the last frame did not
    /// draw; none when every kept picture is on screen.
    fn make_room(&mut self) {
        if self.groups.iter().flatten().count() < KEPT {
            return;
        }
        let oldest = self
            .groups
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| Some((index, slot.as_ref()?.1)))
            .filter(|(_, drawn)| *drawn < self.previous)
            .min_by_key(|(_, drawn)| *drawn);
        if let Some((index, _)) = oldest {
            self.groups[index] = None;
        }
    }
}
