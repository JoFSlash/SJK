//! The card faces' pictures (`crate::cards::faces`): one texture each, for the faces a
//! screen draws. A face drawn without its texture asks for its picture
//! ([`CardTextures::touch`]); the worker decodes it and the renderer uploads it a couple
//! a frame. Past [`KEPT`] textures, those not drawn for [`IDLE_FRAMES`] are let go, to be
//! asked for again if drawn again. All are made again when the packs are mounted anew.
//! Every face uploaded at start held about 2 GB of video memory on a built-in Radeon
//! (339 faces at 5.3 MiB, `vram`, 25/10/2026).

use crate::cards::faces;

/// Pictures uploaded in one frame at most, so a new page of cards does not stall a frame.
const PER_FRAME: usize = 2;
/// Textures kept when not drawn: about 250 MiB at 5.3 MiB each. Faces on screen are never
/// let go, however many.
const KEPT: usize = 48;
/// Frames a face goes undrawn before its texture may be let go.
const IDLE_FRAMES: u64 = 300;

pub(super) struct CardTextures {
    /// The faces' generation the groups belong to.
    generation: u64,
    groups: Vec<Option<wgpu::BindGroup>>,
    /// The frame each face was last drawn in (0 never).
    drawn: Vec<u64>,
    frame: u64,
    sampler: Option<wgpu::Sampler>,
}

impl CardTextures {
    pub(super) fn new() -> Self {
        Self {
            generation: 0,
            groups: Vec::new(),
            drawn: Vec::new(),
            frame: 1,
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
        *drawn = self.frame;
        if self.groups[index].is_none() {
            faces::current().request(index);
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
            self.drawn = vec![0; faces.list.len()];
        }
        self.frame += 1;
        let mut uploaded = 0;
        for index in 0..self.groups.len() {
            if uploaded == PER_FRAME {
                break;
            }
            if self.groups[index].is_some() {
                continue;
            }
            let Some(chain) = faces.take_art(index) else {
                continue;
            };
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
        for index in released(&self.groups, &self.drawn, self.frame) {
            self.groups[index] = None;
        }
    }
}

/// The faces whose textures to let go: past [`KEPT`] uploaded, the ones drawn longest
/// ago among those undrawn for [`IDLE_FRAMES`].
fn released<T>(groups: &[Option<T>], drawn: &[u64], frame: u64) -> Vec<usize> {
    let mut idle: Vec<usize> = (0..groups.len())
        .filter(|&index| groups[index].is_some())
        .collect();
    let excess = idle.len().saturating_sub(KEPT);
    idle.retain(|&index| frame.saturating_sub(drawn[index]) > IDLE_FRAMES);
    idle.sort_by_key(|&index| drawn[index]);
    idle.truncate(excess);
    idle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_textures_go_past_the_kept_number_longest_undrawn_first() {
        let groups: Vec<Option<()>> = vec![Some(()); KEPT + 3];
        // All drawn long ago except face 0, drawn now; face 5 the longest ago.
        let mut drawn = vec![10_u64; KEPT + 3];
        drawn[0] = 1_000;
        drawn[5] = 1;
        drawn[7] = 2;
        let gone = released(&groups, &drawn, 1_000);
        assert_eq!(gone.len(), 3);
        assert_eq!(&gone[..2], &[5, 7]);
        assert!(!gone.contains(&0));
    }

    #[test]
    fn nothing_goes_within_the_kept_number_or_while_drawn() {
        let groups: Vec<Option<()>> = vec![Some(()); KEPT];
        assert!(released(&groups, &vec![1; KEPT], 1_000).is_empty());
        // More on screen than are kept: none of them goes.
        let groups: Vec<Option<()>> = vec![Some(()); KEPT + 10];
        assert!(released(&groups, &vec![995; KEPT + 10], 1_000).is_empty());
    }
}
