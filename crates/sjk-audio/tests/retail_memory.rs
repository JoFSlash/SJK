//! What the retail sounds cost in memory once decoded, the way the client
//! decodes them (mono, 44.1 kHz): 16-bit PCM now, against the 32-bit floats it
//! replaced. Reads the installed game named by `JKA_GAME_DATA` (its `GameData`
//! folder); nothing is written or kept. Run in release:
//! `cargo test --release -p sjk-audio --test retail_memory -- --ignored --nocapture`

use sjk_audio::{SoundBank, decode_encoded};
use sjk_vfs::VirtualFileSystem;

const MIB: f64 = 1024.0 * 1024.0;
/// The client's mixer rate (`audio_output.rs`).
const SAMPLE_RATE: u32 = 44_100;

#[derive(Default)]
struct Tally {
    files: usize,
    encoded: usize,
    pcm16: usize,
    largest: (usize, String),
    failed: usize,
}

impl Tally {
    fn add(&mut self, path: &str, encoded: usize, pcm16: Option<usize>) {
        let Some(pcm16) = pcm16 else {
            self.failed += 1;
            return;
        };
        self.files += 1;
        self.encoded += encoded;
        self.pcm16 += pcm16;
        if pcm16 > self.largest.0 {
            self.largest = (pcm16, path.to_owned());
        }
    }

    fn print(&self, name: &str) {
        println!(
            "{name}: {} files ({} undecodable), {:.1} MiB of files -> {:.1} MiB as floats, \
             {:.1} MiB as 16-bit; largest {} at {:.1} -> {:.1} MiB",
            self.files,
            self.failed,
            self.encoded as f64 / MIB,
            (self.pcm16 * 2) as f64 / MIB,
            self.pcm16 as f64 / MIB,
            self.largest.1,
            (self.largest.0 * 2) as f64 / MIB,
            self.largest.0 as f64 / MIB,
        );
    }
}

#[test]
#[ignore = "reads the installed game data named by JKA_GAME_DATA"]
fn retail_sounds_take_half_the_memory_as_16_bit_pcm() {
    let Ok(data) = std::env::var("JKA_GAME_DATA") else {
        return;
    };
    let mut vfs = VirtualFileSystem::new();
    for index in 0..4 {
        let archive = std::path::Path::new(&data).join(format!("base/assets{index}.pk3"));
        vfs.mount_pk3(&archive).unwrap();
    }
    // Every retail sound, though a session decodes only the ones it plays: the
    // WAVs are the effects, the MP3s under sound/ mostly single-player dialogue.
    let (mut wavs, mut mp3s, mut music) = (Tally::default(), Tally::default(), Tally::default());
    // Sounds stay registered for the whole session, as in the client.
    let mut bank = SoundBank::new(SAMPLE_RATE);
    for path in vfs.paths() {
        let path = path.as_str();
        let Some(extension) = path.rsplit_once('.').map(|(_, extension)| extension) else {
            continue;
        };
        let is_music = path.starts_with("music/");
        if !(is_music || path.starts_with("sound/"))
            || !(extension.eq_ignore_ascii_case("wav") || extension.eq_ignore_ascii_case("mp3"))
        {
            continue;
        }
        let bytes = vfs.read(path).unwrap().unwrap().bytes;
        let decoded = decode_encoded(&bytes, extension, SAMPLE_RATE).ok();
        let pcm16 = decoded.as_ref().map(sjk_audio::DecodedSound::byte_len);
        if is_music {
            music.add(path, bytes.len(), pcm16);
            continue;
        }
        if extension.eq_ignore_ascii_case("wav") {
            wavs.add(path, bytes.len(), pcm16);
        } else {
            mp3s.add(path, bytes.len(), pcm16);
        }
        if let Some(decoded) = decoded {
            bank.register_decoded(decoded);
        }
    }
    wavs.print("sound/*.wav");
    mp3s.print("sound/*.mp3");
    music.print("music/");
    println!(
        "sound/ in one bank: {:.1} MiB held ({:.1} MiB before, as floats); music, one \
         track on average: {:.1} MiB ({:.1} MiB before)",
        bank.sample_bytes() as f64 / MIB,
        (bank.sample_bytes() * 2) as f64 / MIB,
        music.pcm16 as f64 / music.files.max(1) as f64 / MIB,
        (music.pcm16 * 2) as f64 / music.files.max(1) as f64 / MIB,
    );
    assert_eq!(bank.sample_bytes(), wavs.pcm16 + mp3s.pcm16);
}
