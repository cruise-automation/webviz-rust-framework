use crate::*;

/// Indicates when to wrap the current line to a new line. See also [`Direction`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LineWrap {
    /// Never wrap to a new line.
    None,

    /// Wrap to a new line when the available width is exhausted.
    Overflow,
}
impl LineWrap {
    /// TODO(JP): Replace these with LineWrap::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: LineWrap = LineWrap::None;
}
impl Default for LineWrap {
    fn default() -> Self {
        LineWrap::DEFAULT
    }
}

/// Configure how a [`CxTurtle`] is going to walk, typically bounded by the
/// dimensions of a parent [`CxTurtle`].
#[derive(Copy, Clone, Debug)]
pub struct Layout {
    /// See [`Walk`].
    pub walk: Walk,
    /// See [`Padding`].
    pub padding: Padding,
    /// See [`Direction`].
    pub direction: Direction,
    /// See [`LineWrap`].
    pub line_wrap: LineWrap,
    /// Absolutely position by overriding the [`CxTurtle::origin`] with (0,0) instead of using the parent's
    /// current position.
    pub absolute: bool,
    /// Override the maximum size of the [`Window`]/[`Pass`]. Should typically
    /// not be used; instead set [`CxTurtle::width`] and [`CxTurtle::height`]
    /// through [`Layout::walk`].
    pub abs_size: Option<Vec2>,
}

impl Layout {
    /// TODO(JP): Replace these with Layout::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Layout = Layout {
        walk: Walk::DEFAULT,
        padding: Padding::DEFAULT,
        direction: Direction::DEFAULT,
        line_wrap: LineWrap::DEFAULT,
        absolute: false,
        abs_size: None,
    };

    pub fn abs_origin_zero() -> Self {
        Layout { absolute: true, ..Default::default() }
    }
}
impl Default for Layout {
    fn default() -> Self {
        Layout::DEFAULT
    }
}

/// Determines how a [`CxTurtle`] should walk. Can be applied to a new [`CxTurtle`]
/// through [`Layout::walk`], or directly to move an existing [`CxTurtle`] by
/// using [`Cx::walk_turtle`].
#[derive(Copy, Clone, Debug)]
pub struct Walk {
    pub width: Width,
    pub height: Height,
}

impl Walk {
    /// TODO(JP): Replace these with Align::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Walk = Walk { width: Width::DEFAULT, height: Height::DEFAULT };

    pub const fn wh(w: Width, h: Height) -> Self {
        Self { width: w, height: h }
    }
}
impl Default for Walk {
    fn default() -> Self {
        Walk::DEFAULT
    }
}

/// A margin that should be added around a [`Walk`].
///
/// TODO(JP): these values can be negative, which can be quite confusing, but we
/// seem to actually honor that in the turtle code. Might be good to look into that
/// and see if we should forbid that or not (we seem to never actually do that yet).
#[derive(Clone, Copy, Debug)]
pub struct Margin {
    pub l: f32,
    pub t: f32,
    pub r: f32,
    pub b: f32,
}
impl Margin {
    pub const ZERO: Margin = Margin { l: 0.0, t: 0.0, r: 0.0, b: 0.0 };

    /// TODO(JP): Replace these with Margin::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Margin = Margin::ZERO;

    pub const fn all(v: f32) -> Margin {
        Margin { l: v, t: v, r: v, b: v }
    }

    pub const fn left(v: f32) -> Margin {
        Margin { l: v, ..Margin::ZERO }
    }

    pub const fn top(v: f32) -> Margin {
        Margin { t: v, ..Margin::ZERO }
    }

    pub const fn right(v: f32) -> Margin {
        Margin { r: v, ..Margin::ZERO }
    }

    pub const fn bottom(v: f32) -> Margin {
        Margin { b: v, ..Margin::ZERO }
    }
}
impl Default for Margin {
    fn default() -> Self {
        Margin::DEFAULT
    }
}

/// Inner padding dimensions that should be applied on top of the width/height
/// from the parent [`CxTurtle`].
#[derive(Clone, Copy, Debug)]
pub struct Padding {
    pub l: f32,
    pub t: f32,
    pub r: f32,
    pub b: f32,
}
impl Padding {
    pub const ZERO: Padding = Padding { l: 0.0, t: 0.0, r: 0.0, b: 0.0 };

    /// TODO(JP): Replace these with Padding::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Padding = Padding::ZERO;

    pub const fn all(v: f32) -> Padding {
        Padding { l: v, t: v, r: v, b: v }
    }

    pub const fn left(v: f32) -> Padding {
        Padding { l: v, ..Padding::ZERO }
    }

    pub const fn top(v: f32) -> Padding {
        Padding { t: v, ..Padding::ZERO }
    }

    pub const fn right(v: f32) -> Padding {
        Padding { r: v, ..Padding::ZERO }
    }

    pub const fn bottom(v: f32) -> Padding {
        Padding { b: v, ..Padding::ZERO }
    }
}
impl Default for Padding {
    fn default() -> Self {
        Padding::DEFAULT
    }
}

/// The direction in which the [`CxTurtle`] should walk. It will typically walk
/// in a straight line in this direction. E.g. when walking to [`Direction::Right`],
/// it will only walk horizontally, not vertically, until it hits the [`CxTurtle::width`],
/// at which point it will wrap around using [`LineWrap`], based on the maximum
/// height of widgets that have been drawn so far, which is registered in
/// [`CxTurtle::biggest`].
///
/// TODO(JP): This line wrapping behavior makes sense for [`Direction::Right`],
/// but not so much for [`Direction::Down`].. Maybe we should split [`CxTurtle`]
/// into different kinds of behavior?
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Right,
    Down,
}
impl Direction {
    /// TODO(JP): Replace these with Direction::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Direction = Direction::Right;
}
impl Default for Direction {
    fn default() -> Self {
        Direction::DEFAULT
    }
}

/// Different ways in which a [`Walk`] can get a width.
///
/// TODO(JP): Something like `FillUpTo(f32)` or `FillMax(f32)` might be useful here, to mimic
/// CSS'es `max-width`. For now you can manually use `Cx::get_width_left` with
/// `Width::Fix` as a workaround.
///
/// TODO(JP): See [`Height::DEFAULT`] for a related TODO.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Width {
    /// Fill up as much of the available space as possible.
    Fill,
    /// Use a fixed width.
    Fix(f32),
    /// Will defer computation of [`CxTurtle::width`] by setting it to [`f32::NAN`],
    /// and only properly computing it later on.
    ///
    /// TODO(JP): This can also be passed into [`Cx::walk_turtle`] but there it
    /// makes no sense!
    Compute,
    // Fill up as much of the available space as possible up to provided width
    FillUntil(f32),
}
impl Width {
    /// TODO(JP): Replace these with Width::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Width = Width::Fill;
}
impl Default for Width {
    fn default() -> Self {
        Width::Fill
    }
}

/// Different ways in which a [`Walk`] can get a height.
///
/// See [`Width`] for more documentation, since it's analogous.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Height {
    // See [`Width::Fill`].
    Fill,
    // See [`Width::Fix`].
    Fix(f32),
    // See [`Width::Compute`].
    Compute,
    // See [`Width::FillUntil`],
    FillUntil(f32),
}
impl Height {
    /// TODO(JP): [`Height::Fill`] might be a bad default, because if you use
    /// [`Direction::Down`] it will push out everything out it below.
    /// HTML/CSS uses something more like [`Height::Compute`] by default for height,
    /// and only [`Height::Fill`] for width (for block-layout elements).
    ///
    /// TODO(JP): Replace these with Height::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Height = Height::Fill;
}
impl Default for Height {
    fn default() -> Self {
        Height::Fill
    }
}

/// Defines how elements on [`Cx::turtle_align_list`] should be moved horizontally
pub(crate) struct AlignX(pub f32);

impl AlignX {
    // Note: LEFT is the default so not needed as explicit option
    pub(crate) const CENTER: AlignX = AlignX(0.5);
    #[allow(dead_code)]
    pub(crate) const RIGHT: AlignX = AlignX(1.0);
}

/// Defines how elements on [`Cx::turtle_align_list`] should be moved vertically
pub(crate) struct AlignY(pub f32);

impl AlignY {
    // Note: TOP is the default so not needed as explicit option
    pub(crate) const CENTER: AlignY = AlignY(0.5);
    #[allow(dead_code)]
    pub(crate) const BOTTOM: AlignY = AlignY(1.0);
}
