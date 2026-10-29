//! A world shot of the mystery boxes ([`crate::mystery_box`]) on duel6: four boxes,
//! each just stopped on one tier's holocron (Uncommon to Mythical), every part of its form
//! drawn; and four on the prizes of 29/10/2026 (two piles of shards, a free box and the
//! boxes' blade). Ignored like the other world shots.

use super::*;

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn duel6_mystery_box_holocrons() {
    on_big_stack(|| {
        let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", [1600, 900], None, &[]) else {
            return;
        };
        let shots = crate::menu_backdrop::tour_for("Yavin Training Grounds").expect("duel6's tour");
        let (from, at) = (
            Vec3::from_array(shots[0].from),
            Vec3::from_array(shots[0].at),
        );
        // Four boxes in a row across the view, 150 units ahead.
        let ahead = (at - from).with_z(0.0).normalize_or(Vec3::X);
        let across = Vec3::new(-ahead.y, ahead.x, 0.0);
        let floor = from + ahead * 150.0 - Vec3::Z * 60.0;
        let yaw = ahead.y.atan2(ahead.x).to_degrees() + 180.0;
        let boxes: Vec<([f32; 3], f32)> = (0..4)
            .map(|index| {
                (
                    (floor + across * (index as f32 - 1.5) * 60.0).to_array(),
                    yaw,
                )
            })
            .collect();
        gpu.shot_mystery_boxes(
            &boxes,
            crate::mystery_box::SHOT_ROLL + std::time::Duration::from_millis(900),
        );
        let (yaw, pitch) = look(from.to_array(), (floor + Vec3::Z * 40.0).to_array());
        aim(&mut gpu, from.to_array(), yaw, pitch);
        gpu.is_menu_world = false;
        println!(
            "{}",
            shoot(&mut gpu, 90, "duel6-mystery-box-holocrons").display()
        );
    });
}

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn duel6_mystery_box_loot() {
    on_big_stack(|| {
        let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", [1600, 900], None, &[]) else {
            return;
        };
        let shots = crate::menu_backdrop::tour_for("Yavin Training Grounds").expect("duel6's tour");
        let (from, at) = (
            Vec3::from_array(shots[0].from),
            Vec3::from_array(shots[0].at),
        );
        // Four boxes in a row across the view, 150 units ahead.
        let ahead = (at - from).with_z(0.0).normalize_or(Vec3::X);
        let across = Vec3::new(-ahead.y, ahead.x, 0.0);
        let floor = from + ahead * 150.0 - Vec3::Z * 60.0;
        let yaw = ahead.y.atan2(ahead.x).to_degrees() + 180.0;
        let boxes: Vec<([f32; 3], f32)> = (0..4)
            .map(|index| {
                (
                    (floor + across * (index as f32 - 1.5) * 60.0).to_array(),
                    yaw,
                )
            })
            .collect();
        gpu.shot_mystery_box_loot(
            &boxes,
            crate::mystery_box::SHOT_ROLL + std::time::Duration::from_millis(900),
        );
        let (yaw, pitch) = look(from.to_array(), (floor + Vec3::Z * 40.0).to_array());
        aim(&mut gpu, from.to_array(), yaw, pitch);
        gpu.is_menu_world = false;
        println!(
            "{}",
            shoot(&mut gpu, 90, "duel6-mystery-box-loot").display()
        );
    });
}
