//! Timed, presentation-only player monitoring, from JoF EJKSol's CG_Peek_f
//! and CG_CalcViewValues. Only players in the received snapshot are available.
//! A player who runs SJK online is asked first and sees who watches them
//! (`peek_consent.rs`); either side can stop with the peek card (`peek_card.rs`).
use crate::peek_consent::{self, Consent, Gate, Heard};
use crate::{GpuState, movement_collision};
use glam::Vec3;
use sjk_protocol::GameState;
use sjk_runtime::EntityId;
use std::net::SocketAddr;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

const CS_PLAYERS: usize = 1131;
#[derive(Default)]
pub(crate) struct State {
    target: Option<Target>,
    /// The questions and the watcher, over the hub.
    consent: Consent,
    /// The server the questions were for.
    server: Option<SocketAddr>,
    pub(crate) card: crate::peek_card::Card,
}
impl State {
    /// Missing PVS data pauses a peek without changing its deadline or identity.
    fn can_render(&mut self, time: i64, identity_current: bool, available: bool) -> bool {
        let Some(target) = self.target.as_ref() else {
            return false;
        };
        if !identity_current || time < target.start - 1000 || time >= target.end {
            self.target = None;
            return false;
        }
        available
    }
}

struct Target {
    client: u16,
    name: Vec<u8>,
    map: Vec<u8>,
    start: i64,
    end: i64,
    /// The target allowed it over the hub, so they are told when it stops early.
    asked: bool,
}

fn name(game: &GameState, client: u16) -> Option<&[u8]> {
    sjk_client::LegacyClientInfo::new(game.config_string(CS_PLAYERS + usize::from(client))?)
        .bytes("n")
}
fn map(game: &GameState) -> &[u8] {
    game.config_string(0)
        .and_then(|info| sjk_protocol::info_value(info, b"mapname"))
        .unwrap_or_default()
}
fn clean(bytes: &[u8]) -> String {
    let bytes = sjk_game_jka::client_view::strip_colours(bytes);
    String::from_utf8(bytes.clone())
        .unwrap_or_else(|_| {
            bytes
                .into_iter()
                .map(sjk_protocol::windows_1252_char)
                .collect()
        })
        .to_lowercase()
}

/// Slot, exact colour-insensitive name, then an unambiguous name fragment.
fn resolve<'a>(players: impl Iterator<Item = (u16, &'a [u8])>, query: &str) -> Result<u16, String> {
    let players: Vec<_> = players.collect();
    if let Ok(slot) = query.parse::<u16>()
        && players.iter().any(|(client, _)| *client == slot)
    {
        return Ok(slot);
    }
    let query = clean(query.as_bytes());
    if query.trim().is_empty() {
        return Err("peek: enter a player name or client ID".into());
    }
    let exact: Vec<_> = players
        .iter()
        .filter(|(_, name)| clean(name) == query)
        .collect();
    let matches: Vec<_> = if exact.is_empty() {
        players
            .iter()
            .filter(|(_, name)| clean(name).contains(&query))
            .collect()
    } else {
        exact
    };
    match matches.as_slice() {
        [player] => Ok(player.0),
        [] => Err("peek: no matching player".into()),
        _ => Err("peek: name is ambiguous; use a client ID".into()),
    }
}
fn duration(text: Option<&str>) -> Result<i64, String> {
    let seconds = text
        .unwrap_or("5")
        .parse::<f64>()
        .map_err(|_| "peek: invalid duration")?;
    if !seconds.is_finite() {
        return Err("peek: duration must be finite".into());
    }
    Ok((seconds.clamp(0., 60.) * 1000.) as i64)
}

/// Whether the own identity is on and registered at the SJK hub, so a peek's target can
/// be asked and its events heard.
fn on_hub(state: &GpuState) -> bool {
    state
        .console
        .as_ref()
        .and_then(|console| console.bool_cvar("cl_identity"))
        == Some(true)
        && crate::player_identity::with_snapshot(|snapshot| snapshot.me.is_some()) == Some(true)
}

/// Flight cancels monitoring even before the requested free-camera mode is synced.
fn local_flight(state: &GpuState) -> bool {
    state.local_prediction.fake_noclip()
        || state
            .console
            .as_ref()
            .and_then(|console| console.integer_cvar("cg_freecamera"))
            .unwrap_or(0)
            != 0
}

impl GpuState {
    /// `/peek [id|name] [seconds]`, `/peek off`, or the crosshair player without
    /// arguments; a bare `/peek` during the own peek or question stops it. `/peek off`
    /// also ends a watcher's peek.
    pub(crate) fn peek_command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        if args.len() > 2 {
            return Err("Usage: peek [id|name] [seconds], or peek off".into());
        }
        if args
            .first()
            .is_some_and(|arg| arg.eq_ignore_ascii_case("off"))
            || (args.is_empty()
                && (self.peek.target.is_some() || self.peek.consent.asking.is_some()))
        {
            return Ok(self.stop_peek());
        }
        let milliseconds = duration(args.get(1).map(String::as_str))?;
        if milliseconds == 0 {
            return Ok(self.stop_peek());
        }
        if local_flight(self) {
            return Err("peek: end local flight before watching a player".into());
        }
        let session = self.live_session.as_ref().ok_or("peek: not in a game")?;
        let game = session.game_state();
        let snapshot = session.latest_snapshot();
        if snapshot.player.movement_type() == 7 {
            return Err("peek: unavailable during intermission".into());
        }
        let client = if let Some(query) = args.first() {
            resolve(
                (0..64).filter_map(|client| Some((client, name(game, client)?))),
                query,
            )?
        } else {
            self.crosshair_scan
                .chat_client(snapshot.server_time)
                .ok_or("peek: aim at a player, or use peek <id|name> [seconds]")?
        };
        if client == snapshot.player.client_num() {
            return Err("peek: select another player".into());
        }
        if self.muted_players.contains(client) {
            return Err("peek: you blocked this player; /unblock them first".into());
        }
        if !snapshot
            .entities
            .iter()
            .any(|entity| entity.number() == client && entity.entity_type() == 1)
        {
            return Err(
                "peek: player is outside your received view; no camera data available".into(),
            );
        }
        let target_name = name(game, client).ok_or("peek: player left")?.to_vec();
        let target_map = map(game).to_vec();
        let server = session.server();
        let now = i64::from(snapshot.server_time);
        let seconds = milliseconds as f64 / 1000.;
        // An SJK player on the hub is asked first; a stock client cannot be. Without the
        // own identity on the hub nobody could be told, so a server peek waits for it.
        if !session.is_local() && !on_hub(self) {
            return Err(
                "peek: turn on your SJK identity (cl_identity 1) so the player can be asked".into(),
            );
        }
        let shown = crate::player_identity::shown_name(game, usize::from(client));
        let askable = !session.is_local()
            && shown.as_deref().is_some_and(|shown| {
                crate::player_identity::hub_mark(client as u8, shown).is_some()
            });
        if askable {
            let shown = shown.unwrap_or_default();
            if self.peek.consent.asking.is_some() {
                return Err("peek: still waiting for an answer; peek off cancels".into());
            }
            if !crate::player_identity::toy_event(
                peek_consent::TOY,
                &peek_consent::request_event(milliseconds),
                client as u8,
            ) {
                return Err("peek: the SJK hub is not reachable to ask them".into());
            }
            self.stop_own_peek();
            self.peek.server = Some(server);
            self.peek
                .consent
                .ask(client, shown.clone(), milliseconds, now);
            return Ok(vec![format!(
                "peek: asked {shown}^7 to let you watch for {seconds:.1}s; waiting for an answer"
            )]);
        }
        self.stop_own_peek();
        self.peek.target = Some(Target {
            client,
            name: target_name,
            map: target_map,
            start: now,
            end: now + milliseconds,
            asked: false,
        });
        Ok(vec![format!(
            "peek: watching client {client} for {seconds:.1}s (the SJK hub does not know them, so \
             they cannot be asked); Del or /peek off returns"
        )])
    }

    /// Whether the card shows something Stop puts away: a peek, a question or a watcher.
    fn peek_stoppable(&self) -> bool {
        self.peek.target.is_some()
            || self.peek.consent.asking.is_some()
            || self.peek.consent.watcher.is_some()
    }

    /// End the own peek (telling an asked target) and withdraw an open question.
    fn stop_own_peek(&mut self) -> bool {
        let mut stopped = false;
        if let Some(target) = self.peek.target.take() {
            if target.asked {
                crate::player_identity::toy_event(
                    peek_consent::TOY,
                    peek_consent::END,
                    target.client as u8,
                );
            }
            stopped = true;
        }
        if let Some(asking) = self.peek.consent.asking.take() {
            crate::player_identity::toy_event(
                peek_consent::TOY,
                peek_consent::END,
                asking.slot as u8,
            );
            stopped = true;
        }
        stopped
    }

    /// Stop everything the card shows: the own peek, the question, and the watcher.
    pub(crate) fn stop_peek(&mut self) -> Vec<String> {
        let mut lines = Vec::new();
        if self.stop_own_peek() {
            lines.push("peek: returned to your camera".to_owned());
        }
        if let Some(watcher) = self.peek.consent.watcher.take() {
            crate::player_identity::toy_event(
                peek_consent::TOY,
                peek_consent::END,
                watcher.slot as u8,
            );
            lines.push(format!("peek: {}^7 no longer watches you", watcher.name));
        }
        if lines.is_empty() {
            lines.push("peek: returned to your camera".to_owned());
        }
        lines
    }

    /// Answer the request on show: allow lets its sender watch for the time asked.
    pub(crate) fn answer_peek(&mut self, allow: bool) -> Vec<String> {
        let Some(session) = self.live_session.as_ref() else {
            return Vec::new();
        };
        let now = i64::from(session.latest_snapshot().server_time);
        let Some((event, sender)) = self.peek.consent.answer(allow, now) else {
            return vec!["peek: nobody asks to watch you".to_owned()];
        };
        crate::player_identity::toy_event(peek_consent::TOY, event, sender.slot as u8);
        vec![if allow {
            format!(
                "peek: {}^7 may watch you for {:.0}s; Del or /peek off stops it",
                sender.name,
                sender.milliseconds as f64 / 1000.
            )
        } else {
            format!("peek: you did not let {}^7 watch you", sender.name)
        }]
    }

    /// The frame's peek questions: the hub's events for this server (`events`), the
    /// answers waited for and the watcher's time.
    pub(crate) fn update_peek(&mut self, events: &[sjk_identity::ToyEvent]) {
        let Some(session) = self
            .live_session
            .as_ref()
            .filter(|session| !session.is_local())
        else {
            self.peek.consent.clear();
            return;
        };
        if self.peek.server != Some(session.server()) {
            self.peek.consent.clear();
            self.peek.server = Some(session.server());
        }
        let game = session.game_state();
        let now = i64::from(session.latest_snapshot().server_time);
        let own = u8::try_from(game.client_num).ok();
        let takes_requests = self
            .console
            .as_ref()
            .and_then(|console| console.bool_cvar(peek_consent::CVAR))
            != Some(false);
        let muted = self.muted_players.slots();
        let mut lines = Vec::new();
        for event in events {
            if event.toy != peek_consent::TOY
                || Some(event.slot) == own
                || Some(event.target) != own
            {
                continue;
            }
            // A sender counts only when the game shows the name they claimed.
            let Some(shown) = crate::player_identity::shown_name(game, usize::from(event.slot))
                .filter(|shown| sjk_identity::names_match(&event.claimed_name, shown))
            else {
                continue;
            };
            let sender = u16::from(event.slot);
            // A muted player's requests go unseen; with requests off, each is refused.
            let gate = if muted.contains(usize::from(event.slot)) {
                Gate::Ignore
            } else if takes_requests {
                Gate::Ask
            } else {
                Gate::Refuse
            };
            match self
                .peek
                .consent
                .hear(sender, &shown, &event.event, now, gate)
            {
                Some(Heard::Refused) => {
                    crate::player_identity::toy_event(
                        peek_consent::TOY,
                        peek_consent::DENY,
                        event.slot,
                    );
                }
                Some(Heard::Request) => {
                    let request = self.peek.consent.request.as_ref().expect("just heard");
                    lines.push(format!(
                        "peek: {shown}^7 asks to watch you for {:.0}s; Ins allows, Del denies, Esc to click",
                        request.milliseconds as f64 / 1000.
                    ));
                }
                Some(Heard::Allowed(party)) => {
                    let Some(target_name) = name(game, sender) else {
                        continue;
                    };
                    self.peek.target = Some(Target {
                        client: sender,
                        name: target_name.to_vec(),
                        map: map(game).to_vec(),
                        start: now,
                        end: now + party.milliseconds,
                        asked: true,
                    });
                    lines.push(format!(
                        "peek: {shown}^7 allowed it; watching for {:.1}s, Del returns",
                        party.milliseconds as f64 / 1000.
                    ));
                }
                Some(Heard::Denied(_)) => {
                    lines.push(format!("peek: {shown}^7 did not let you watch"));
                }
                Some(Heard::Ended { slot, watcher }) => {
                    if self
                        .peek
                        .target
                        .take_if(|target| target.client == slot && target.asked)
                        .is_some()
                    {
                        lines.push(format!("peek: {shown}^7 ended your peek"));
                    }
                    if watcher.is_some() {
                        lines.push(format!("peek: {shown}^7 no longer watches you"));
                    }
                }
                None => {}
            }
        }
        // Blocking the watcher ends their peek.
        if let Some(watcher) = self
            .peek
            .consent
            .watcher
            .take_if(|watcher| muted.contains(usize::from(watcher.slot)))
        {
            crate::player_identity::toy_event(
                peek_consent::TOY,
                peek_consent::END,
                watcher.slot as u8,
            );
            lines.push(format!("peek: {}^7 no longer watches you", watcher.name));
        }
        if let Some(asked) = self.peek.consent.expire(now) {
            lines.push(format!("peek: {}^7 did not answer", asked.name));
        }
        self.peek.consent.keep_present(|slot, name| {
            crate::player_identity::shown_name(game, usize::from(slot)).as_deref() == Some(name)
        });
        if let Some(console) = self.console.as_mut() {
            for line in lines {
                console.push_log(line);
            }
        }
    }

    /// A key while the peek card shows in play: Insert allows a request, Delete denies it
    /// or stops what the card shows, each while the key is bound to nothing else. Escape
    /// is left to the game menu, where the card's buttons take a click. True when the
    /// key was taken.
    pub(crate) fn peek_key(&mut self, event: &KeyEvent) -> bool {
        if event.state != ElementState::Pressed || event.repeat || !self.peek.card.is_shown() {
            return false;
        }
        let PhysicalKey::Code(code) = event.physical_key else {
            return false;
        };
        let unbound = |key: &str| {
            !self
                .console
                .as_ref()
                .is_some_and(|console| console.has_key_binding(key))
        };
        let choice = match code {
            KeyCode::Insert if unbound("INS") => crate::peek_card::Choice::Allow,
            KeyCode::Delete if unbound("DEL") => crate::peek_card::Choice::Deny,
            _ => return false,
        };
        self.peek_card_choice(Some(choice));
        true
    }

    /// Act on the card: a request's answer, or Stop (`Deny` stops when nobody asks).
    pub(crate) fn peek_card_choice(&mut self, choice: Option<crate::peek_card::Choice>) {
        let lines = match choice {
            Some(crate::peek_card::Choice::Allow) if self.peek.consent.request.is_some() => {
                self.answer_peek(true)
            }
            Some(crate::peek_card::Choice::Allow) => return,
            Some(crate::peek_card::Choice::Deny) if self.peek.consent.request.is_some() => {
                self.answer_peek(false)
            }
            Some(crate::peek_card::Choice::Deny | crate::peek_card::Choice::Stop) => {
                if !self.peek_stoppable() {
                    return;
                }
                self.stop_peek()
            }
            None => return,
        };
        if let Some(console) = self.console.as_mut() {
            for line in lines {
                console.push_log(line);
            }
        }
    }

    /// What the peek card shows now, most pressing first: a request to watch the
    /// player, the own peek or question, then the watcher.
    pub(crate) fn peek_card_view(&self) -> Option<crate::peek_card::View> {
        use crate::peek_card::View;
        let session = self.live_session.as_ref()?;
        let now = i64::from(session.latest_snapshot().server_time);
        let left = |until: i64| ((until - now).max(0) + 999) / 1000;
        let consent = &self.peek.consent;
        if let Some(request) = consent.request.as_ref() {
            return Some(View::Request {
                name: request.name.clone(),
                seconds: (request.milliseconds + 999) / 1000,
                left: left(request.until),
            });
        }
        if let Some(target) = self.peek.target.as_ref() {
            let name = crate::player_identity::shown_name(
                session.game_state(),
                usize::from(target.client),
            )
            .unwrap_or_default();
            return Some(View::Watching {
                name,
                left: left(target.end),
            });
        }
        if let Some(asking) = consent.asking.as_ref() {
            return Some(View::Asking {
                name: asking.name.clone(),
                left: left(asking.until),
            });
        }
        consent.watcher.as_ref().map(|watcher| View::Watched {
            name: watcher.name.clone(),
            left: left(watcher.until),
        })
    }
}

/// Derive the rendered view without modifying player origin, angles or commands.
pub(crate) fn camera(state: &mut GpuState, time: i64) -> Option<(Vec3, Vec3)> {
    state.peek.target.as_ref()?;
    if local_flight(state) {
        state.stop_own_peek();
        return None;
    }
    let Some(session) = state.live_session.as_ref() else {
        state.peek.target = None;
        return None;
    };
    let target = state.peek.target.as_ref()?;
    let game = session.game_state();
    let snapshot = session.snapshot_at_or_before(time as i32);
    let identity_current = map(game) == target.map
        && name(game, target.client) == Some(target.name.as_slice())
        && snapshot.player.movement_type() != 7;
    let client = target.client;
    let available = time - i64::from(snapshot.server_time) < 1000
        && snapshot
            .entities
            .iter()
            .any(|entity| entity.number() == client && entity.entity_type() == 1);
    if !state.peek.can_render(time, identity_current, available) {
        return None;
    }
    let entity = state
        .live_world
        .entity(EntityId::new(u64::from(client) + 1))?;
    let transform = entity.sample(time);
    let yaw = entity
        .sample_pose(time)
        .map_or(0., |pose| pose.view_angles_degrees[1])
        .to_radians();
    let focus = Vec3::from_array(transform.translation) + Vec3::Z * 26.;
    let wanted = focus - Vec3::new(yaw.cos(), yaw.sin(), 0.) * 80. + Vec3::Z * 32.;
    let position = movement_collision::camera_trace(
        &state.bsp,
        &mut state.trace_scratch,
        Some((game, snapshot)),
        time as i32,
        focus,
        wanted,
    );
    // A fully collapsed trace still has a defined look direction.
    let look = if position.distance_squared(focus) < 0.001 {
        focus + Vec3::new(yaw.cos(), yaw.sin(), 0.)
    } else {
        focus
    };
    Some((position, look))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn target_loss_pauses_and_resumes_without_extending_the_timer() {
        let mut state = State {
            target: Some(Target {
                client: 1,
                name: b"Tox".to_vec(),
                map: b"mp/duel1".to_vec(),
                start: 1000,
                end: 6000,
                asked: false,
            }),
            ..State::default()
        };
        assert!(state.can_render(2000, true, true));
        assert!(!state.can_render(3000, true, false));
        assert_eq!(state.target.as_ref().unwrap().end, 6000);
        assert!(state.can_render(4000, true, true));
        assert!(!state.can_render(6000, true, false));
        assert!(state.target.is_none());
    }

    #[test]
    fn resolve_names_and_reject_ambiguous_fragments() {
        let players = [(1, b"^1Tox".as_slice()), (2, b"Toxiee".as_slice())];
        assert_eq!(resolve(players.into_iter(), "1").unwrap(), 1);
        assert_eq!(resolve(players.into_iter(), "tox").unwrap(), 1);
        assert!(resolve(players.into_iter(), "to").is_err());
        assert!(resolve(players.into_iter(), "^1^2").is_err());
        assert!(resolve(players.into_iter(), "  ").is_err());
        assert_eq!(resolve(players.into_iter(), "xiee").unwrap(), 2);
    }
    #[test]
    fn durations_are_bounded_and_nonfinite_values_rejected() {
        assert_eq!(duration(None).unwrap(), 5000);
        assert_eq!(duration(Some("90")).unwrap(), 60000);
        assert_eq!(duration(Some("0")).unwrap(), 0);
        assert!(duration(Some("NaN")).is_err());
    }
}
