//! The card faces' pictures (`crate::cards::faces`): one texture each, for the faces a
//! screen draws. A face drawn without its texture asks for its picture
//! ([`CardTextures::touch`]), once; the worker decodes it and the renderer uploads it, a
//! couple a frame. Past [`KEPT`] textures, those not drawn for [`IDLE`] are let go, to be
//! asked for again if drawn again. All are made again when the packs are mounted anew.
//! Every face uploaded at start held about 2 GB of video memory on a built-in Radeon
//! (339 faces at 5.3 MiB, `vram`, 25/10/2026).

use crate::cards::faces;

/// Pictures uploaded in one frame at most, so a new page of cards does not stall a frame.
const PER_FRAME: usize = 2;
/// Textures kept when not drawn: about 250 MiB at 5.3 MiB each. Faces on screen are never
/// let go, however many.
const KEPT: usize = 48;
/// How long a face goes undrawn before its texture may be let go: time, not frames, so
/// a fast menu does not let a page go as soon as it is left.
const IDLE: std::time::Duration = std::time::Duration::from_secs(5);

pub(super) struct CardTextures {
    /// The faces' generation the groups belong to.
    generation: u64,
    /// The faces of that generation, for the frame's requests.
    faces: std::sync::Arc<faces::Faces>,
    groups: Vec<Option<wgpu::BindGroup>>,
    /// When each face was last drawn.
    drawn: Vec<Option<std::time::Instant>>,
    /// Faces whose picture was asked for and has not come yet (or could not be read).
    asked: Vec<bool>,
    /// This frame's time.
    now: std::time::Instant,
    sampler: Option<wgpu::Sampler>,
}

impl CardTextures {
    pub(super) fn new() -> Self {
        Self {
            generation: 0,
            faces: faces::current(),
            groups: Vec::new(),
            drawn: Vec::new(),
            asked: Vec::new(),
            now: std::time::Instant::now(),
            sampler: None,
        }
    }

    /// The bind group sampling face `index`'s picture, once uploaded.
    pub(super) fn group(&self, index: usize) -> Option<&wgpu::BindGroup> {
        self.groups.get(index)?.as_ref()
    }

    /// Face `index` is drawn this frame: keep its texture, or ask for its picture.
    pub(super) fn touch(&mut self, index: usize) {
        let Some(drawn) = self.drawn.get_mut(index) else {
            return;
        };
        *drawn = Some(self.now);
        if self.groups[index].is_none() && !self.asked[index] {
            self.asked[index] = true;
            self.faces.request(index);
        }
    }

    /// Upload the pictures asked for and decoded since the last call (a few a frame), let
    /// long-undrawn ones go past [`KEPT`]; forget them all when the faces changed.
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
            self.drawn = vec![None; faces.list.len()];
            self.asked = vec![false; faces.list.len()];
        }
        self.faces = faces;
        self.now = std::time::Instant::now();
        let mut uploaded = 0;
        while uploaded < PER_FRAME {
            let Some((index, chain)) = self.faces.take_ready() else {
                break;
            };
            if index >= self.groups.len() || self.groups[index].is_some() {
                continue;
            }
            self.asked[index] = false;
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
            self.groups[index] = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            }));
            uploaded += 1;
        }
        for index in released(&self.groups, &self.drawn, self.now) {
            self.groups[index] = None;
        }
    }
}

/// The faces whose textures to let go: past [`KEPT`] uploaded, the ones drawn longest
/// ago among those undrawn for [`IDLE`].
fn released<T>(
    groups: &[Option<T>],
    drawn: &[Option<std::time::Instant>],
    now: std::time::Instant,
) -> Vec<usize> {
    let uploaded = groups.iter().filter(|group| group.is_some()).count();
    if uploaded <= KEPT {
        return Vec::new();
    }
    let mut idle: Vec<usize> = (0..groups.len())
        .filter(|&index| groups[index].is_some())
        .filter(|&index| drawn[index].is_none_or(|at| now.duration_since(at) > IDLE))
        .collect();
    idle.sort_by_key(|&index| drawn[index]);
    idle.truncate(uploaded - KEPT);
    idle
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn idle_textures_go_past_the_kept_number_longest_undrawn_first() {
        let now = Instant::now() + Duration::from_secs(100);
        let groups: Vec<Option<()>> = vec![Some(()); KEPT + 3];
        // All drawn long ago except face 0, drawn now; face 5 the longest ago.
        let mut drawn = vec![Some(now - Duration::from_secs(60)); KEPT + 3];
        drawn[0] = Some(now);
        drawn[5] = Some(now - Duration::from_secs(90));
        drawn[7] = Some(now - Duration::from_secs(80));
        let gone = released(&groups, &drawn, now);
        assert_eq!(gone.len(), 3);
        assert_eq!(&gone[..2], &[5, 7]);
        assert!(!gone.contains(&0));
    }

    #[test]
    fn nothing_goes_within_the_kept_number_or_while_drawn() {
        let now = Instant::now() + Duration::from_secs(100);
        let long_ago = Some(now - Duration::from_secs(60));
        let groups: Vec<Option<()>> = vec![Some(()); KEPT];
        assert!(released(&groups, &vec![long_ago; KEPT], now).is_empty());
        // More on screen than are kept: none of them goes.
        let groups: Vec<Option<()>> = vec![Some(()); KEPT + 10];
        let recent = Some(now - Duration::from_secs(1));
        assert!(released(&groups, &vec![recent; KEPT + 10], now).is_empty());
    }
}
