//! Pins the saber toggle sound cap: a short burst plays, spam settles to one toggle
//! per interval, and one toggle's sounds share an answer.

use super::*;

/// Toggle every 50 ms (a server frame at `sv_fps 20`) for `count` toggles from
/// `start`, and return which ones played.
fn spam(limit: &mut SaberToggleLimit, client: u16, start: i32, count: i32) -> Vec<bool> {
    (0..count)
        .map(|index| limit.allow(client, start + index * 50))
        .collect()
}

#[test]
fn a_burst_plays_then_spam_settles_to_one_toggle_per_interval() {
    let mut limit = SaberToggleLimit::default();
    let played = spam(&mut limit, 0, 1_000, 40);
    assert!(played[..4].iter().all(|&played| played));
    assert!(!played[4]);
    // Two seconds of spam: the burst, then one every 500 ms.
    assert_eq!(played.iter().filter(|&&played| played).count(), 4 + 3);
}

#[test]
fn a_few_quick_toggles_all_play_and_the_budget_refills() {
    let mut limit = SaberToggleLimit::default();
    assert!(
        spam(&mut limit, 0, 1_000, 4)
            .into_iter()
            .all(|played| played)
    );
    assert!(!limit.allow(0, 1_200));
    // Two seconds later the whole burst is back.
    assert!(
        spam(&mut limit, 0, 3_200, 4)
            .into_iter()
            .all(|played| played)
    );
}

#[test]
fn one_toggles_sounds_share_an_answer_and_players_are_apart() {
    let mut limit = SaberToggleLimit::default();
    spam(&mut limit, 3, 1_000, 4);
    // Both hands of the next toggle are muted together.
    assert!(!limit.allow(3, 1_200));
    assert!(!limit.allow(3, 1_200));
    // Someone else's toggle is not.
    assert!(limit.allow(4, 1_200));
    assert!(limit.allow(4, 1_200));
    // NPCs are never capped.
    assert!(
        spam(&mut limit, 40, 1_000, 20)
            .into_iter()
            .all(|played| played)
    );
}

#[test]
fn time_going_back_starts_afresh() {
    let mut limit = SaberToggleLimit::default();
    spam(&mut limit, 0, 50_000, 10);
    assert!(
        spam(&mut limit, 0, 1_000, 4)
            .into_iter()
            .all(|played| played)
    );
}

#[test]
fn a_refund_gives_one_toggle_back_once() {
    let mut limit = SaberToggleLimit::default();
    spam(&mut limit, 0, 1_000, 4);
    assert!(!limit.allow(0, 1_200));
    // The toggle at 1_150 was a late copy: given back, once for both its hilts.
    limit.refund(0, 1_150);
    limit.refund(0, 1_150);
    assert!(limit.allow(0, 1_250));
    assert!(!limit.allow(0, 1_300));
    // NPCs have nothing to give back.
    limit.refund(40, 1_300);
}

/// A staff put away again and again over a JA+ back: each put-away's shutdown at
/// its reach pays, and its weapon switch copy, let through or not, is given back
/// when let through, since the reach already voiced that toggle.
#[test]
fn a_staff_put_away_over_and_over_is_capped_like_any_toggle() {
    // The same spam of plain toggles, one every 100 ms (a reach takes longer than that).
    let mut plain = SaberToggleLimit::default();
    let expected = (0..40)
        .filter(|toggle| plain.allow(0, 1_000 + toggle * 100))
        .count();
    for copy_delay in [-75, -25, 25, 75] {
        let mut limit = SaberToggleLimit::default();
        let mut heard = 0;
        for put_away in 0..40 {
            let time = 1_000 + put_away * 100;
            let (reach, copy) = (time, time + copy_delay);
            let mut moments = [(reach, true), (copy, false)];
            moments.sort();
            for (at, is_reach) in moments {
                let allowed = limit.allow(0, at);
                if is_reach {
                    heard += usize::from(allowed);
                } else if allowed {
                    limit.refund(0, at);
                }
            }
        }
        assert_eq!(heard, expected, "copy {copy_delay} ms after the reach");
    }
}
