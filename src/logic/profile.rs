//! Profiles without a browser: the name to show for an account, its
//! counts, what an edit of the profile sends, the checks of a picture
//! before it goes, and the square the crop picker cuts from it as the
//! picture is dragged and zoomed under the circle.

use starbooru_core::profile::{
    MAX_ABOUT_CHARS, MAX_AVATAR_BYTES, MAX_DISPLAY_NAME_CHARS, MIN_AVATAR_SIDE,
};
use starbooru_core::{
    AvatarCrop, Profile, ProfileError, ProfileUpdate, clean_about, clean_display_name,
};

use super::settings::initial;
use super::upload::human_size;

/// The name an account goes by: its display name, or else its username.
pub fn shown_name(name: &str, display: Option<&str>) -> String {
    display
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .unwrap_or(name)
        .to_owned()
}

/// The letter on an account's disc when it has no picture: the display
/// name's first, or the username's.
pub fn badge(name: &str, display: Option<&str>) -> String {
    initial(&shown_name(name, display))
}

/// The display name as typed: cleaned as the server cleans it, empty
/// meaning none.
pub fn check_name(typed: &str) -> Result<Option<String>, ProfileError> {
    clean_display_name(typed)
}

/// The text about the account as typed, its lines kept.
pub fn check_about(typed: &str) -> Result<Option<String>, ProfileError> {
    clean_about(typed)
}

pub fn error_text(e: &ProfileError) -> String {
    match e {
        ProfileError::NameTooLong => t!("profile-name-long", max = MAX_DISPLAY_NAME_CHARS),
        ProfileError::NameControlChar | ProfileError::AboutControlChar => {
            t!("profile-bad-character")
        }
        ProfileError::AboutTooLong => t!("profile-about-long", max = MAX_ABOUT_CHARS),
        ProfileError::PictureTooSmall => t!("profile-picture-small", min = MIN_AVATAR_SIDE),
        ProfileError::CropOutside => t!("profile-picture-outside"),
    }
}

/// What saving the profile's form sends: only what differs from `saved`, a
/// field emptied clearing it. `None` when nothing changed or something
/// typed can't be saved. The text about the account is a public server's
/// alone; `about` is `None` elsewhere.
pub fn update(saved: &Profile, name: &str, about: Option<&str>) -> Option<ProfileUpdate> {
    let name = check_name(name).ok()?;
    let about = about.map(check_about).transpose().ok()?;
    let display_name = (name != saved.display_name).then_some(name);
    let about = about.filter(|a| *a != saved.about);
    (display_name.is_some() || about.is_some()).then_some(ProfileUpdate {
        display_name,
        about,
    })
}

/// What's wrong with a picture before it's sent, so nothing is uploaded
/// for the server to turn down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PictureProblem {
    TooBig,
    NotAPicture,
}

/// The kinds of picture the server takes, by the type the browser gives.
const TAKEN: [&str; 4] = ["image/jpeg", "image/png", "image/webp", "image/gif"];

pub fn check_picture(bytes: f64, mime: &str) -> Result<(), PictureProblem> {
    let mime = mime.trim().to_ascii_lowercase();
    // A browser that can't tell leaves the type empty: the server reads it.
    if !mime.is_empty() && !TAKEN.contains(&mime.as_str()) {
        return Err(PictureProblem::NotAPicture);
    }
    if !(0.0..=MAX_AVATAR_BYTES as f64).contains(&bytes) {
        return Err(PictureProblem::TooBig);
    }
    Ok(())
}

pub fn picture_problem_text(p: PictureProblem) -> String {
    match p {
        PictureProblem::TooBig => t!(
            "profile-picture-big",
            max = human_size(MAX_AVATAR_BYTES as u64)
        ),
        PictureProblem::NotAPicture => t!("profile-picture-kind"),
    }
}

// ---------- the crop picker ----------

/// How far in the picker zooms, at most: eight times the square that
/// covers the circle, and never past the server's smallest square.
const MAX_ZOOM: f64 = 8.0;
/// An arrow moves the picture this share of the square; with Shift, more.
const STEP: f64 = 0.04;
const BIG_STEP: f64 = 0.2;
/// A press of + or − zooms by this much.
const ZOOM_STEP: f64 = 1.25;

/// The square cut from a picture of `width` × `height` pixels, as the
/// picker shows it: the middle of the square, in the picture's pixels, and
/// how far in it is, 1 being the largest square the picture holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crop {
    pub width: f64,
    pub height: f64,
    pub zoom: f64,
    pub cx: f64,
    pub cy: f64,
}

impl Crop {
    /// The largest square in the middle, as the server cuts by default.
    /// `None` for a picture smaller than the server takes.
    pub fn new(width: f64, height: f64) -> Option<Crop> {
        let fine = |v: f64| v.is_finite() && v >= f64::from(MIN_AVATAR_SIDE);
        (fine(width) && fine(height)).then_some(Crop {
            width,
            height,
            zoom: 1.0,
            cx: width / 2.0,
            cy: height / 2.0,
        })
    }

    /// The square's side, in the picture's pixels.
    pub fn side(&self) -> f64 {
        self.width.min(self.height) / self.zoom
    }

    fn max_zoom(&self) -> f64 {
        (self.width.min(self.height) / f64::from(MIN_AVATAR_SIDE)).clamp(1.0, MAX_ZOOM)
    }

    /// Keeps the square inside the picture.
    fn clamped(mut self) -> Crop {
        self.zoom = if self.zoom.is_finite() {
            self.zoom.clamp(1.0, self.max_zoom())
        } else {
            1.0
        };
        let half = self.side() / 2.0;
        let keep = |v: f64, room: f64| {
            if v.is_finite() {
                v.clamp(half, (room - half).max(half))
            } else {
                room / 2.0
            }
        };
        self.cx = keep(self.cx, self.width);
        self.cy = keep(self.cy, self.height);
        self
    }

    /// The picture dragged by `dx`, `dy` CSS pixels in a frame `view`
    /// pixels wide: the square moves the other way under it.
    pub fn dragged(self, dx: f64, dy: f64, view: f64) -> Crop {
        if !(view.is_finite() && view > 0.0) {
            return self;
        }
        let per = self.side() / view;
        Crop {
            cx: self.cx - dx * per,
            cy: self.cy - dy * per,
            ..self
        }
        .clamped()
    }

    /// Zoomed to `zoom`, the point of the frame at `ax`, `ay` (0 to 1 across
    /// it) staying where it is, as under a finger or the wheel.
    pub fn zoomed_at(self, zoom: f64, ax: f64, ay: f64) -> Crop {
        let before = self.side();
        let after = Crop { zoom, ..self }.clamped();
        let (ax, ay) = (ax.clamp(0.0, 1.0) - 0.5, ay.clamp(0.0, 1.0) - 0.5);
        Crop {
            cx: self.cx + ax * (before - after.side()),
            cy: self.cy + ay * (before - after.side()),
            ..after
        }
        .clamped()
    }

    /// Zoomed to `zoom` about the middle, as by the slider.
    pub fn zoomed(self, zoom: f64) -> Crop {
        self.zoomed_at(zoom, 0.5, 0.5)
    }

    /// What a key does: the arrows move the picture as a drag does, + and −
    /// zoom, 0 goes back to the whole square. `None` for another key.
    pub fn keyed(self, key: &str, shift: bool) -> Option<Crop> {
        let step = if shift { BIG_STEP } else { STEP } * self.side();
        let moved = |dx: f64, dy: f64| {
            Crop {
                cx: self.cx - dx * step,
                cy: self.cy - dy * step,
                ..self
            }
            .clamped()
        };
        Some(match key {
            "ArrowLeft" => moved(-1.0, 0.0),
            "ArrowRight" => moved(1.0, 0.0),
            "ArrowUp" => moved(0.0, -1.0),
            "ArrowDown" => moved(0.0, 1.0),
            "+" | "=" => self.zoomed(self.zoom * ZOOM_STEP),
            "-" | "_" => self.zoomed(self.zoom / ZOOM_STEP),
            "0" => Crop::new(self.width, self.height)?,
            _ => return None,
        })
    }

    /// Where the picture lies in the frame, as shares of the frame's side
    /// in percent: left, top, width and height.
    pub fn placement(&self) -> (f64, f64, f64, f64) {
        let side = self.side();
        (
            (0.5 - self.cx / side) * 100.0,
            (0.5 - self.cy / side) * 100.0,
            self.width / side * 100.0,
            self.height / side * 100.0,
        )
    }

    /// The square for `PUT /auth/avatar`, in whole pixels of the picture,
    /// inside it.
    pub fn square(&self) -> AvatarCrop {
        let (w, h) = (self.width.floor(), self.height.floor());
        let size = self
            .side()
            .floor()
            .clamp(f64::from(MIN_AVATAR_SIDE), w.min(h).max(1.0));
        // Down, as the server halves the room left over.
        let at = |middle: f64, room: f64| {
            (middle - size / 2.0 + 1e-6)
                .floor()
                .clamp(0.0, (room - size).max(0.0))
        };
        AvatarCrop {
            x: at(self.cx, w) as u32,
            y: at(self.cy, h) as u32,
            size: size as u32,
        }
    }

    /// The zoom as a share of the way to the most, 0 to 1, for the slider.
    pub fn zoom_share(&self) -> f64 {
        let most = self.max_zoom();
        if most <= 1.0 {
            0.0
        } else {
            (self.zoom.ln() / most.ln()).clamp(0.0, 1.0)
        }
    }

    /// The zoom at `share` of the slider.
    pub fn zoom_of_share(&self, share: f64) -> f64 {
        self.max_zoom().powf(share.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use starbooru_core::{Role, avatar_square};

    use super::*;

    fn profile() -> Profile {
        Profile {
            name: "ann".into(),
            display_name: Some("Ann".into()),
            about: Some("I tag skies.".into()),
            avatar_url: None,
            avatar_small_url: None,
            role: Role::User,
            joined: DateTime::from_timestamp(1_800_000_000, 0).unwrap(),
            posts: 12,
            forum_posts: 1,
        }
    }

    #[test]
    fn an_account_goes_by_its_display_name() {
        assert_eq!(shown_name("ann", Some("Ann B")), "Ann B");
        assert_eq!(shown_name("ann", Some("  ")), "ann");
        assert_eq!(shown_name("ann", None), "ann");
        assert_eq!(badge("ann", Some("zoë")), "Z");
        assert_eq!(badge("_x", None), "X");
    }

    #[test]
    fn saving_sends_only_what_changed() {
        let saved = profile();
        assert_eq!(update(&saved, " Ann ", Some("I tag skies.")), None);
        let renamed = update(&saved, "Ann  B", Some("I tag skies.")).unwrap();
        assert_eq!(renamed.display_name, Some(Some("Ann B".into())));
        assert_eq!(renamed.about, None);
        // Emptied, a field is cleared.
        let cleared = update(&saved, "", Some(" \n ")).unwrap();
        assert_eq!(cleared.display_name, Some(None));
        assert_eq!(cleared.about, Some(None));
        // A private server has no text about the account to send.
        let private = update(&saved, "Ann", None);
        assert_eq!(private, None);
        // What the server would refuse isn't sent.
        assert_eq!(update(&saved, &"x".repeat(41), None), None);
        assert_eq!(update(&saved, "Ann", Some(&"x".repeat(501))), None);
        assert_eq!(update(&saved, "ad\u{202E}min", None), None);
        assert!(error_text(&ProfileError::NameTooLong).contains("40"));
        assert!(error_text(&ProfileError::AboutTooLong).contains("500"));
    }

    #[test]
    fn a_picture_is_checked_before_it_goes() {
        assert_eq!(check_picture(1000.0, "image/png"), Ok(()));
        assert_eq!(check_picture(1000.0, "IMAGE/JPEG"), Ok(()));
        assert_eq!(check_picture(1000.0, ""), Ok(()));
        assert_eq!(
            check_picture((5 << 20) as f64 + 1.0, "image/webp"),
            Err(PictureProblem::TooBig)
        );
        assert_eq!(check_picture((5 << 20) as f64, "image/gif"), Ok(()));
        assert_eq!(
            check_picture(10.0, "video/mp4"),
            Err(PictureProblem::NotAPicture)
        );
        assert_eq!(
            check_picture(10.0, "image/svg+xml"),
            Err(PictureProblem::NotAPicture)
        );
        assert!(picture_problem_text(PictureProblem::TooBig).contains("5.0 MB"));
    }

    #[test]
    fn the_picker_starts_on_the_servers_own_square() {
        let crop = Crop::new(300.0, 100.0).unwrap();
        assert_eq!(crop.square(), avatar_square(300, 100, None).unwrap());
        let tall = Crop::new(100.0, 301.0).unwrap();
        assert_eq!(tall.square(), avatar_square(100, 301, None).unwrap());
        assert_eq!(Crop::new(15.0, 400.0), None);
        assert_eq!(Crop::new(f64::NAN, 400.0), None);
        // The picture covers the frame, its middle on the frame's.
        let (left, top, width, height) = crop.placement();
        assert_eq!((left, top, width, height), (-100.0, 0.0, 300.0, 100.0));
    }

    #[test]
    fn dragging_moves_the_picture_under_the_circle_and_no_further() {
        let crop = Crop::new(300.0, 100.0).unwrap();
        // The frame is 200 CSS pixels across: 2 per pixel of the picture.
        let right = crop.dragged(-100.0, 0.0, 200.0);
        assert_eq!(
            right.square(),
            AvatarCrop {
                x: 150,
                y: 0,
                size: 100
            }
        );
        // Past the edge it stops there.
        let far = crop.dragged(-10_000.0, 500.0, 200.0);
        assert_eq!(
            far.square(),
            AvatarCrop {
                x: 200,
                y: 0,
                size: 100
            }
        );
        let left = crop.dragged(10_000.0, 0.0, 200.0);
        assert_eq!(left.square().x, 0);
        // A frame not drawn yet moves nothing.
        assert_eq!(crop.dragged(50.0, 0.0, 0.0), crop);
    }

    #[test]
    fn zooming_keeps_the_point_under_the_finger() {
        let crop = Crop::new(400.0, 400.0).unwrap();
        let twice = crop.zoomed_at(2.0, 0.0, 0.0);
        // The frame's top left stays the picture's.
        assert_eq!(
            twice.square(),
            AvatarCrop {
                x: 0,
                y: 0,
                size: 200
            }
        );
        let middle = crop.zoomed(2.0);
        assert_eq!(
            middle.square(),
            AvatarCrop {
                x: 100,
                y: 100,
                size: 200
            }
        );
        // Never smaller than the server's smallest square, nor out of 1..8.
        let deep = crop.zoomed(1000.0);
        assert_eq!(deep.square().size, 50);
        let small = Crop::new(20.0, 20.0).unwrap().zoomed(4.0);
        assert!(small.square().size >= MIN_AVATAR_SIDE);
        assert_eq!(crop.zoomed(0.2).zoom, 1.0);
        assert_eq!(crop.zoomed(f64::NAN).zoom, 1.0);
        // Zooming out at an edge pulls the square back inside.
        let edge = crop
            .zoomed(4.0)
            .dragged(-10_000.0, -10_000.0, 100.0)
            .zoomed(1.0);
        assert_eq!(
            edge.square(),
            AvatarCrop {
                x: 0,
                y: 0,
                size: 400
            }
        );
    }

    #[test]
    fn the_keys_move_and_zoom() {
        let crop = Crop::new(400.0, 400.0).unwrap().zoomed(2.0);
        let right = crop.keyed("ArrowRight", false).unwrap();
        // The picture moves right: the square goes left, 4% of its side.
        assert_eq!(right.square().x, 92);
        let up = crop.keyed("ArrowUp", true).unwrap();
        assert_eq!(up.square().y, 140);
        let closer = crop.keyed("+", false).unwrap();
        assert!(closer.zoom > crop.zoom);
        let wider = crop.keyed("-", false).unwrap();
        assert!(wider.zoom < crop.zoom);
        assert_eq!(crop.keyed("0", false), Crop::new(400.0, 400.0));
        assert_eq!(crop.keyed("a", false), None);
    }

    #[test]
    fn every_square_is_one_the_server_takes() {
        let mut crop = Crop::new(1234.0, 567.0).unwrap();
        for (i, key) in [
            "+",
            "ArrowLeft",
            "+",
            "ArrowDown",
            "+",
            "ArrowRight",
            "-",
            "+",
        ]
        .iter()
        .cycle()
        .take(60)
        .enumerate()
        {
            crop = crop.keyed(key, i % 3 == 0).unwrap();
            let sq = crop.square();
            assert_eq!(avatar_square(1234, 567, Some(sq)), Ok(sq), "{crop:?}");
        }
    }

    #[test]
    fn the_slider_runs_from_the_whole_square_to_the_closest() {
        let crop = Crop::new(400.0, 400.0).unwrap();
        assert_eq!(crop.zoom_share(), 0.0);
        let most = crop.zoom_of_share(1.0);
        assert_eq!(most, 8.0);
        let half = crop.zoomed(crop.zoom_of_share(0.5));
        assert!((half.zoom_share() - 0.5).abs() < 1e-9);
        // A picture too small to zoom keeps the slider at the start.
        let tiny = Crop::new(16.0, 16.0).unwrap();
        assert_eq!(tiny.zoom_share(), 0.0);
        assert_eq!(tiny.zoom_of_share(1.0), 1.0);
    }
}
