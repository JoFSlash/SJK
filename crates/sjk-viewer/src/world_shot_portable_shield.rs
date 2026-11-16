//! World shots of the portable shield ([`crate::portable_shields`], hub report #133): a
//! blue and a red field on the floor ahead of duel6's first spawn, without a session
//! (the shot adds the particles a standing field adds each cgame frame, six of them, as
//! at the reference 125 FPS, and keeps them alive). Ignored like the other world shots.

use super::*;

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn duel6_portable_shields() {
    on_big_stack(|| {
        let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", [1280, 720], None, &[]) else {
            return;
        };
        gpu.is_menu_world = false;
        let (spawn, yaw) = assets::initial_camera(&gpu.bsp).expect("a spawn point");
        let turn = glam::Quat::from_rotation_z(yaw);
        let eye = Vec3::from_array(spawn);
        let floor = |gpu: &mut GpuState, point: Vec3| {
            let trace = gpu.bsp.trace_box_with(
                &mut gpu.trace_scratch,
                point.to_array(),
                (point - Vec3::Z * 512.0).to_array(),
                sjk_bsp::Aabb::new([0.0; 3], [0.0; 3]).expect("point bounds are valid"),
                1,
            );
            Vec3::from_array(trace.end_position)
        };
        let fields = [
            (
                eye + turn * Vec3::new(200.0, -60.0, 0.0),
                "gfx/misc/blue_portashield",
            ),
            (
                eye + turn * Vec3::new(260.0, 90.0, 0.0),
                "gfx/misc/red_portashield",
            ),
        ];
        for (point, shader) in fields {
            let base = floor(&mut gpu, point) + Vec3::Z * 1.0;
            // `PlaceShield`'s default: 64 tall, 48 either side, across the view.
            let side = turn * Vec3::Y;
            let field = crate::portable_shields::Field {
                start: base - side * 48.0 + Vec3::Z * 32.0,
                end: base + side * 48.0 + Vec3::Z * 32.0,
                height: 64.0,
                shader,
            };
            for _ in 0..6 {
                let mut particle = crate::portable_shields::shot_particle(
                    field,
                    &mut gpu.effects,
                    std::time::Instant::now(),
                );
                particle.lifetime = std::time::Duration::from_secs(600);
                gpu.particles.push(particle);
            }
        }
        let views = [
            (Vec3::new(0.0, 0.0, 40.0), Vec3::new(230.0, 0.0, 30.0)),
            (Vec3::new(120.0, -200.0, 50.0), Vec3::new(230.0, 0.0, 30.0)),
        ];
        let mut images = Vec::new();
        for (from, at) in views {
            let camera = eye + turn * from;
            let target = eye + turn * at;
            let (look_yaw, look_pitch) = look(camera.to_array(), target.to_array());
            aim(&mut gpu, camera.to_array(), look_yaw, look_pitch);
            images.push(frame(&mut gpu, 20));
            println!(
                "particles {}, field layers {:?}",
                gpu.particles.len(),
                gpu.particle_atlas
                    .layers_for("gfx/misc/blue_portashield", 0.0)
                    .iter()
                    .map(|layer| (layer.blend, layer.uv_rect))
                    .collect::<Vec<_>>()
            );
        }
        println!(
            "{}",
            sheet(&images, 2, 960, "duel6-portable-shields").display()
        );
    });
}
