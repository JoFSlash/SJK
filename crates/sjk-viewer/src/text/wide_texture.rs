//! The GPU copies of the wide-glyph atlas ([`super::wide`]) and the emoji atlas
//! ([`super::emoji`]), and the text bind groups that carry them.
//!
//! Every text bind group holds its font's own atlas at binding 0, the sampler at 1,
//! the wide atlas at 2 and the emoji atlas at 3; `text.wgsl` samples the wide atlas for
//! vertices whose `u` is [`super::wide::U_OFFSET`] or more and the emoji atlas for
//! those at [`super::emoji::U_OFFSET`] or more.
//!
//! Each atlas is a 1x1 stand-in until the first glyph or emoji is written into it: the
//! full textures, about 10.7 MiB each, are made then, and every text bind group is made
//! again with them ([`crate::GpuState::update_wide_text`]). A world in which no one
//! writes beyond the code page or sends an emoji never holds them.

use super::wide::Cell;
use super::{emoji, wide};
use crate::frame_queue::FrameQueue;

/// The wide atlas (R8 coverage) and the emoji atlas (RGBA display values) on the GPU,
/// with the code-page atlases' mip levels.
pub(crate) struct WideTexture {
    wide: Atlas,
    emoji: Atlas,
    /// Reused list of cells to copy this frame.
    cells: Vec<Cell>,
}

/// One atlas texture: the stand-in, or the full atlas once made.
struct Atlas {
    label: &'static str,
    size: [u32; 2],
    format: wgpu::TextureFormat,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    made: bool,
    /// The card had no room for it: the stand-in stays and it is not tried again.
    refused: bool,
}

impl Atlas {
    fn new(
        device: &wgpu::Device,
        label: &'static str,
        size: [u32; 2],
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = atlas_texture(device, "SJK text atlas stand-in", [1, 1], 1, format);
        Self {
            label,
            size,
            format,
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            texture,
            made: false,
            refused: false,
        }
    }

    /// Make the full atlas if it is still the stand-in; whether it is made (not when the
    /// card has no room for it, logged once: its characters then draw nothing).
    fn make(&mut self, device: &wgpu::Device) -> bool {
        if !self.made && !self.refused {
            let (label, size, format) = (self.label, self.size, self.format);
            match crate::gpu_guard::capture(device, || {
                atlas_texture(device, label, size, super::ATLAS_MIP_LEVELS, format)
            }) {
                Ok(texture) => {
                    self.view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                    self.texture = texture;
                    self.made = true;
                }
                Err(error) => {
                    self.refused = true;
                    crate::log::progress(format_args!("warning: no {label} ({error})"));
                }
            }
        }
        self.made
    }
}

fn atlas_texture(
    device: &wgpu::Device,
    label: &str,
    size: [u32; 2],
    mip_level_count: u32,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

impl WideTexture {
    /// The stand-ins. The CPU atlases start over with them, so every wide glyph and
    /// emoji on screen is drawn into the atlases made for them over the next frames.
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let wide = Atlas::new(
            device,
            "SJK wide glyph atlas",
            [wide::ATLAS_WIDTH, wide::ATLAS_HEIGHT],
            wgpu::TextureFormat::R8Unorm,
        );
        // Display values, as every picture the 2D layer samples (`ui_target`).
        let emoji = Atlas::new(
            device,
            "SJK emoji atlas",
            [emoji::ATLAS_WIDTH, emoji::ATLAS_HEIGHT],
            crate::ui_target::TEXTURE_FORMAT,
        );
        wide::start_over();
        emoji::start_over();
        Self {
            wide,
            emoji,
            cells: Vec::new(),
        }
    }

    /// Rasterize the glyphs and decode the emoji this frame asked for (within
    /// [`wide::FRAME_BUDGET`] for both) and copy every new cell, with its mip levels,
    /// into the textures ahead of the next submission, making an atlas for its first
    /// cell. Nothing is copied or allocated on a frame that asked for nothing new.
    /// Whether an atlas was made: the text bind groups must then be made again.
    pub(crate) fn update(&mut self, device: &wgpu::Device, queue: &FrameQueue) -> bool {
        let deadline = std::time::Instant::now() + wide::FRAME_BUDGET;
        wide::rasterize_pending(wide::FRAME_BUDGET);
        emoji::rasterize_pending(deadline);
        let made = (self.wide.made, self.emoji.made);
        wide::take_dirty(&mut self.cells);
        if !self.cells.is_empty() && self.wide.make(device) {
            let cells = &self.cells;
            wide::mirror(|pixels| {
                upload(queue, &self.wide.texture, cells, 1, |cell, level| {
                    wide::cell_level(pixels, cell, level)
                });
            });
        }
        self.cells.clear();
        emoji::take_dirty(&mut self.cells);
        if !self.cells.is_empty() && self.emoji.make(device) {
            let cells = &self.cells;
            emoji::mirror(|pixels| {
                upload(queue, &self.emoji.texture, cells, 4, |cell, level| {
                    emoji::cell_level(pixels, cell, level)
                });
            });
        }
        self.cells.clear();
        made != (self.wide.made, self.emoji.made)
    }
}

impl crate::GpuState {
    /// Bring the wide and emoji atlases up to date ([`WideTexture::update`]); when that
    /// made one, every text bind group is made again to carry it.
    pub(crate) fn update_wide_text(&mut self) {
        if !self.wide_text.update(&self.device, &self.queue) {
            return;
        }
        self.text_bind_group = bind_group(
            &self.device,
            "SJK text bind group",
            &self.text_layout,
            &self.text_atlas,
            &self.text_sampler,
            &self.wide_text,
        );
        if let Some(atlas) = &self.classic_text_atlas {
            self.classic_text_bind_group = Some(bind_group(
                &self.device,
                "SJK classic HUD text bind group",
                &self.text_layout,
                atlas,
                &self.text_sampler,
                &self.wide_text,
            ));
        }
        self.game_fonts.rebind(&crate::game_font::Device {
            device: &self.device,
            queue: &self.queue,
            layout: &self.text_layout,
            sampler: &self.text_sampler,
            wide: &self.wide_text,
        });
    }
}

/// Copy `cells` of `texture` (`texel` bytes a texel) at every mip level, each level's
/// texels from `level_of`.
fn upload(
    queue: &FrameQueue,
    texture: &wgpu::Texture,
    cells: &[Cell],
    texel: u32,
    level_of: impl Fn(Cell, u32) -> Vec<u8>,
) {
    for &cell in cells {
        for level in 0..super::ATLAS_MIP_LEVELS {
            let data = level_of(cell, level);
            let (width, height) = (cell.width >> level, cell.height >> level);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: level,
                    origin: wgpu::Origin3d {
                        x: cell.x >> level,
                        y: cell.y >> level,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * texel),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
}

/// A text bind group: `atlas` (the font's own), the text sampler, and the wide and
/// emoji atlases of `shared`.
pub(crate) fn bind_group(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    atlas: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    shared: &WideTexture,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(atlas),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&shared.wide.view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&shared.emoji.view),
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_text_program_validates_and_shares_the_offsets() {
        let source = include_str!("../text.wgsl");
        crate::wgsl_source::validate(source);
        for declared in [
            format!("const WIDE_U_OFFSET: f32 = {:.1};", super::wide::U_OFFSET),
            format!("const EMOJI_U_OFFSET: f32 = {:.1};", super::emoji::U_OFFSET),
        ] {
            assert!(source.contains(&declared), "{declared}");
        }
    }

    /// The code-page atlases are R8 (`gpu_texture::create_r8_texture_mipmapped`): the
    /// programs sampling one read its coverage or distance from red, where alpha is 1.
    #[test]
    fn the_font_atlases_are_read_from_red() {
        let text = include_str!("../text.wgsl");
        let sample = "textureSample(font_texture, font_sampler, input.uv)";
        assert_eq!(text.matches(&format!("{sample}.r;")).count(), 2);
        assert_eq!(text.matches(sample).count(), 2);
        let ground = include_str!("../ground_hud.wgsl");
        crate::wgsl_source::validate(ground);
        assert!(ground.contains("q.bounds.zw), dx, dy).r;"));
    }

    /// The game chat in other alphabets and emoji as they arrive: Windows-1251 from an
    /// EternalJK player, UTF-8 from SJK players (Russian, Japanese, Chinese, Korean,
    /// emoji with skin tones, a family and flags), and a draft typed in them, in Inter
    /// and in the game fonts (SJK Chat), at 1080p and at 4K.
    #[test]
    #[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
    fn chat_in_russian_and_japanese() {
        for (size, game_font, suffix) in [
            ([1920, 1080], "0", "1080p"),
            ([3840, 2160], "0", "4k"),
            ([1920, 1080], "1", "1080p-game-font"),
        ] {
            crate::world_shot::on_big_stack(move || {
                let Some((mut gpu, _profile)) = crate::world_shot::open(
                    "maps/mp/duel6.bsp",
                    size,
                    None,
                    &[("cg_chatbox", "60000"), (crate::game_font::CVAR, game_font)],
                ) else {
                    return;
                };
                gpu.chat.for_shot = true;
                let lines: [&[u8]; 7] = [
                    // An EternalJK player with `se_language russian`: Windows-1251.
                    b"^1Vasya^7: \xef\xf0\xe8\xe2\xe5\xf2, \xea\xf2\xee \xed\xe0 \xe4\xf3\xfd\xeb\xfc?",
                    "^5Sol^7: Да, иду! Ёлки-палки, ПРИВЕТ всем".as_bytes(),
                    "^3Kenji^7: こんにちは、日本語のチャットです。漢字もカタカナも OK".as_bytes(),
                    "^2Wei^7: 你好，我们这里说中文。繁體字也可以".as_bytes(),
                    "^4Minji^7: 안녕하세요! 한국어 채팅 ㅋㅋㅋ".as_bytes(),
                    "^6Emoji^7: gg 😀🔥👍🏽 ❤\u{FE0F} 👨\u{200D}👩\u{200D}👧 🇫🇷🇯🇵 1\u{FE0F}\u{20E3}".as_bytes(),
                    "^2Mixed^7: café ½ €5 © ™ привет 日本 한국".as_bytes(),
                ];
                for line in lines {
                    gpu.chat.receive(
                        sjk_client::ServerEventKind::Chat,
                        sjk_client::decode_legacy(line).into_owned(),
                        None,
                        std::time::Instant::now(),
                    );
                }
                gpu.chat.draft_for_shot("пишу 日本語 中文 한국어 🎉");
                let path =
                    crate::world_shot::shoot(&mut gpu, 10, &format!("chat-unicode-{suffix}"));
                println!("{}", path.display());
            });
        }
    }
}
