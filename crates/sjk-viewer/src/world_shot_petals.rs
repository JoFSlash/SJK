//! World shots of the blossoms a swung blade sheds ([`crate::saber_petals`]), without a
//! session: petals are queued as a swing would queue them, in front of the camera on
//! duel6, and the frame spawns and draws them; one shot as they leave the blade, one a
//! second and a half on, as they sink and settle. Ignored like the other world shots.

use super::*;

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn petals_drift_down_ahead_of_the_spawn() {
    on_big_stack(|| {
        let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", [1280, 720], None, &[]) else {
            return;
        };
        let (spawn, yaw) = assets::initial_camera(&gpu.bsp).expect("a spawn point");
        let turn = glam::Quat::from_rotation_z(yaw);
        let eye = Vec3::from_array(spawn);
        let ahead = eye + turn * Vec3::new(70.0, 0.0, 0.0);
        let (look_yaw, look_pitch) = look(eye.to_array(), ahead.to_array());
        aim(&mut gpu, eye.to_array(), look_yaw, look_pitch);
        let _ = frame(&mut gpu, 10);
        // A swing's worth along a blade sweeping across the view.
        let spec = crate::blade_signatures::of("saber_varykino_blossom")
            .and_then(|signature| signature.petals())
            .expect("the blossom's petals");
        let queue = gpu.saber_states.petals();
        for i in 0..40u32 {
            let along = i as f32 / 40.0;
            let across = turn * Vec3::new(0.0, (along - 0.5) * 60.0, (along * 7.0).sin() * 8.0);
            queue.push(crate::saber_petals::PetalSpawn {
                origin: ahead + across,
                velocity: turn * Vec3::new(0.0, 10.0, 0.0) - Vec3::Z * spec.fall * 0.5,
                seed: 0x5eed ^ i.wrapping_mul(2_654_435_761),
                spec,
            });
        }
        let before = gpu.particles.len();
        let first = frame(&mut gpu, 1);
        let spawned = gpu.particles.len() - before;
        println!("{spawned} petals spawned");
        assert!(spawned >= 40, "every queued petal is a particle");
        let later = frame(&mut gpu, 45);
        println!(
            "{}",
            sheet(&[first, later], 2, 1280, "petals_duel6").display()
        );
    });
}
