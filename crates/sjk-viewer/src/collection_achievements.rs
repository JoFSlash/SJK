//! The Collection's Achievements tab: a wall of medallions in bands, each category's
//! run under its name and count (two short categories share a band), each medallion
//! ringed by the count in the category's colour, filled and gold once unlocked, the
//! goal inside and the name under it; the one chosen is shown up close on the right with
//! what to do and how far the player is, and under it the three nearest to unlocking.
//! A hidden achievement not unlocked yet stands as a secret (`?`, no count); a retired
//! one stands only once held, with "No longer obtainable".

use super::view::{LEAD_Y, LEFT_X, SHOWCASE_WIDTH, SHOWCASE_X, centred_lines, glow};
use super::*;
use crate::achievements::Category;
use crate::achievements::medallion::{self, Medallion, tint};
use crate::menu::sjk::{Frame, color, kit, text, wrap};
use crate::menu_widgets::TextFamily;
use sjk_ui::{DrawCommand, FontWeight, TextAlign};

/// The wall: where its first band starts, the room a band takes and how far below its
/// heading the medallions stand (four bands, then five a little closer, which still end
/// over the keys), the medallions a band holds, a medallion's cell and its radius, and
/// how many characters a line of a name under one holds.
const WALL_TOP: f32 = 286.0;
const BAND_STEP: f32 = 172.0;
const BAND_INSET: f32 = 40.0;
const FIVE_TOP: f32 = 272.0;
const FIVE_STEP: f32 = 146.0;
const FIVE_INSET: f32 = 36.0;
const COLUMNS: usize = 12;
const CELL: f32 = 85.0;
const RADIUS: f32 = 28.0;
const NAME_CHARS: usize = 13;
/// The medallion up close.
const BIG_RADIUS: f32 = 104.0;
/// How many the Next up list shows.
const NEXT_UP: usize = 3;

/// One category's run of medallions on a band of the wall: the achievements' indices,
/// the band, its first column, and whether it is the category's first run (a category
/// longer than a band goes on in the next).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Group {
    pub(super) category: Category,
    pub(super) members: Vec<usize>,
    pub(super) band: usize,
    pub(super) column: usize,
    pub(super) first: bool,
}

/// A medallion's place on the wall: its achievement's index, band and column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Slot {
    pub(super) index: usize,
    pub(super) band: usize,
    pub(super) column: usize,
}

/// The wall's runs, of the achievements listed ([`Standing::listed`]): the categories in
/// the catalogue's order, each after the one before on its band when it fits there with
/// a column between them, else on the next band.
pub(super) fn groups(standings: &[Standing]) -> Vec<Group> {
    let mut categories: Vec<(Category, Vec<usize>)> = Vec::with_capacity(8);
    for (index, standing) in standings.iter().enumerate() {
        if !standing.listed() {
            continue;
        }
        let category = standing.kind.category;
        match categories.iter_mut().find(|(other, _)| *other == category) {
            Some((_, members)) => members.push(index),
            None => categories.push((category, vec![index])),
        }
    }
    let mut groups = Vec::with_capacity(categories.len() + 1);
    let (mut band, mut column) = (0, 0);
    for (category, members) in categories {
        for (run, chunk) in members.chunks(COLUMNS).enumerate() {
            let mut start = if column == 0 { 0 } else { column + 1 };
            if start + chunk.len() > COLUMNS {
                band += 1;
                start = 0;
            }
            groups.push(Group {
                category,
                members: chunk.to_vec(),
                band,
                column: start,
                first: run == 0,
            });
            column = start + chunk.len();
        }
    }
    groups
}

/// Every medallion's place, in reading order.
pub(super) fn slots(groups: &[Group]) -> impl Iterator<Item = Slot> + '_ {
    groups.iter().flat_map(|group| {
        group
            .members
            .iter()
            .enumerate()
            .map(|(offset, index)| Slot {
                index: *index,
                band: group.band,
                column: group.column + offset,
            })
    })
}

/// The achievement Up (`down` false) or Down from `index` lands on: the next band's, as
/// near its column as it has. An index not on the wall goes to its first.
pub(super) fn vertical(slots: &[Slot], index: usize, down: bool) -> usize {
    let Some(at) = slots.iter().find(|slot| slot.index == index) else {
        return slots.first().map_or(index, |slot| slot.index);
    };
    let band = if down {
        at.band + 1
    } else {
        at.band.saturating_sub(1)
    };
    slots
        .iter()
        .filter(|slot| slot.band == band)
        .min_by_key(|slot| slot.column.abs_diff(at.column))
        .map_or(index, |slot| slot.index)
}

/// The achievement `by` medallions on from `index` in reading order: wrapping round when
/// `wrap`, else stopping at the ends. An index not on the wall goes to its first.
pub(super) fn sideways(slots: &[Slot], index: usize, by: isize, wrap: bool) -> usize {
    let Some(at) = slots.iter().position(|slot| slot.index == index) else {
        return slots.first().map_or(index, |slot| slot.index);
    };
    let count = slots.len() as isize;
    let next = if wrap {
        (at as isize + by).rem_euclid(count)
    } else {
        (at as isize + by).clamp(0, count - 1)
    };
    slots[next as usize].index
}

/// Whether the board may offer `standing` as one to work towards: listed, locked, and
/// no secret (a secret says nothing of how far it is).
fn open(standing: &Standing) -> bool {
    standing.listed() && standing.unlocked.is_none() && !standing.secret()
}

/// The one to show first: the open one nearest to unlocking, else the first listed.
fn first_choice(standings: &[Standing]) -> usize {
    standings
        .iter()
        .enumerate()
        .filter(|(_, standing)| open(standing))
        .max_by(|a, b| a.1.fraction().total_cmp(&b.1.fraction()))
        .or_else(|| {
            standings
                .iter()
                .enumerate()
                .find(|(_, standing)| standing.listed())
        })
        .map_or(0, |(index, _)| index)
}

/// The open ones nearest to unlocking but `chosen`, nearest first.
fn next_up(standings: &[Standing], chosen: usize) -> Vec<usize> {
    let mut locked: Vec<usize> = (0..standings.len())
        .filter(|index| *index != chosen && open(&standings[*index]))
        .collect();
    locked.sort_by(|a, b| {
        standings[*b]
            .fraction()
            .total_cmp(&standings[*a].fraction())
    });
    locked.truncate(NEXT_UP);
    locked
}

impl Panel {
    /// The Achievements tab.
    pub(super) fn achievements(&mut self, frame: &Frame, inputs: &Inputs<'_>) {
        let standings = inputs.standings;
        if standings.is_empty() {
            self.lead(frame, "No achievements", "");
            return;
        }
        let groups = groups(standings);
        self.achievement_slots.clear();
        self.achievement_slots.extend(slots(&groups));
        if !self
            .achievement_slots
            .iter()
            .any(|slot| slot.index == self.achievement)
        {
            self.achievement = first_choice(standings);
        }
        let s = frame.s;
        let (unlocked, total) = crate::achievements::tally(standings);
        self.lead(frame, &format!("{unlocked} of {total} unlocked"), "");
        let fraction = unlocked as f32 / total.max(1) as f32;
        bar(
            &mut self.ui,
            frame,
            [LEFT_X + 300.0, LEAD_Y + 16.0, 300.0, 6.0],
            fraction,
            color::GOLD,
        );
        let bands = groups.last().map_or(0, |group| group.band + 1);
        let (wall_top, step, inset) = if bands > 4 {
            (FIVE_TOP, FIVE_STEP, FIVE_INSET)
        } else {
            (WALL_TOP, BAND_STEP, BAND_INSET)
        };
        for group in &groups {
            let top = wall_top + group.band as f32 * step;
            let left = LEFT_X + group.column as f32 * CELL;
            let right = left + group.members.len() as f32 * CELL - 8.0;
            let hue = tint(group.category);
            if group.first {
                let members: Vec<&Standing> = groups
                    .iter()
                    .filter(|other| other.category == group.category)
                    .flat_map(|other| other.members.iter())
                    .filter_map(|index| standings.get(*index))
                    .collect();
                let held = members.iter().filter(|s| s.unlocked.is_some()).count();
                text(
                    &mut self.ui,
                    TextFamily::Display,
                    format_args!("{}", group.category.name()),
                    frame.rect(left, top, right - left, 28.0),
                    22.0 * s,
                    hue,
                    FontWeight::Semibold,
                    TextAlign::Start,
                );
                let name_width = crate::text::display_width(group.category.name(), 22.0);
                let count = format!("{held} of {}", members.len());
                text(
                    &mut self.ui,
                    TextFamily::Body,
                    format_args!("{count}"),
                    frame.rect(left + name_width + 12.0, top + 2.0, 90.0, 24.0),
                    15.0 * s,
                    color::MUTED,
                    FontWeight::Regular,
                    TextAlign::Start,
                );
                let rule_x = left + name_width + 24.0 + crate::text::display_width(&count, 15.0);
                if right - rule_x > 12.0 {
                    let _ = self.ui.draw_list_mut().push(DrawCommand::SolidRect {
                        rect: frame.rect(rule_x, top + 14.0, right - rule_x, 1.0),
                        color: color::alpha(hue, 0.3),
                    });
                }
            }
            for (offset, index) in group.members.iter().enumerate() {
                if let Some(standing) = standings.get(*index) {
                    let x = left + offset as f32 * CELL;
                    self.wall_medallion(frame, standing, *index, x, top + inset);
                }
            }
        }
        if let Some(standing) = standings.get(self.achievement) {
            self.achievement_up_close(frame, standing);
            self.next_up(frame, standings);
        }
        centred_lines(
            &mut self.ui,
            frame,
            "Counted on this PC in your matches on servers, and kept on the SJK hub with your identity on.",
            [SHOWCASE_X, 950.0, SHOWCASE_WIDTH],
            96,
            1,
            15.0,
            color::QUIET,
        );
    }

    /// Achievement `index`'s medallion in its cell from (`x`, `top`), its name under it.
    fn wall_medallion(
        &mut self,
        frame: &Frame,
        standing: &Standing,
        index: usize,
        x: f32,
        top: f32,
    ) {
        let s = frame.s;
        let chosen = index == self.achievement;
        let done = standing.unlocked.is_some();
        let centre = [x + CELL * 0.5 - 4.0, top + RADIUS + 4.0];
        if done {
            glow(&mut self.ui, frame, centre, RADIUS * 1.5, color::GOLD, 0.16);
        }
        medallion::draw(
            &mut self.ui,
            Medallion {
                kind: standing.kind,
                centre: frame.point(centre[0], centre[1]),
                radius: RADIUS * s,
                fraction: standing.fraction(),
                done,
            },
        );
        if chosen {
            let _ = self.ui.draw_list_mut().push(DrawCommand::Arc {
                center: frame.point(centre[0], centre[1]),
                radius: (RADIUS + 9.0) * s,
                width: 2.0 * s,
                start: 0.0,
                sweep: std::f32::consts::TAU,
                color: color::GOLD_BRIGHT,
                knockout: None,
            });
        }
        let mut y = top + RADIUS * 2.0 + 14.0;
        let name = if standing.secret() {
            std::borrow::Cow::Borrowed("Secret")
        } else {
            crate::terms::say(standing.kind.name)
        };
        for part in wrap(&name, NAME_CHARS).take(2) {
            text(
                &mut self.ui,
                TextFamily::Body,
                format_args!("{part}"),
                frame.rect(centre[0] - CELL * 0.5 - 6.0, y, CELL + 12.0, 20.0),
                14.0 * s,
                match (chosen, done) {
                    (true, _) => color::GOLD_BRIGHT,
                    (false, true) => color::TEXT,
                    (false, false) => color::MUTED,
                },
                FontWeight::Regular,
                TextAlign::Center,
            );
            y += 17.0;
        }
        self.ui.hit_region(
            ACHIEVEMENT_BASE + index as u16,
            frame.rect(centre[0] - CELL * 0.5, top, CELL - 4.0, RADIUS * 2.0 + 52.0),
        );
    }

    /// The achievement chosen, up close on the right.
    fn achievement_up_close(&mut self, frame: &Frame, standing: &Standing) {
        let s = frame.s;
        let kind = standing.kind;
        let done = standing.unlocked.is_some();
        let hue = tint(kind.category);
        let middle = SHOWCASE_X + SHOWCASE_WIDTH * 0.5;
        let centre = [middle, 226.0 + BIG_RADIUS + 8.0];
        glow(
            &mut self.ui,
            frame,
            centre,
            BIG_RADIUS * 2.0,
            if done { color::GOLD } else { hue },
            if done { 0.32 } else { 0.16 },
        );
        medallion::draw(
            &mut self.ui,
            Medallion {
                kind,
                centre: frame.point(centre[0], centre[1]),
                radius: BIG_RADIUS * s,
                fraction: standing.fraction(),
                done,
            },
        );
        let mut y = centre[1] + BIG_RADIUS + 28.0;
        text(
            &mut self.ui,
            TextFamily::Display,
            format_args!("{}", standing.name()),
            frame.rect(SHOWCASE_X, y, SHOWCASE_WIDTH, 56.0),
            50.0 * s,
            if done {
                color::GOLD_BRIGHT
            } else {
                color::TEXT
            },
            FontWeight::Semibold,
            TextAlign::Center,
        );
        y += 60.0;
        let retired = if kind.retired {
            crate::achievements::RETIRED_NOTE.trim_end_matches('.')
        } else {
            ""
        };
        let dot = if kind.retired { " \u{b7} " } else { "" };
        text(
            &mut self.ui,
            TextFamily::Body,
            format_args!("{}{dot}{retired}", kind.category.name()),
            frame.rect(SHOWCASE_X, y, SHOWCASE_WIDTH, 22.0),
            16.0 * s,
            hue,
            FontWeight::Semibold,
            TextAlign::Center,
        );
        y += 30.0;
        y = centred_lines(
            &mut self.ui,
            frame,
            &standing.description(),
            [SHOWCASE_X, y, SHOWCASE_WIDTH],
            56,
            2,
            20.0,
            color::TEXT,
        );
        y += 12.0;
        let width = 520.0;
        bar(
            &mut self.ui,
            frame,
            [middle - width * 0.5, y, width, 6.0],
            standing.fraction(),
            if done { color::GOLD } else { hue },
        );
        // A secret says nothing of its goal or how far the player is.
        let progress = if standing.secret() {
            String::new()
        } else {
            format!(
                "{} / {}",
                kind.amount(standing.progress.min(kind.goal)),
                kind.amount(kind.goal)
            )
        };
        let status = match standing.unlocked {
            Some(at) if at > 0 => format!("Unlocked {}", crate::medals::date_text(at)),
            Some(_) => "Unlocked".to_owned(),
            None if standing.secret() => "Not unlocked yet".to_owned(),
            None if standing.waiting => crate::achievements::WAITING_NOTE.to_owned(),
            None => {
                let left = kind.goal.saturating_sub(standing.progress);
                format!("{} more to go", kind.amount(left))
            }
        };
        text(
            &mut self.ui,
            TextFamily::Body,
            format_args!("{progress}"),
            frame.rect(middle - width * 0.5, y + 14.0, width * 0.5, 22.0),
            16.0 * s,
            color::MUTED,
            FontWeight::Regular,
            TextAlign::Start,
        );
        text(
            &mut self.ui,
            TextFamily::Body,
            format_args!("{status}"),
            frame.rect(middle, y + 14.0, width * 0.5, 22.0),
            16.0 * s,
            if done {
                color::GOLD_BRIGHT
            } else {
                color::MUTED
            },
            FontWeight::Regular,
            TextAlign::End,
        ); // The kyber crystal it brings (21/10/2026), its gem before it in its colour.
        if let Some(tier) = crate::kyber::reward(kind.id) {
            let words = crate::kyber::reward_line(tier);
            let size = 20.0;
            let wide =
                crate::text::display_width(words, size) * crate::text::style::current().scale;
            let left = middle - (wide + 26.0) * 0.5;
            let [gx, gy] = frame.point(left + 9.0, y + 56.0);
            crate::holocrons::gem::draw(
                self.ui.draw_list_mut(),
                [gx, gy],
                9.0 * s,
                tier.colour,
                if done { 1.0 } else { 0.8 },
                crate::holocrons::gem::ROWS,
            );
            text(
                &mut self.ui,
                TextFamily::Display,
                format_args!("{words}"),
                frame.rect(left + 26.0, y + 43.0, wide + 20.0, 26.0),
                size * s,
                tier.colour_alpha(if done { 1.0 } else { 0.85 }),
                FontWeight::Semibold,
                TextAlign::Start,
            );
        }
    }

    /// The locked achievements nearest to unlocking, under the one up close.
    fn next_up(&mut self, frame: &Frame, standings: &[Standing]) {
        let s = frame.s;
        let width = 560.0;
        let x = SHOWCASE_X + (SHOWCASE_WIDTH - width) * 0.5;
        let near = next_up(standings, self.achievement);
        if near.is_empty() {
            return;
        }
        let top = 762.0;
        kit::heading(&mut self.ui, frame, x, top, width, "Next up");
        for (row, index) in near.into_iter().enumerate() {
            let standing = &standings[index];
            let kind = standing.kind;
            let hue = tint(kind.category);
            let y = top + 34.0 + row as f32 * 50.0;
            medallion::draw(
                &mut self.ui,
                Medallion {
                    kind,
                    centre: frame.point(x + 20.0, y + 20.0),
                    radius: 19.0 * s,
                    fraction: standing.fraction(),
                    done: false,
                },
            );
            text(
                &mut self.ui,
                TextFamily::Display,
                format_args!("{}", crate::terms::say(kind.name)),
                frame.rect(x + 54.0, y + 6.0, 260.0, 28.0),
                22.0 * s,
                color::TEXT,
                FontWeight::Semibold,
                TextAlign::Start,
            );
            bar(
                &mut self.ui,
                frame,
                [x + 320.0, y + 18.0, 150.0, 4.0],
                standing.fraction(),
                hue,
            );
            text(
                &mut self.ui,
                TextFamily::Body,
                format_args!(
                    "{} / {}",
                    kind.amount(standing.progress),
                    kind.amount(kind.goal)
                ),
                frame.rect(x + 470.0, y + 8.0, 90.0, 22.0),
                15.0 * s,
                color::MUTED,
                FontWeight::Regular,
                TextAlign::End,
            );
        }
    }
}

/// A progress bar over `rect` (frame pixels), filled to `fraction` in `fill`.
fn bar(
    canvas: &mut crate::menu_widgets::MenuCanvas,
    frame: &Frame,
    rect: [f32; 4],
    fraction: f32,
    fill: sjk_ui::Color,
) {
    let [x, y, width, height] = rect;
    let s = frame.s;
    let _ = canvas.draw_list_mut().push(DrawCommand::RoundedRect {
        rect: frame.rect(x, y, width, height),
        radius: height * 0.5 * s,
        color: color::alpha(color::HOLO, 0.14),
    });
    let filled = width * fraction.clamp(0.0, 1.0);
    if filled > 0.5 {
        let _ = canvas.draw_list_mut().push(DrawCommand::RoundedRect {
            rect: frame.rect(x, y, filled, height),
            radius: height * 0.5 * s,
            color: fill,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index_of(id: &str) -> usize {
        crate::achievements::ALL
            .iter()
            .position(|kind| kind.id == id)
            .expect("an achievement")
    }

    /// Standings with `held` unlocked and nothing else.
    fn standings_holding(held: &[&str]) -> Vec<Standing> {
        let held: Vec<sjk_identity::Achievement> = held
            .iter()
            .map(|id| sjk_identity::Achievement {
                id: (*id).to_owned(),
                progress: 1,
                goal: 1,
                unlocked: 1_791_336_225,
                reward: None,
            })
            .collect();
        crate::achievements::Record::default().standings(&held)
    }

    /// The six categories fit five bands of twelve: Combat alone, Duels and flags alone
    /// (with the five style masters), Journeys beside Community, Collection alone, Toys
    /// alone (with the five Fishing ones);
    /// a retired achievement not held is left out and one held takes its place.
    #[test]
    fn the_wall_packs_the_categories_into_bands() {
        let standings = standings_holding(&[]);
        let groups = groups(&standings);
        let layout: Vec<(&str, usize, usize, usize)> = groups
            .iter()
            .map(|g| (g.category.name(), g.band, g.column, g.members.len()))
            .collect();
        assert_eq!(
            layout,
            [
                ("Combat", 0, 0, 11),
                ("Duels and flags", 1, 0, 11),
                ("Journeys", 2, 0, 5),
                ("Community", 2, 6, 3),
                ("Collection", 3, 0, 7),
                ("Toys", 4, 0, 9),
            ]
        );
        let slots: Vec<Slot> = slots(&groups).collect();
        assert_eq!(slots.len(), crate::achievements::ALL.len() - 1);
        assert!(!slots.iter().any(|slot| slot.index == index_of("surveyor")));
        assert!(slots.iter().all(|slot| slot.column < COLUMNS));
        let holder = standings_holding(&["surveyor"]);
        let slots: Vec<Slot> = slots_of(&holder);
        assert_eq!(slots.len(), crate::achievements::ALL.len());
        assert!(slots.iter().any(|slot| slot.index == index_of("surveyor")));
    }

    fn slots_of(standings: &[Standing]) -> Vec<Slot> {
        slots(&groups(standings)).collect()
    }

    #[test]
    fn up_and_down_move_between_the_bands_near_the_column() {
        let slots = slots_of(&standings_holding(&[]));
        let at = |index: usize| *slots.iter().find(|slot| slot.index == index).unwrap();
        // Down from Arsenal (Combat's ninth) lands on Journeys's below it.
        let arsenal = at(index_of("arsenal"));
        let below = at(vertical(&slots, arsenal.index, true));
        assert_eq!((below.band, below.column), (1, arsenal.column));
        assert_eq!(vertical(&slots, below.index, false), arsenal.index);
        // Up from the first band and Down from the last stay.
        assert_eq!(vertical(&slots, arsenal.index, false), arsenal.index);
        let toys = index_of("box_minutes_10");
        assert_eq!(vertical(&slots, toys, true), toys);
        // Left and right read the wall in order, Tab wraps round.
        let first = slots[0].index;
        let last = slots.last().unwrap().index;
        assert_eq!(sideways(&slots, first, -1, false), first);
        assert_eq!(sideways(&slots, first, -1, true), last);
        assert_eq!(sideways(&slots, last, 1, true), first);
        assert_eq!(
            sideways(&slots, index_of("arsenal"), 1, false),
            index_of("saber_throw_far"),
            "Combat's newest follow on its band"
        );
        // An index not on the wall (a retired one) goes to the first.
        assert_eq!(vertical(&slots, index_of("surveyor"), true), first);
        assert_eq!(sideways(&slots, index_of("surveyor"), 1, false), first);
    }

    /// A secret is never offered as the next to work towards, nor a retired one.
    #[test]
    fn secrets_and_retired_ones_are_not_next_up() {
        let standings = standings_holding(&[]);
        let secret = index_of("mythical_find");
        assert!(standings[secret].secret());
        assert_ne!(first_choice(&standings), secret);
        for chosen in 0..standings.len() {
            let near = next_up(&standings, chosen);
            assert!(!near.contains(&secret));
            assert!(!near.contains(&index_of("surveyor")));
        }
    }

    #[test]
    fn the_board_opens_on_the_nearest_to_unlocking_and_lists_the_next() {
        let held = [sjk_identity::Achievement {
            id: "first_blood".into(),
            progress: 1,
            goal: 1,
            unlocked: 1_791_336_225,
            reward: None,
        }];
        let standings = crate::achievements::standings(&held);
        let chosen = first_choice(&standings);
        assert!(standings[chosen].unlocked.is_none());
        let near = next_up(&standings, chosen);
        assert!(near.len() <= NEXT_UP && !near.contains(&chosen));
        assert!(
            near.iter()
                .all(|index| standings[*index].unlocked.is_none())
        );
    }

    /// Every medallion listed stands in the canvas over the keys and answers the pointer,
    /// at 1080 lines, 4K, 4:3 and 21:9, and a retired one not held has no place.
    #[test]
    fn the_wall_fits_and_answers_the_pointer() {
        let standings = crate::achievements::standings(&[]);
        let inputs = Inputs {
            standings: &standings,
            ..super::super::tests::inputs(Holdings::Known(&[]), "")
        };
        let font = crate::text::load_modern(1.0, None).expect("Inter").font;
        for viewport in [
            [1_920.0, 1_080.0],
            [3_840.0, 2_160.0],
            [1_440.0, 1_080.0],
            [2_560.0, 1_080.0],
        ] {
            let mut panel = Panel::new();
            panel.open(Tab::Achievements, true, true, ReturnTarget::InGame);
            panel.build(&inputs, &font, viewport);
            assert!(!panel.ui.overflowed(), "{viewport:?}");
            let frame = Frame::new(viewport);
            let keys = frame.point(0.0, super::super::view::KEYS_Y)[1];
            for index in (0..standings.len()).filter(|index| standings[*index].listed()) {
                let rect = panel
                    .ui
                    .rect_for(ACHIEVEMENT_BASE + index as u16)
                    .expect("a medallion's area");
                assert!(rect.bottom() <= keys, "{index} {viewport:?}");
                assert!(rect.right() <= frame.point(SHOWCASE_X, 0.0)[0]);
            }
            let surveyor = index_of("surveyor") as u16;
            assert!(panel.ui.rect_for(ACHIEVEMENT_BASE + surveyor).is_none());
            for command in panel.ui.draw_list().commands() {
                if let DrawCommand::Text { rect, .. } = command {
                    assert!(rect.bottom() <= keys || rect.y >= keys, "{rect:?}");
                }
            }
        }
    }
}
