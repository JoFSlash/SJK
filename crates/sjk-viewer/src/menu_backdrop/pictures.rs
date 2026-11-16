//! The backdrop in picture mode (`ui_menuPictures`, [`crate::menu_pictures`]): the
//! menu world is empty, and each view the camera would fly to is a picture taken
//! from that view beforehand. The camera stands exactly where its picture was
//! taken, without drift, so what is still drawn in 3D over it (the player on the
//! stage, the thrown saber, the holocron) lines up with the picture. Moving to
//! another view cuts to its picture instead of flying (the painter crossfades,
//! `menu_pictures_paint.rs`); the main menu of a map with a camera tour shows the
//! tour's pictures one after another.

use super::{Sample, Shot, Vantage, tour};

/// One view a picture is taken from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum View {
    /// The map's intermission view, behind the main menu of a map without a tour.
    Main,
    /// A camera tour's shot, halfway through its glide.
    Tour(usize),
    /// Where the flight to one of the other shots ends.
    Shot(Shot),
}

impl View {
    /// The picture's file and manifest name: `main`, `tour-3`, `player`.
    pub(crate) fn name(self) -> String {
        match self {
            Self::Main => "main".to_owned(),
            Self::Tour(index) => format!("tour-{index}"),
            Self::Shot(shot) => shot_name(shot).to_owned(),
        }
    }

    /// The view `name` names.
    pub(crate) fn parse(name: &str) -> Option<Self> {
        if name == "main" {
            return Some(Self::Main);
        }
        if let Some(index) = name.strip_prefix("tour-") {
            return index.parse().ok().map(Self::Tour);
        }
        SHOTS
            .iter()
            .find(|(_, shot_name)| *shot_name == name)
            .map(|(shot, _)| Self::Shot(*shot))
    }
}

const SHOTS: [(Shot, &str); 5] = [
    (Shot::Browser, "browser"),
    (Shot::Settings, "settings"),
    (Shot::Player, "player"),
    (Shot::Saber, "saber"),
    (Shot::Holocrons, "holocrons"),
];

fn shot_name(shot: Shot) -> &'static str {
    SHOTS
        .iter()
        .find(|(candidate, _)| *candidate == shot)
        .map_or("main", |(_, name)| name)
}

#[cfg(test)]
/// The vantage a tour shot's picture is taken from: halfway through its glide,
/// looking at its point.
pub(super) fn tour_vantage(shot: &super::routes::TourShot) -> Vantage {
    use glam::Vec3;
    let origin = Vec3::from_array(shot.from).lerp(Vec3::from_array(shot.to), 0.5);
    let direction = (Vec3::from_array(shot.at) - origin).normalize_or(Vec3::X);
    Vantage {
        origin,
        yaw: direction.y.atan2(direction.x),
        pitch: direction.z.clamp(-1.0, 1.0).asin(),
    }
}

/// The picture state behind the menus: the set's views and which one is on show.
pub(crate) struct Pictures {
    views: Vec<(View, Vantage)>,
    tour_shots: usize,
    current: Option<usize>,
    tour_began: Option<u64>,
}

impl Pictures {
    pub(super) fn new(views: Vec<(View, Vantage)>) -> Self {
        let tour_shots = views
            .iter()
            .filter(|(view, _)| matches!(view, View::Tour(_)))
            .count();
        Self {
            views,
            tour_shots,
            current: None,
            tour_began: None,
        }
    }

    fn index_of(&self, view: View) -> Option<usize> {
        self.views
            .iter()
            .position(|(candidate, _)| *candidate == view)
    }

    /// Show the picture of `view` at backdrop time `millis` (the main menu's being the
    /// tour, when the set has one) and return the camera, standing where it was taken.
    pub(super) fn drive(&mut self, view: View, millis: u64) -> Option<Sample> {
        let wanted = if view == View::Main && self.tour_shots > 0 {
            let began = *self.tour_began.get_or_insert(millis);
            let shot =
                (millis.saturating_sub(began) / tour::SHOT_MILLIS) as usize % self.tour_shots;
            self.index_of(View::Tour(shot))
        } else {
            self.index_of(view)
        }
        .or_else(|| self.index_of(View::Main))
        .or_else(|| self.index_of(View::Tour(0)))?;
        self.current = Some(wanted);
        let vantage = self.views[wanted].1;
        Some(Sample {
            origin: vantage.origin,
            yaw: vantage.yaw,
            pitch: vantage.pitch,
        })
    }

    /// The picture on show as of the last [`Self::drive`], by its index.
    pub(crate) fn shown(&self) -> Option<usize> {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    fn vantage(x: f32) -> Vantage {
        Vantage {
            origin: Vec3::new(x, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    #[test]
    fn names_read_back() {
        for view in [
            View::Main,
            View::Tour(0),
            View::Tour(9),
            View::Shot(Shot::Player),
            View::Shot(Shot::Holocrons),
        ] {
            assert_eq!(View::parse(&view.name()), Some(view));
        }
        assert_eq!(View::parse("tour-x"), None);
        assert_eq!(View::parse("nowhere"), None);
    }

    #[test]
    fn a_view_change_cuts_to_the_pictures_camera() {
        let mut pictures = Pictures::new(vec![
            (View::Main, vantage(1.0)),
            (View::Shot(Shot::Player), vantage(2.0)),
        ]);
        let sample = pictures.drive(View::Main, 0).expect("a camera");
        assert_eq!(sample.origin.x, 1.0);
        assert_eq!(pictures.shown(), Some(0));
        let sample = pictures
            .drive(View::Shot(Shot::Player), 1_000)
            .expect("a camera");
        assert_eq!(
            sample.origin.x, 2.0,
            "no flight: it cuts to the picture's camera"
        );
        assert_eq!(pictures.shown(), Some(1));
    }

    #[test]
    fn the_main_menu_plays_the_tour_and_a_view_without_a_picture_shows_main() {
        let mut pictures = Pictures::new(vec![
            (View::Tour(0), vantage(0.0)),
            (View::Tour(1), vantage(1.0)),
            (View::Shot(Shot::Player), vantage(5.0)),
        ]);
        let first = pictures.drive(View::Main, 0).expect("a camera");
        let second = pictures
            .drive(View::Main, tour::SHOT_MILLIS)
            .expect("a camera");
        let around = pictures
            .drive(View::Main, 2 * tour::SHOT_MILLIS)
            .expect("a camera");
        assert_eq!(
            [first.origin.x, second.origin.x, around.origin.x],
            [0.0, 1.0, 0.0]
        );
        // The browser has no picture of its own here: the tour's first stands in.
        let browser = pictures
            .drive(View::Shot(Shot::Browser), 3 * tour::SHOT_MILLIS)
            .expect("a camera");
        assert_eq!(browser.origin.x, 0.0);
    }
}
