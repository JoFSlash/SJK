//! `CG_DrawOldScoreboard` row placement at JoF EJK bd5e202d (NEW_SCOREBOARD=0).
//! Keep the reference's per-section budgets and cursor advancement, including
//! empty sections; a viewport clip is not a substitute for those decisions.
use super::*;

pub(super) fn plan(rows: &[ScoreRow], header: &ClassicHeader<'_>, small: bool) -> Plan {
    let scored = rows.iter().filter(|row| row.has_score).count();
    let crowded = header.context.connected >= MANY_CLIENTS;
    let size = RowSize::choose(scored, crowded, small, header.gametype);
    let line = size.height();
    let (top_border, bottom_border) = size.borders();
    let compact = crowded && header.context.compact();
    let max_rows = if compact {
        24
    } else {
        match size {
            RowSize::Large => 12,
            RowSize::Small => 19,
            RowSize::Many => 32,
        }
    };
    let mut out = Plan {
        size,
        crowded,
        placed: [(0, 0.0); 33],
        len: 0,
        bands: [(0, 0.0, 0.0); 2],
        band_len: 0,
        local_appended: false,
        hint_y: None,
    };
    let local = rows
        .iter()
        .position(|row| row.has_score && u16::from(row.client_num) == header.local.client);
    let local_listed = |out: &Plan| {
        local.is_some_and(|index| {
            out.placed[..out.len]
                .iter()
                .any(|(row, _)| usize::from(*row) == index)
        })
    };
    let gap = if crowded { GROUP_GAP - 8.0 } else { GROUP_GAP };
    let mut y = if crowded { TOP_Y - 62.0 } else { TOP_Y };
    let mut remaining = max_rows;
    if header.gametype >= TEAM {
        y += line / 2.0;
        let red_leads = header.team_scores[0] >= header.team_scores[1];
        let teams = if red_leads { [RED, BLUE] } else { [BLUE, RED] };
        let count = |team| {
            rows.iter()
                .filter(|row| row.has_score && row.team == team)
                .count()
                .min(max_rows)
        };
        let mut lead_max = count(teams[0]);
        let losing = count(teams[1]);
        if lead_max > 10 && lead_max + losing > max_rows {
            lead_max = lead_max.saturating_sub(losing).max(10);
        }
        for (team, cap) in [(teams[0], lead_max), (teams[1], max_rows - lead_max)] {
            let n = members(rows, team, cap, scored > 0).count();
            if compact && n > 0 && out.len > 0 {
                y += GROUP_GAP;
            }
            if n > 0 {
                out.bands[out.band_len] = (team, y - top_border, n as f32 * line + bottom_border);
                out.band_len += 1;
            }
            append(&mut out, rows, team, cap, scored > 0, y);
            y += n as f32 * line;
            remaining = remaining.saturating_sub(n);
            if !compact {
                // The reference's red-leading branch keeps BIGCHAR_HEIGHT;
                // its blue-leading branch uses the crowded section spacing.
                y += if red_leads { GROUP_GAP } else { gap };
            }
        }
        // A populated free team is supported generically; no server-name branch.
        let free = members(rows, FREE, remaining, scored > 0).count();
        if free > 0 {
            if compact && out.len > 0 {
                y += gap;
            }
            let cap = if compact { remaining } else { max_rows };
            let n = append(&mut out, rows, FREE, cap, scored > 0, y);
            y += n as f32 * line;
            remaining = remaining.saturating_sub(n);
            if !compact {
                y += gap;
            }
        }
        let reserved = 1 + usize::from(
            !local_listed(&out) && local.is_some_and(|i| rows[i].team != SPECTATOR),
        );
        let cap = if compact {
            remaining.saturating_sub(reserved)
        } else {
            max_rows
        };
        let n = members(rows, SPECTATOR, cap, scored > 0).count();
        if compact && n > 0 && out.len > 0 {
            y += gap;
        }
        append(&mut out, rows, SPECTATOR, cap, scored > 0, y);
        y += n as f32 * line;
        if !compact {
            y += gap;
        }
    } else {
        let n = append(&mut out, rows, FREE, max_rows, scored > 0, y);
        y += n as f32 * line;
        let reserved = 1 + usize::from(
            !local_listed(&out) && local.is_some_and(|i| rows[i].team != SPECTATOR),
        );
        let cap = if compact {
            max_rows.saturating_sub(n + reserved)
        } else if crowded {
            32
        } else {
            max_rows.saturating_sub(n)
        };
        if !compact || (n > 0 && members(rows, SPECTATOR, cap, scored > 0).next().is_some()) {
            y += gap;
        }
        let n = append(&mut out, rows, SPECTATOR, cap, scored > 0, y);
        y += n as f32 * line;
        if !compact {
            y += gap;
        }
    }
    let append_local = if compact {
        local.is_some_and(|i| rows[i].team != SPECTATOR && rows[i].team != BLUE)
    } else {
        !crowded && !header.local.intermission && !header.context.dead
    };
    if !local_listed(&out)
        && append_local
        && let Some(index) = local
    {
        out.placed[out.len] = (index as u8, y);
        out.len += 1;
        out.local_appended = true;
        if compact {
            y += line;
        }
    }
    if compact {
        out.hint_y = Some(y);
    }
    out
}

fn members(
    rows: &[ScoreRow],
    team: u8,
    cap: usize,
    has_scores: bool,
) -> impl Iterator<Item = usize> + '_ {
    // CG_TeamScoreboard draws real scores, then unlisted client slots in slot order.
    rows.iter()
        .enumerate()
        .filter(move |(_, row)| row.team == team && row.has_score)
        .chain(
            rows.iter()
                .enumerate()
                .filter(move |(_, row)| has_scores && row.team == team && !row.has_score),
        )
        .map(|(index, _)| index)
        .take(cap)
}

fn append(
    out: &mut Plan,
    rows: &[ScoreRow],
    team: u8,
    cap: usize,
    has_scores: bool,
    y: f32,
) -> usize {
    let start = out.len;
    for index in members(rows, team, cap, has_scores) {
        if out.len == 32 {
            break;
        }
        out.placed[out.len] = (
            index as u8,
            y + (out.len - start) as f32 * out.size.height(),
        );
        out.len += 1;
    }
    out.len - start
}
