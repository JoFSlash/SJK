//! Submit cached stowed hilts and choose the staff's single visible attachment.
use super::*;
use crate::holstered_sabers::{Body, staff::Phase};

#[derive(Default, Clone, Copy)]
pub(super) struct Plan {
    pub(super) on_back: bool,
    pub(super) blades_in: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn submit(
    sinks: &mut Sinks<'_>,
    index: usize,
    entity: &sjk_runtime::SceneEntity,
    transform: sjk_runtime::Transform,
    snapshot: Option<&Snapshot>,
    state: Option<&sjk_protocol::EntityState>,
    equipment: Option<sjk_runtime::HeldEquipment>,
    local: bool,
    draw_actor: bool,
    hidden: bool,
) -> Plan {
    let mesh = &sinks.actor_meshes[index];
    // Off (setting, plugin bit or a server without the contract): no per-actor work.
    // The audio slot, no longer reported, is released by `end_holsters`.
    if !sinks.holsters.enabled {
        mesh.holsters.hide();
        mesh.holsters.swap.reset();
        return Plan::default();
    }
    let player = snapshot.map(|s| &s.player);
    let predicted = sinks.predicted_local_state.filter(|_| local);
    let in_flight = if local {
        predicted
            .map(|p| p.saber_in_flight)
            .or_else(|| player.map(|p| p.saber_in_flight()))
            .unwrap_or(false)
    } else {
        state.is_some_and(|s| s.saber_in_flight())
    };
    let flags = if local {
        predicted
            .map(|p| p.entity_flags)
            .or_else(|| player.map(|p| p.entity_flags()))
            .unwrap_or(0)
    } else {
        state.map_or(0, |s| s.e_flags())
    };
    let viewed_player =
        local || player.is_some_and(|p| u64::from(p.client_num()) + 1 == entity.id.get());
    let body = Body {
        owned: if local {
            predicted
                .map(|p| p.weapons)
                .or_else(|| player.map(|p| p.stats[4]))
                .unwrap_or(0)
                & (1 << 3)
                != 0
        } else {
            mesh.saber_names[0].is_some() || mesh.saber_names[1].is_some()
        },
        dead: flags & 2 != 0
            || mesh.body_identity.is_some()
            || mesh.corpse_pool
            || predicted.is_some_and(|p| p.health <= 0)
            || (local && player.is_some_and(|p| p.health() <= 0)),
        mounted: if local {
            predicted
                .map(|p| p.vehicle_entity_num)
                .or_else(|| player.map(|p| p.vehicle_entity_num()))
                .unwrap_or(0)
                != 0
        } else {
            state.is_some_and(|s| s.vehicle_entity_num() != 0)
        },
        hidden: hidden
            || mesh.surfaces.weapon_lost
            || entity.kind != EntityKind::Actor
            || state.is_some_and(|s| {
                !sinks
                    .holsters
                    .allows_subject(s.entity_type(), s.npc_class())
            }),
        flags,
        cloaked: if local {
            player.is_some_and(|p| p.powerup_active(11, sinks.presentation_time as i32))
        } else {
            state.is_some_and(|s| s.powerups() & (1 << 11) != 0)
        },
        sight: sinks.viewer_force_powers_active & (1 << 14) != 0,
        viewed_player,
        third_person: sinks.third_person || sinks.detached_camera,
    };
    let Some(catalog) = sinks.saber_hilts else {
        mesh.holsters.hide();
        return Plan::default();
    };
    let staff = mesh.saber_names[1].is_none()
        && mesh.saber_names[0]
            .as_deref()
            .is_some_and(|name| catalog.get(name).is_staff);
    let staff_back = staff && sinks.holsters.back;
    let worn = sinks.holsters.visible(Body {
        viewed_player: false,
        ..body
    });
    // The handoff only needs the model's back tag to exist; its matrix is evaluated
    // below only when a hilt is drawn. A missing tag never removes the held staff.
    let back_tag = staff_back && worn && mesh.holsters.has_back_tag();
    let (phase, putting_away) = if back_tag && mesh.entity_id == Some(entity.id) && !in_flight {
        mesh.holsters
            .swap
            .phase(mesh, sinks.presentation_time, sinks.holsters.swap)
    } else {
        mesh.holsters.swap.reset();
        (Phase::None, false)
    };
    let plan = Plan {
        on_back: phase == Phase::OnBack,
        blades_in: phase == Phase::OnBack || putting_away,
    };
    let audio_enabled = back_tag
        && crate::holstered_sabers::staff::installed(mesh)
        && !crate::muted_players::entity_muted(sinks.muted, entity.id.get());
    // Only a staff reach has sounds to synchronise; any other slot is released.
    if audio_enabled
        && let Some(audio) = sinks.game_audio.as_mut()
        && mesh.saber_names[0].is_some()
    {
        audio.holstered_staff(
            entity.id.get().saturating_sub(1) as usize,
            mesh.holsters.identity(),
            transform.translation,
            phase,
            putting_away,
            mesh.animator
                .requested
                .map(|s| (s.upper.clip, s.upper.revision, s.upper.started_at_millis)),
            equipment.is_some_and(|e| e.active),
            sinks.presentation_time,
        );
    }
    let saber_held = equipment.is_some_and(|e| e.kind == sjk_runtime::HeldItemKind::EnergyBlade);
    // Bolts are evaluated only while a hilt is drawn, never for a saber in hand.
    if !sinks.holsters.visible(body) || phase == Phase::InHand || (saber_held && !plan.on_back) {
        mesh.holsters.hide();
        return plan;
    }
    let attachments = mesh
        .holsters
        .attachments(mesh, staff_back, sinks.holster_geometry);
    let rotation = weapon_view::actor_world_rotation(transform.rotation);
    let scale = Vec3::from_array(transform.scale);
    for (hand, attachment) in attachments.into_iter().enumerate() {
        if hand == 0 && in_flight {
            continue;
        }
        let Some((name, mut attachment)) = mesh.saber_names[hand].as_deref().zip(attachment) else {
            continue;
        };
        attachment.grip = (Vec3::from_array(attachment.grip) * scale).to_array();
        let (origin, rotation) = saber::world_attachment(
            Vec3::from_array(transform.translation),
            rotation,
            attachment,
        );
        let mut instance =
            ActorInstance::new(origin.to_array(), rotation.to_array(), transform.scale);
        instance.view_flags = sinks.entity_view_flags | u32::from(!draw_actor);
        let group = &mut sinks.object_groups[catalog.get(name).mesh_index];
        // Shared groups are preallocated; never grow them in the frame loop.
        if group.len() < group.capacity() {
            group.push(instance);
        }
    }
    plan
}
