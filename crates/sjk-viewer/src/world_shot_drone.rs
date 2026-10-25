//! World shots of the Drone ([`crate::drone`]) on a recorded game: the demo
//! `SJK_TEST_DEMO` names is played without a server; a camera orbiting another player
//! sees a drone flown just ahead of it, then the same drone peeking at that player, with
//! its pilot's name over it. Ignored like the other world shots; skipped when
//! `SJK_TEST_DEMO` is not set.

use super::*;

#[test]
#[ignore = "renders with the GPU, the installed game data and the demo SJK_TEST_DEMO names"]
fn drone_on_a_demo() {
    on_big_stack(|| {
        let Some(demo) = std::env::var_os("SJK_TEST_DEMO").map(PathBuf::from) else {
            println!("SJK_TEST_DEMO is not set: skipped");
            return;
        };
        let probe = demo_playback::Session::open(&demo, demo_playback::Camera::FirstPerson, 60)
            .expect("the demo");
        let own = probe.latest_snapshot().player.client_num();
        let Some(target) = probe
            .latest_snapshot()
            .entities
            .iter()
            .map(|entity| entity.number())
            .find(|number| *number < 32 && *number != own)
        else {
            println!("no other player in the demo's first snapshot: skipped");
            return;
        };
        let camera = demo_playback::Camera::Orbit(target);
        let Some(mut gpu) = card_outline_shots::open_demo(&demo, camera, [1600, 900]) else {
            return;
        };
        let _ = frame(&mut gpu, 20);
        let mut shots = Vec::with_capacity(2);
        // The demo's own player flies a drone just ahead of the camera, a little aside.
        let pilot = u8::try_from(own).unwrap_or(0);
        let (yaw, pitch) = (gpu.camera_yaw, gpu.camera_pitch);
        let forward = glam::Vec3::new(
            yaw.cos() * pitch.cos(),
            yaw.sin() * pitch.cos(),
            pitch.sin(),
        );
        let side = glam::Vec3::new(-yaw.sin(), yaw.cos(), 0.0);
        let at = gpu.camera_position + forward * 70.0 + side * 22.0;
        gpu.shot_drone(
            pilot,
            Some(at.to_array()),
            (yaw.to_degrees() as i32 + 180).rem_euclid(360),
            0,
        );
        shots.push(frame(&mut gpu, 30));
        // Then peeks at the player instead: the drone behind them.
        gpu.shot_drone(pilot, None, 0, u8::try_from(target).unwrap_or(0));
        shots.push(frame(&mut gpu, 30));
        println!(
            "player {target}: {}",
            sheet(&shots, 2, 800, "drone").display()
        );
    });
}
