//! Vote panel backed by BaseJKA vote configstrings: a centred card with the
//! proposal, the time left and the YES / NO counts beside their keys.

use super::{HudOverlay, widgets::EmitContext};
use crate::console::ViewerConsole;
use crate::menu::sjk::color;
use crate::menu_widgets::{VoteLayout, VoteRow};
use sjk_client::sent_commands::SentCommands;
use sjk_protocol::GameState;
use sjk_ui::{
    Color, DrawCommand, DrawList, FontWeight, HudWidget, Rect, TextAlign, TextId, TextOverflow,
    Theme,
};
use std::fmt::Write as _;
use std::time::{Duration, Instant};

/// A BaseJKA vote runs 30 seconds (`VOTE_TIME`, `g_main.c`).
const VOTE_MILLIS: f32 = 30_000.0;
/// First text id of the global vote's card; the team vote's follows it.
const CARD_TEXT_FIRST: u32 = 200;
/// Text ids per card: label, proposal, seconds, YES, NO.
const CARD_TEXTS: u32 = 5;
/// The keys bound to `vote yes` and `vote no`.
pub(super) const YES_KEYS_TEXT: u32 = 210;
pub(super) const NO_KEYS_TEXT: u32 = 211;
/// Last vote text id.
pub(super) const TEXT_LAST: u32 = NO_KEYS_TEXT;

// SJK's own palette ([`crate::menu::sjk::color`]), as its menus' cards draw it.
const YES: Color = color::TRUSTED;
const NO: Color = color::RED_TEAM;
const PLATE: Color = Color::new(0.04, 0.06, 0.12, 0.9);
const SHADOW: Color = color::alpha(color::SPACE, 0.5);
const EDGE: Color = color::alpha(color::HOLO, 0.45);
const TRACK: Color = color::alpha(color::HOLO, 0.16);
/// Room above the card, so it stands under the match timer at the top centre.
const BELOW_TIMER: f32 = 44.0;
/// A vote that starts this soon after the player's own `callvote` is theirs: the
/// server counts its caller's YES (`Cmd_CallVote_f`).
const CALL_WINDOW: Duration = Duration::from_secs(5);
/// The keys that vote while the player has not voted yet, whatever they are bound to.
pub(super) const YES_KEY: &str = "F1";
pub(super) const NO_KEY: &str = "F2";

/// One vote's texts and numbers for its card.
#[derive(Debug, Default)]
pub(super) struct Card {
    label: String,
    text: String,
    seconds: String,
    yes: String,
    no: String,
    /// The vote shows.
    pub(super) active: bool,
    /// Time left as a fraction of the vote's 30 seconds.
    pub(super) time: f32,
    /// Votes cast for and against.
    pub(super) yes_count: i32,
    pub(super) no_count: i32,
    /// The player's vote in this one: YES, NO or not yet.
    pub(super) voted: Option<bool>,
    /// The card is the `testvote` preview, not a server's vote.
    preview: bool,
    /// The player's own `callvote` (or `callteamvote`) left at this time.
    called: Option<Instant>,
}

impl Card {
    fn set(&mut self, vote: Option<sjk_client::VoteState<'_>>, label: &str) {
        self.label.clear();
        self.text.clear();
        self.seconds.clear();
        self.yes.clear();
        self.no.clear();
        let Some(vote) = vote.filter(|vote| vote.active) else {
            self.active = false;
            self.time = 0.0;
            self.yes_count = 0;
            self.no_count = 0;
            self.voted = None;
            self.preview = false;
            return;
        };
        if !self.active {
            // A new vote: the player has not voted in it, unless they called it.
            self.voted = self
                .called
                .take()
                .filter(|at| at.elapsed() <= CALL_WINDOW)
                .map(|_| true);
        }
        self.active = true;
        self.label.push_str(label);
        self.text.push_str(vote.text);
        let _ = write!(self.seconds, "{}s", vote.remaining_seconds().max(0));
        let _ = write!(self.yes, "YES  {}", vote.yes);
        let _ = write!(self.no, "{}  NO", vote.no);
        self.time = (vote.remaining_ms as f32 / VOTE_MILLIS).clamp(0.0, 1.0);
        self.yes_count = vote.yes.max(0);
        self.no_count = vote.no.max(0);
    }

    /// Card text `index` (0 label, 1 proposal, 2 seconds, 3 YES, 4 NO).
    fn text(&self, index: u32) -> &str {
        match index {
            0 => &self.label,
            1 => &self.text,
            2 => &self.seconds,
            3 => &self.yes,
            _ => &self.no,
        }
    }
}

/// The `testvote` console command: a made-up vote on the HUD, to see the panel
/// where no vote can run (alone on a devmap a vote passes at once).
pub(crate) const PREVIEW_COMMAND: &str = "testvote";
/// Its help line.
pub(crate) const PREVIEW_HELP: &str =
    "Show a made-up vote on the HUD for 30 seconds: testvote [team], testvote off";
/// `testpoll <question>`: the preview as an `ampoll` with the player's question.
pub(crate) const POLL_PREVIEW_COMMAND: &str = "testpoll";
/// Its help line.
pub(crate) const POLL_PREVIEW_HELP: &str =
    "Show a made-up ampoll with your question on the HUD for 30 seconds: testpoll <question>";
/// The preview's question when `testvote` gives none.
const PREVIEW_QUESTION: &str = "poll all vs all?";

/// A `testvote` preview's state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) enum Preview {
    #[default]
    Off,
    /// Asked for; starts at the next HUD update, with its server time.
    Asked { team: bool },
    /// Running since this server time.
    Since { start: i32, team: bool },
}

/// The preview's vote `elapsed` ms in: counts that grow, and the time left.
fn preview_vote(
    elapsed: i32,
    scope: sjk_client::LegacyVoteScope,
    question: &str,
) -> sjk_client::VoteState<'_> {
    let team = scope != sjk_client::LegacyVoteScope::Global;
    sjk_client::VoteState {
        active: true,
        scope,
        text: if team { "Leader: Rain" } else { question },
        yes: 1 + elapsed / 2_500,
        no: elapsed / 6_000,
        remaining_ms: VOTE_MILLIS as i32 - elapsed,
    }
}

/// What [`HudOverlay::take_vote_key`] made of F1 or F2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VoteKey {
    /// Send this vote command.
    Send(&'static str),
    /// The `testvote` card took the vote; nothing goes to the server.
    Preview,
}

/// `key` (F1 or F2), then the other keys bound to `command`.
fn write_vote_keys(console: Option<&ViewerConsole>, command: &str, key: &str, output: &mut String) {
    output.clear();
    let Some(console) = console else {
        output.push_str(key);
        return;
    };
    console.write_keys_for_command(command, output);
    let joined = std::iter::once(key)
        .chain(
            output
                .split(" / ")
                .filter(|bound| *bound != "unbound" && !bound.eq_ignore_ascii_case(key)),
        )
        .collect::<Vec<_>>()
        .join(" / ");
    *output = joined;
}

/// What the vote widget draws: the global and the team vote.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Meters {
    pub(super) cards: [Meter; 2],
}

/// One card's numbers.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Meter {
    pub(super) active: bool,
    pub(super) time: f32,
    pub(super) yes: i32,
    pub(super) no: i32,
    /// The player has voted: the card folds to one row, the YES / NO bar.
    pub(super) voted: bool,
}

impl HudOverlay {
    pub(super) fn update_votes(
        &mut self,
        game_state: &GameState,
        team: u8,
        server_time: i32,
        console: Option<&ViewerConsole>,
        sent: Option<&SentCommands>,
    ) {
        if let Some(sent) = sent {
            self.read_sent_votes(sent);
        }
        let mut global = Some(sjk_client::legacy_global_vote(game_state, server_time));
        let mut team_vote = sjk_client::legacy_team_vote(game_state, team, server_time);
        let mut preview = [false; 2];
        if let Preview::Asked { team } = self.vote_preview {
            self.vote_preview = Preview::Since {
                start: server_time,
                team,
            };
        }
        if let Preview::Since { start, team } = self.vote_preview {
            let elapsed = server_time.saturating_sub(start);
            if !(0..VOTE_MILLIS as i32).contains(&elapsed) {
                self.vote_preview = Preview::Off;
            } else {
                // A real vote keeps its card.
                if !global.is_some_and(|vote| vote.active) {
                    global = Some(preview_vote(
                        elapsed,
                        sjk_client::LegacyVoteScope::Global,
                        &self.preview_question,
                    ));
                    preview[0] = true;
                }
                if team && !team_vote.is_some_and(|vote| vote.active) {
                    team_vote = Some(preview_vote(
                        elapsed,
                        sjk_client::LegacyVoteScope::RedTeam,
                        &self.preview_question,
                    ));
                    preview[1] = true;
                }
            }
        }
        self.vote_cards[0].set(global, "VOTE");
        self.vote_cards[1].set(team_vote, "TEAM VOTE");
        for (card, preview) in self.vote_cards.iter_mut().zip(preview) {
            if card.active && card.preview != preview {
                // A preview card and a real vote are different votes.
                card.voted = None;
            }
            card.preview = card.active && preview;
        }
        self.vote_active = self.vote_cards[0].active;
        self.team_vote_active = self.vote_cards[1].active;
        // F1 and F2 vote ([`Self::take_vote_key`]); other keys bound to the vote
        // commands are listed after them.
        write_vote_keys(console, "vote yes", YES_KEY, &mut self.yes_keys);
        write_vote_keys(console, "vote no", NO_KEY, &mut self.no_keys);
    }

    /// Note the votes and calls among the commands this client sent since the last
    /// look: a vote cast with any key, the console or a menu shrinks its card too.
    fn read_sent_votes(&mut self, sent: &SentCommands) {
        let seen = self.sent_votes_seen;
        for (number, at, command) in sent.iter().filter(|(number, ..)| *number > seen) {
            self.sent_votes_seen = number;
            let Ok(command) = std::str::from_utf8(command) else {
                continue;
            };
            let mut words = command.split_whitespace();
            let verb = words.next().unwrap_or("").to_ascii_lowercase();
            let (card, call) = match verb.as_str() {
                "vote" => (0, false),
                "teamvote" => (1, false),
                "callvote" => (0, true),
                "callteamvote" => (1, true),
                _ => continue,
            };
            let card = &mut self.vote_cards[card];
            if call {
                if !card.active {
                    card.called = Some(at);
                }
                continue;
            }
            // BaseJKA reads the first letter: y or 1 for YES (`Cmd_Vote_f`).
            let yes = words
                .next()
                .and_then(|word| word.chars().next())
                .is_some_and(|first| matches!(first, 'y' | 'Y' | '1'));
            if card.active && !card.preview && card.voted.is_none() {
                card.voted = Some(yes);
            }
        }
    }

    /// F1 (`yes`) or F2 went down: while a vote shows that the player has not voted
    /// in, the key votes in it (the global vote first) instead of running its bind.
    pub(crate) fn take_vote_key(&mut self, yes: bool) -> Option<VoteKey> {
        let index = self
            .vote_cards
            .iter()
            .position(|card| card.active && card.voted.is_none())?;
        let card = &mut self.vote_cards[index];
        card.voted = Some(yes);
        if card.preview {
            return Some(VoteKey::Preview);
        }
        Some(VoteKey::Send(match (index, yes) {
            (0, true) => "vote yes",
            (0, false) => "vote no",
            (_, true) => "teamvote yes",
            (_, false) => "teamvote no",
        }))
    }

    /// `testvote [team|off]`: start (or stop) the made-up vote; the reply line.
    pub(crate) fn preview_vote_command(&mut self, args: &[String]) -> Result<String, String> {
        self.preview_question.clear();
        self.preview_question.push_str(PREVIEW_QUESTION);
        match args {
            [] => self.vote_preview = Preview::Asked { team: false },
            [word] if word.eq_ignore_ascii_case("team") => {
                self.vote_preview = Preview::Asked { team: true };
            }
            [word] if word.eq_ignore_ascii_case("off") => {
                self.vote_preview = Preview::Off;
                return Ok("Test vote off".into());
            }
            _ => return Err(format!("usage: {PREVIEW_COMMAND} [team|off]")),
        }
        Ok("Test vote on the HUD for 30 seconds (only you see it)".into())
    }

    /// `testpoll <question>`: the `testvote` card asking `Poll: <question>`, as a JA+
    /// `ampoll` shows; the reply line.
    pub(crate) fn preview_poll_command(&mut self, args: &[String]) -> Result<String, String> {
        let question = args.join(" ");
        let question = question.trim();
        if question.is_empty() {
            return Err(format!("usage: {POLL_PREVIEW_COMMAND} <question>"));
        }
        self.preview_question.clear();
        self.preview_question.push_str("Poll: ");
        self.preview_question.push_str(question);
        self.vote_preview = Preview::Asked { team: false };
        Ok("Test poll on the HUD for 30 seconds (only you see it)".into())
    }

    /// The numbers the vote widget draws.
    pub(super) fn vote_meters(&self) -> Meters {
        Meters {
            cards: self.vote_cards.each_ref().map(|card| Meter {
                active: card.active,
                time: card.time,
                yes: card.yes_count,
                no: card.no_count,
                voted: card.voted.is_some(),
            }),
        }
    }

    /// Vote text `id`, from [`CARD_TEXT_FIRST`] to [`TEXT_LAST`].
    pub(super) fn vote_text(&self, id: u32) -> &str {
        match id {
            YES_KEYS_TEXT => &self.yes_keys,
            NO_KEYS_TEXT => &self.no_keys,
            _ => {
                let offset = id.saturating_sub(CARD_TEXT_FIRST);
                let card = (offset / CARD_TEXTS) as usize;
                self.vote_cards
                    .get(card)
                    .map_or("", |card| card.text(offset % CARD_TEXTS))
            }
        }
    }
}

pub(super) fn emit(
    draw: &mut DrawList,
    _theme: Theme,
    _widget: &HudWidget,
    rect: Rect,
    context: &EmitContext<'_>,
) {
    let meters = context.data.votes;
    if !meters.cards.iter().any(|card| card.active) {
        return;
    }
    let hud = context.hero_scale;
    let ratio = hud / context.dpi_scale;
    // The card is centred on the widget's anchor, so a wide widget does not leave it
    // at the left.
    let center = rect.x + rect.width * 0.5;
    let mut y = rect.y * ratio + BELOW_TIMER * hud;
    for (index, meter) in meters.cards.iter().enumerate() {
        if !meter.active {
            continue;
        }
        let scale = hud;
        let width = VoteLayout::WIDTH * scale;
        let top = Rect::new(center - width * 0.5, y, width, 0.0);
        let plate = if meter.voted {
            // Voted: only how the vote goes, on one row.
            let row = VoteRow::new(top, scale);
            emit_row(draw, &row, *meter, index as u32, scale);
            row.plate
        } else {
            let layout = VoteLayout::new(top, scale);
            emit_card(draw, &layout, *meter, index as u32, scale);
            layout.plate
        };
        y += plate.height + 10.0 * scale;
    }
}

/// The SJK menu card's plate: a soft shadow, the navy plate and a holo edge.
fn emit_plate(draw: &mut DrawList, plate: Rect, radius: f32, scale: f32) {
    // The SJK menu card: a soft shadow, the navy plate and a holo edge.
    let _ = draw.push(DrawCommand::RoundedRect {
        rect: Rect::new(
            plate.x + 4.0 * scale,
            plate.y + 8.0 * scale,
            plate.width,
            plate.height,
        ),
        radius,
        color: SHADOW,
    });
    let _ = draw.push(DrawCommand::RoundedRect {
        rect: plate,
        radius,
        color: PLATE,
    });
    let _ = draw.push(DrawCommand::Border {
        rect: plate,
        radius,
        width: 1.5 * scale,
        color: EDGE,
    });
}

/// The voted card: the time left along the top, YES count, the proposal over the YES / NO
/// bar, NO count.
fn emit_row(draw: &mut DrawList, row: &VoteRow, meter: Meter, card: u32, scale: f32) {
    let text = |index: u32| TextId(CARD_TEXT_FIRST + card * CARD_TEXTS + index);
    emit_plate(draw, row.plate, 12.0 * scale, scale);
    bar(draw, row.time, 1.0, TRACK, scale);
    bar(draw, row.time, meter.time, color::GOLD, scale);
    emit_text(
        draw,
        row.question,
        text(1),
        18.0 * scale,
        color::TEXT,
        TextAlign::Center,
        FontWeight::Semibold,
        0.3,
    );
    emit_split(draw, row.split, meter, scale);
    emit_text(
        draw,
        row.counts,
        text(3),
        20.0 * scale,
        YES,
        TextAlign::Start,
        FontWeight::Semibold,
        1.0,
    );
    emit_text(
        draw,
        row.counts,
        text(4),
        20.0 * scale,
        NO,
        TextAlign::End,
        FontWeight::Semibold,
        1.0,
    );
}

fn emit_card(draw: &mut DrawList, layout: &VoteLayout, meter: Meter, card: u32, scale: f32) {
    let text = |index: u32| TextId(CARD_TEXT_FIRST + card * CARD_TEXTS + index);
    emit_plate(draw, layout.plate, 18.0 * scale, scale);
    // The gold mark beside the heading, as SJK's menu bands have.
    let _ = draw.push(DrawCommand::RoundedRect {
        rect: Rect::new(
            layout.label.x - 10.0 * scale,
            layout.label.y + 5.0 * scale,
            4.0 * scale,
            layout.label.height - 10.0 * scale,
        ),
        radius: 2.0 * scale,
        color: color::GOLD_BRIGHT,
    });
    emit_text(
        draw,
        layout.label,
        text(0),
        17.0 * scale,
        color::GOLD_BRIGHT,
        TextAlign::Start,
        FontWeight::Semibold,
        2.0,
    );
    emit_text(
        draw,
        layout.label,
        text(2),
        22.0 * scale,
        color::TEXT,
        TextAlign::End,
        FontWeight::Semibold,
        0.5,
    );
    // Time left, shrinking from the full width.
    bar(draw, layout.time, 1.0, TRACK, scale);
    bar(draw, layout.time, meter.time, color::GOLD, scale);
    emit_text(
        draw,
        layout.proposal,
        text(1),
        30.0 * scale,
        color::TEXT,
        TextAlign::Center,
        FontWeight::Semibold,
        0.3,
    );
    emit_split(draw, layout.split, meter, scale);
    emit_text(
        draw,
        layout.counts,
        text(3),
        24.0 * scale,
        YES,
        TextAlign::Start,
        FontWeight::Semibold,
        1.0,
    );
    emit_text(
        draw,
        layout.counts,
        text(4),
        24.0 * scale,
        NO,
        TextAlign::End,
        FontWeight::Semibold,
        1.0,
    );
    emit_text(
        draw,
        layout.keys,
        TextId(YES_KEYS_TEXT),
        15.0 * scale,
        color::MUTED,
        TextAlign::Start,
        FontWeight::Regular,
        0.5,
    );
    emit_text(
        draw,
        layout.keys,
        TextId(NO_KEYS_TEXT),
        15.0 * scale,
        color::MUTED,
        TextAlign::End,
        FontWeight::Regular,
        0.5,
    );
}

/// YES fills `split` from the left, NO from the right, in proportion to the votes cast.
fn emit_split(draw: &mut DrawList, split: Rect, meter: Meter, scale: f32) {
    bar(draw, split, 1.0, TRACK, scale);
    let cast = meter.yes + meter.no;
    if cast > 0 {
        let yes = meter.yes as f32 / cast as f32;
        bar(draw, split, yes, YES, scale);
        let no = Rect::new(
            split.x + split.width * yes,
            split.y,
            split.width * (1.0 - yes),
            split.height,
        );
        bar(draw, no, 1.0, NO, scale);
    }
}

/// The leading `fraction` of `rect`, rounded.
fn bar(draw: &mut DrawList, rect: Rect, fraction: f32, color: Color, scale: f32) {
    let width = rect.width * fraction.clamp(0.0, 1.0);
    if width <= 0.0 {
        return;
    }
    let _ = draw.push(DrawCommand::RoundedRect {
        rect: Rect::new(rect.x, rect.y, width, rect.height),
        radius: (rect.height * 0.5).min(width * 0.5).min(4.0 * scale),
        color,
    });
}

#[allow(clippy::too_many_arguments)]
fn emit_text(
    draw: &mut DrawList,
    rect: Rect,
    text: TextId,
    size: f32,
    color: Color,
    align: TextAlign,
    weight: FontWeight,
    letter_spacing: f32,
) {
    let _ = draw.push(DrawCommand::Text {
        rect,
        text,
        size,
        color,
        align,
        overflow: TextOverflow::Ellipsis,
        weight,
        letter_spacing,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_texts_follow_the_vote() {
        let mut card = Card::default();
        card.set(
            Some(sjk_client::VoteState {
                active: true,
                scope: sjk_client::LegacyVoteScope::Global,
                text: "poll all vs all?",
                yes: 14,
                no: 4,
                remaining_ms: 12_400,
            }),
            "VOTE",
        );
        assert!(card.active);
        assert_eq!(card.text(0), "VOTE");
        assert_eq!(card.text(1), "poll all vs all?");
        assert_eq!(card.text(2), "12s");
        assert_eq!(card.text(3), "YES  14");
        assert_eq!(card.text(4), "4  NO");
        assert!((card.time - 12.4 / 30.0).abs() < 1e-4);
        card.set(None, "VOTE");
        assert!(!card.active);
        assert_eq!(card.text(1), "");
        assert_eq!((card.yes_count, card.no_count), (0, 0));
    }

    #[test]
    fn f1_votes_once_and_the_card_folds() {
        let game = GameState::empty_local(0);
        let mut hud = HudOverlay::new();
        hud.vote_preview = Preview::Asked { team: true };
        hud.update_votes(&game, 1, 1_000, None, None);
        assert!(
            hud.vote_cards
                .iter()
                .all(|card| card.active && card.voted.is_none())
        );
        // The preview takes the vote without sending; the global card first.
        assert_eq!(hud.take_vote_key(true), Some(VoteKey::Preview));
        assert_eq!(hud.vote_cards[0].voted, Some(true));
        assert_eq!(hud.vote_cards[1].voted, None);
        assert_eq!(hud.take_vote_key(false), Some(VoteKey::Preview));
        assert_eq!(hud.vote_cards[1].voted, Some(false));
        // Both voted: F1 and F2 run their binds again.
        assert_eq!(hud.take_vote_key(true), None);
        for time in (1_050..=1_400).step_by(50) {
            hud.update_votes(&game, 1, time, None, None);
        }
        let meters = hud.vote_meters();
        assert!(meters.cards.iter().all(|card| card.voted));
        assert_eq!(hud.vote_text(YES_KEYS_TEXT), "F1");
    }

    #[test]
    fn testpoll_asks_the_players_question() {
        let game = GameState::empty_local(0);
        let mut hud = HudOverlay::new();
        assert!(hud.preview_poll_command(&[]).is_err());
        let words = ["do", "you", "like", "apple", "pie?"].map(String::from);
        hud.preview_poll_command(&words).unwrap();
        hud.update_votes(&game, 0, 1_000, None, None);
        assert_eq!(
            hud.vote_text(CARD_TEXT_FIRST + 1),
            "Poll: do you like apple pie?"
        );
        hud.preview_vote_command(&[]).unwrap();
        hud.update_votes(&game, 0, 1_050, None, None);
        assert_eq!(hud.vote_text(CARD_TEXT_FIRST + 1), PREVIEW_QUESTION);
    }
}
