//! Layout system. 🐢

use crate::debug_log::DebugLog;
use crate::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CxTurtleType {
    Normal,
    RightBox,
    BottomBox,
    CenterXAlign,
    CenterYAlign,
    CenterXYAlign,
    PaddingBox,
    Row,
    Column,
    AbsoluteBox,
    WrappingBox,
}

impl Default for CxTurtleType {
    fn default() -> CxTurtleType {
        CxTurtleType::Normal
    }
}

/// A [`Turtle`] is a structure used for layouting: laying out different widgets on the screen.
///
/// This is really a pointer to a [`CxTurtle`] (indexed in [`Cx::turtles`] using [`Turtle::turtle_id`]),
/// so you can find more information there.
///
/// TODO(JP): Most methods for turtles are actually on [`Cx`]. This can make it confusing which
/// [`Turtle`] is actually being moved. It would be good to move more methods into [`Turtle`] itself.
#[derive(Clone)]
pub struct Turtle {
    /// The id referring to the corresponding [`CxTurtle`]. It's an index in [`Cx::turtles`].
    turtle_id: usize,
}

/// A [`CxTurtle`] is a structure used for layouting: laying out different widgets on the screen.
///
/// It is roughly modeled after the [Logo Turtle](https://en.wikipedia.org/wiki/Logo_(programming_language))
/// of old, but our version has a lot more state, and behaves differently in many ways.
///
/// At the core you can imagine the turtle as having a position ([`CxTurtle::pos`]), a "sandbox" that
/// it can move in (delineated by [`CxTurtle::origin`] and [`CxTurtle::width`] and [`CxTurtle::height`]).
///
/// Its movement is determined primarily by the [`Layout`] that you pass in, and you can modify it
/// ad-hoc by calling various functions.
///
/// Turtles can be nested, so we have a stack of turtles in [`Cx::turtles`]. The last [`CxTurtle`] on
/// the stack is the "current" or "active" turtle. When you call [`Cx::end_turtle`], the last turtle's
/// "sandbox" [`Rect`] will be used to walk its parent turtle. It truly is [turtles all the way
/// down](https://en.wikipedia.org/wiki/Turtles_all_the_way_down)!
///
/// A core philosophy of the turtle model is its simplicity and speed, by having only a single pass
/// to do layouting. Contrast this with systems like [CSS Flexbox](https://en.wikipedia.org/wiki/CSS_Flexible_Box_Layout),
/// which use a constraint satisfaction system to lay out your widgets. Instead, we make a single
/// pass, but do sometimes shift over individual elements after the fact, typically using
/// [`Cx::turtle_align_list`]. When doing this we can regard it as a "1.5-pass" rendering. Currently
/// we have to go through every individual element if we want to move it, but in the future we could
/// exploit groupings of elements in [`View`]s and [`DrawCall`]s, and set uniforms on them.
///
/// TODO(JP): The way the turtle moves around is quite confusing in a lot of cases! This model
/// probably requires a complete rework. We can take inspiration from other layouting systems (e.g.
/// the [CSS box model](https://developer.mozilla.org/en-US/docs/Learn/CSS/Building_blocks/The_box_model))
/// while retaining the philosophical core of the current turtle approach.
#[derive(Clone, Default, Debug)]
pub(crate) struct CxTurtle {
    /// The layout that is associated directly with this turtle, which determines a lot of its
    /// behavior.
    pub(crate) layout: Layout,

    /// The index within Cx::turtle_align_list, which contains all the things that we draw within this
    /// turtle, and which needs to get aligned at some point. We have a separate list for x/y
    /// because you can manually trigger an alignment (aside from it happening automatically at the
    /// end), which resets this list to no longer align those areas again.
    pub(crate) align_list_x_start_index: usize,

    /// Same as [`CxTurtle::align_list_x_start_index`] but for vertical alignment.
    pub(crate) align_list_y_start_index: usize,

    /// The current position of the turtle. This is the only field that seems to actually correspond
    /// to the "turtle graphics" metaphor! This is an absolute position, and starts out at [`CxTurtle::origin`]
    /// plus padding.
    pub(crate) pos: Vec2,

    /// The origin of the current turtle's walking area. Starts off at the parent's turtle [`CxTurtle::pos`]
    pub(crate) origin: Vec2,

    /// The inherent width of the current turtle's walking area. Is [`f32::NAN`] if the width is computed,
    /// and can get set explicitly later.
    pub(crate) width: f32,

    /// The inherent height of the current turtle's walking area. Is [`f32::NAN`] if the height is computed,
    /// and can get set explicitly later.
    pub(crate) height: f32,

    /// Seems to only be used to be passed down to child turtles, so if one of them gets an absolute
    /// origin passed in, we can just use the entire remaining absolute canvas as the width/height.
    ///
    /// TODO(JP): seems pretty unnecessary; why not just grab this field from the current [`Pass`
    /// directly if necessary? Or just always have the caller pass it in (they can take it from the
    /// current [`Pass`] if they want)?
    pub(crate) abs_size: Vec2,

    /// Keeps track of the bottom right corner where we have walked so far, including the width/height
    /// of the walk, whereas [`CxTurtle::pos`] stays in the top left position of what we have last drawn.
    ///
    /// TODO(JP): [`CxTurtle::pos`] and [`CxTurtle::bound_right_bottom`] can (and seem to regularly and intentionally do)
    /// get out of sync, which makes things more confusing.
    pub(crate) bound_right_bottom: Vec2,

    /// We keep track of the [`Walk`] with the greatest height (or width, when walking down), so that
    /// we know how much to move the turtle's y-position when wrapping to the next line. When
    /// wrapping to the next line, this value is reset back to 0.
    ///
    /// See also [`Padding`].
    pub(crate) biggest: f32,

    /// Used for additional checks that enclosing turtles match opening ones
    pub(crate) turtle_type: CxTurtleType,

    /// Available width for the content of the turtle, starting from turtle origin, minus right padding.
    /// This is different from [`CxTurtle::width`] which is turtle outer width.
    /// For example, for Width::Compute turtles width would be [`f32::NAN`] as this needs to be computed,
    /// but available_width is defined until the bounds of parent
    pub(crate) available_width: f32,

    /// Available height for the content of the turtle, starting from turtle origin, minus bottom padding.
    /// This is different from [`CxTurtle::height`] which is turtle outer height.
    /// For example, for height::Compute turtles height would be [`f32::NAN`] as this needs to be computed,
    /// but available_height is defined until the bounds of parent
    pub(crate) available_height: f32,
}

impl CxTurtle {
    /// Returns how much available_width is "left" for current turtle,
    /// i.e. distance from current turtle x position until the right bound
    pub(crate) fn get_available_width_left(&self) -> f32 {
        (self.origin.x + self.available_width - self.pos.x).max(0.)
    }

    /// Returns how much available_height is "left" for current turtle
    /// i.e. distance from current turtle y position until the bottom bound
    pub(crate) fn get_available_height_left(&self) -> f32 {
        (self.origin.y + self.available_height - self.pos.y).max(0.)
    }
}

impl Cx {
    /// Begin a new [`CxTurtle`] with a given [`Layout`]. This new [`CxTurtle`] will be added to the
    /// [`Cx::turtles`] stack.
    pub(crate) fn begin_turtle(&mut self, layout: Layout) -> Turtle {
        self.begin_typed_turtle(CxTurtleType::Normal, layout)
    }

    pub(crate) fn begin_typed_turtle(&mut self, turtle_type: CxTurtleType, layout: Layout) -> Turtle {
        if !self.in_redraw_cycle {
            panic!("calling begin_turtle outside of redraw cycle is not possible!");
        }
        if layout.direction == Direction::Down && layout.line_wrap != LineWrap::None {
            panic!("Direction down with line wrapping is not supported");
        }

        // fetch origin and size from parent
        let (mut origin, mut abs_size) = if let Some(parent) = self.turtles.last() {
            (Vec2 { x: parent.pos.x, y: parent.pos.y }, parent.abs_size)
        } else {
            assert!(layout.absolute);
            assert!(layout.abs_size.is_some());
            (Vec2 { x: 0., y: 0. }, Vec2::default())
        };

        // see if layout overrode size
        if let Some(layout_abs_size) = layout.abs_size {
            abs_size = layout_abs_size;
        }

        let width;
        let height;
        if layout.absolute {
            // absolute overrides origin to start from (0, 0)
            origin = vec2(0.0, 0.0);
            // absolute overrides the computation of width/height to use the parent absolute
            width = self.eval_absolute_width(&layout.walk.width, abs_size.x);
            height = self.eval_absolute_height(&layout.walk.height, abs_size.y);
        } else {
            width = self.eval_width(&layout.walk.width);
            height = self.eval_height(&layout.walk.height);
        }

        let pos = Vec2 { x: origin.x + layout.padding.l, y: origin.y + layout.padding.t };

        let available_width = (self.eval_available_width_left(&layout.walk.width, abs_size) - layout.padding.r).max(0.);
        let available_height = (self.eval_available_height_left(&layout.walk.height, abs_size) - layout.padding.b).max(0.);

        // By induction property this values should never be NaN
        assert!(!available_width.is_nan());
        assert!(!available_height.is_nan());

        let turtle = CxTurtle {
            align_list_x_start_index: self.turtle_align_list.len(),
            align_list_y_start_index: self.turtle_align_list.len(),
            origin,
            pos,
            layout,
            biggest: 0.0,
            bound_right_bottom: Vec2 { x: std::f32::NEG_INFINITY, y: std::f32::NEG_INFINITY },
            width,
            height,
            abs_size,
            turtle_type,
            available_height,
            available_width,
        };

        self.turtles.push(turtle);
        Turtle { turtle_id: self.turtles.len() - 1 }
    }

    fn assert_last_turtle_matches(&self, turtle: &Turtle) {
        let expected_turtle_id = self.turtles.len() - 1;
        if turtle.turtle_id != expected_turtle_id {
            panic!(
                "End turtle turtle_id incorrect! Was called with {} but should have been {}",
                turtle.turtle_id, expected_turtle_id
            );
        }
    }

    pub(crate) fn end_typed_turtle(&mut self, turtle_type: CxTurtleType) -> Rect {
        self.assert_last_turtle_type_matches(turtle_type);
        self.end_last_turtle_unchecked()
    }

    pub(crate) fn assert_last_turtle_type_matches(&self, turtle_type: CxTurtleType) {
        let cx_turtle = self.turtles.last().unwrap();
        if cx_turtle.turtle_type != turtle_type {
            panic!("Closing turtle type doesn't match! Expected: {:?}, found: {:?}", turtle_type, cx_turtle.turtle_type);
        }
    }

    /// Similar to [`Cx::end_turtle`], but doesn't do any matching checks on the turtle. Use at your own risk!
    pub fn end_last_turtle_unchecked(&mut self) -> Rect {
        let old = self.turtles.pop().unwrap();
        let w = if old.width.is_nan() {
            // when nesting Fill turtle inside Compute the former would have nan width
            if old.layout.walk.width == Width::Fill {
                // use all available width + padding
                Width::Fix(old.available_width + old.layout.padding.r)
            } else if old.bound_right_bottom.x == std::f32::NEG_INFINITY {
                // nothing happened, use padding
                Width::Fix(old.layout.padding.l + old.layout.padding.r)
            } else {
                // use the bounding box
                Width::Fix(max_zero_keep_nan(old.bound_right_bottom.x - old.origin.x + old.layout.padding.r))
            }
        } else {
            Width::Fix(old.width)
        };

        let h = if old.height.is_nan() {
            // when nesting Fill turtle inside Compute the former would have nan height
            if old.layout.walk.height == Height::Fill {
                // use all available height + padding
                Height::Fix(old.available_height + old.layout.padding.b)
            } else if old.bound_right_bottom.y == std::f32::NEG_INFINITY {
                // nothing happened use the padding
                Height::Fix(old.layout.padding.t + old.layout.padding.b)
            } else {
                // use the bounding box
                Height::Fix(max_zero_keep_nan(old.bound_right_bottom.y - old.origin.y + old.layout.padding.b))
            }
        } else {
            Height::Fix(old.height)
        };

        let rect = {
            // when a turtle is absolutely positioned don't walk the parent
            if old.layout.absolute {
                let w = if let Width::Fix(vw) = w { vw } else { 0. };
                let h = if let Height::Fix(vh) = h { vh } else { 0. };
                Rect { pos: vec2(0., 0.), size: vec2(w, h) }
            } else {
                self.walk_turtle_with_old(Walk { width: w, height: h }, Some(&old))
            }
        };
        self.debug_logs.push(DebugLog::EndTurtle { rect });
        rect
    }

    /// Pop the current [`CxTurtle`] from the [`Cx::turtles`] stack, returning a [`Rect`] that the turtle walked
    /// during its lifetime. The parent [`CxTurtle`] will be made to walk this [`Rect`].
    ///
    /// Note that this is a method on [`Cx`] instead of on [`Turtle`], since this way we can take ownership
    /// of the [`Turtle`], making it less likely that you accidentally reuse the [`Turtle`] after ending it.
    pub(crate) fn end_turtle(&mut self, turtle: Turtle) -> Rect {
        self.assert_last_turtle_matches(&turtle);
        self.end_last_turtle_unchecked()
    }

    /// Walk the current [`CxTurtle`], returning a [`Rect`] that it ended up walking.
    pub fn walk_turtle(&mut self, walk: Walk) -> Rect {
        self.walk_turtle_with_old(walk, None)
    }

    /// Walk the turtle with a 'w/h'
    ///
    /// Returns a [`Rect`] containing the area that the turtle walked
    ///
    /// TODO(JP): This `old_turtle` stuff is a bit awkward (awkward turtle..) and only used for the
    /// alignment stuff at the end. We can probably structure this in a nicer way.
    fn walk_turtle_with_old(&mut self, walk: Walk, old_turtle: Option<&CxTurtle>) -> Rect {
        let mut align_dx = 0.0;
        let mut align_dy = 0.0;

        // TODO(JP): This seems a bit weird: you can technically pass in Width::Compute, which would
        // return a NaN for `w`, but that doesn't make much sense when you explicitly do a walk.
        // It looks like it's assumed that that never gets passed in here, but it would be better to
        // verify that.
        // NOTE(Dmitry): now this methods will panic when receiving Compute walks.
        // We can probably express this better in type system, but this is good enough for now.
        let w = self.eval_walking_width(&walk.width);
        let h = self.eval_walking_height(&walk.height);

        let ret = if let Some(turtle) = self.turtles.last_mut() {
            let old_pos = match turtle.layout.direction {
                Direction::Right => {
                    match turtle.layout.line_wrap {
                        LineWrap::Overflow => {
                            if (turtle.pos.x + w) > (turtle.origin.x + turtle.available_width) + 0.01 {
                                // what is the move delta.
                                let old_x = turtle.pos.x;
                                let old_y = turtle.pos.y;
                                turtle.pos.x = turtle.origin.x + turtle.layout.padding.l;
                                turtle.pos.y += turtle.biggest;
                                turtle.biggest = 0.0;
                                align_dx = turtle.pos.x - old_x;
                                align_dy = turtle.pos.y - old_y;
                            }
                        }
                        LineWrap::None => {}
                    }

                    let old_pos = turtle.pos;
                    // walk it normally
                    turtle.pos.x += w;

                    // keep track of biggest item in the line
                    turtle.biggest = turtle.biggest.max(h);
                    old_pos
                }
                Direction::Down => {
                    let old_pos = turtle.pos;
                    // walk it normally
                    turtle.pos.y += h;

                    // keep track of biggest item in the line
                    turtle.biggest = turtle.biggest.max(w);
                    old_pos
                }
            };

            // update bounds
            let new_bound = old_pos + vec2(w, h);
            turtle.bound_right_bottom = turtle.bound_right_bottom.max(&new_bound);

            Rect { pos: old_pos, size: vec2(w, h) }
        } else {
            Rect { pos: vec2(0.0, 0.0), size: vec2(w, h) }
        };

        if align_dx != 0.0 {
            if let Some(old_turtle) = old_turtle {
                self.move_by_x(align_dx, old_turtle.align_list_x_start_index);
            }
        };
        if align_dy != 0.0 {
            if let Some(old_turtle) = old_turtle {
                self.move_by_y(align_dy, old_turtle.align_list_y_start_index);
            }
        };

        ret
    }

    /// High performance turtle walk with no indirections and compute visibility.
    pub fn walk_turtle_right_no_wrap(&mut self, w: f32, h: f32, scroll: Vec2) -> Option<Rect> {
        if let Some(turtle) = self.turtles.last_mut() {
            let x = turtle.pos.x;
            let y = turtle.pos.y;
            // walk it normally
            turtle.pos.x += w;

            // keep track of biggest item in the line
            turtle.biggest = turtle.biggest.max(h);

            // update bounds
            turtle.bound_right_bottom.x = turtle.bound_right_bottom.x.max(x + w);
            turtle.bound_right_bottom.y = turtle.bound_right_bottom.y.max(y + h);

            let vx = turtle.origin.x + scroll.x;
            let vy = turtle.origin.y + scroll.y;
            let vw = turtle.width;
            let vh = turtle.height;

            if x > vx + vw || x + w < vx || y > vy + vh || y + h < vy {
                None
            } else {
                Some(Rect { pos: vec2(x, y), size: vec2(w, h) })
            }
        } else {
            None
        }
    }

    /// Explicitly move the current [`CxTurtle`] to a new line.
    ///
    /// TODO(JP): Mostly relevant for [`Direction::Right`], should we just disable
    /// this for [`Direction::Down`] to avoid confusion?
    pub fn turtle_new_line(&mut self) {
        if let Some(turtle) = self.turtles.last_mut() {
            assert_eq!(turtle.layout.direction, Direction::Right, "turtle_new_line with Direction::Down is not supported");
            turtle.pos.x = turtle.origin.x + turtle.layout.padding.l;
            turtle.pos.y += turtle.biggest;
            turtle.biggest = 0.0;
        }
    }

    /// [`Cx::turtle_new_line`] but allows setting a minimum height for the line.
    ///
    /// TODO(JP): Should we instead include `min_height` in [`Layout`]?
    pub fn turtle_new_line_min_height(&mut self, min_height: f32) {
        if let Some(turtle) = self.turtles.last_mut() {
            assert_eq!(
                turtle.layout.direction,
                Direction::Right,
                "turtle_new_line_min_height with Direction::Down is not supported"
            );
            turtle.pos.x = turtle.origin.x + turtle.layout.padding.l;
            turtle.pos.y += turtle.biggest.max(min_height);
            turtle.biggest = 0.0;
        }
    }

    /// Check if a particular line is visible.
    ///
    /// TODO(JP): Only used in one place currently; should we instead expose
    /// more low-level primitives so the user can compute this themselves?
    pub fn turtle_line_is_visible(&mut self, min_height: f32, scroll: Vec2) -> bool {
        if let Some(turtle) = self.turtles.last_mut() {
            assert_eq!(turtle.layout.direction, Direction::Right);
            let y = turtle.pos.y;
            let h = turtle.biggest.max(min_height);
            let vy = turtle.origin.y + scroll.y;
            let vh = turtle.height;

            return !(y > vy + vh || y + h < vy);
        }
        false
    }

    /// Actually perform a horizontal movement of items in [`Cx::turtle_align_list`], but only for positive dx
    pub(crate) fn do_align_x(&mut self, dx: f32, align_start: usize) {
        if dx < 0. {
            // do only forward moving alignment
            // backwards alignment could happen if the size of content became larger than the container
            // in which case the alignment is not well defined
            return;
        }
        self.move_by_x(dx, align_start)
    }

    /// Actually perform a horizontal movement of items in [`Cx::turtle_align_list`].
    /// Unlike "do_align_x" negative moves can happen here because of wrapping behavior.
    ///
    /// TODO(JP): Should we move some of this stuff to [`Area`], where we already seem to do a bunch
    /// of rectangle and position calculations?
    fn move_by_x(&mut self, dx: f32, align_start: usize) {
        let dx = (dx * self.current_dpi_factor).floor() / self.current_dpi_factor;
        for i in align_start..self.turtle_align_list.len() {
            let align_item = &self.turtle_align_list[i];
            match align_item {
                Area::InstanceRange(inst) => {
                    let cxview = &mut self.views[inst.view_id];
                    let draw_call = &mut cxview.draw_calls[inst.draw_call_id];
                    let sh = &self.shaders[draw_call.shader_id];
                    for i in 0..inst.instance_count {
                        if let Some(rect_pos) = sh.mapping.rect_instance_props.rect_pos {
                            draw_call.instances[inst.instance_offset + rect_pos + i * sh.mapping.instance_props.total_slots] +=
                                dx;
                        }
                    }
                }
                Area::View(view_area) => {
                    let cxview = &mut self.views[view_area.view_id];
                    cxview.rect.pos.x += dx;
                }
                // TODO(JP): Would be nice to implement this for [`Align::View`], which would
                // probably require some offset field on [`CxView`] that gets used during rendering.
                _ => unreachable!(),
            }
        }
    }

    /// Actually perform a vertical movement of items in [`Cx::turtle_align_list`], but only for positive dy
    pub(crate) fn do_align_y(&mut self, dy: f32, align_start: usize) {
        if dy < 0. {
            // do only forward moving alignment
            // backwards alignment could happen if the size of content became larger than the container
            // in which case the alignment is not well defined
            return;
        }
        self.move_by_y(dy, align_start);
    }

    /// Actually perform a vertical movement of items in [`Cx::turtle_align_list`].
    /// Unlike "do_align_y" negative moves can happen here because of wrapping behavior.
    ///
    /// TODO(JP): Should we move some of this stuff to [`Area`], where we already seem to do a bunch
    /// of rectangle and position calculations?
    fn move_by_y(&mut self, dy: f32, align_start: usize) {
        let dy = (dy * self.current_dpi_factor).floor() / self.current_dpi_factor;
        for i in align_start..self.turtle_align_list.len() {
            let align_item = &self.turtle_align_list[i];
            match align_item {
                Area::InstanceRange(inst) => {
                    let cxview = &mut self.views[inst.view_id];
                    let draw_call = &mut cxview.draw_calls[inst.draw_call_id];
                    let sh = &self.shaders[draw_call.shader_id];
                    for i in 0..inst.instance_count {
                        if let Some(rect_pos) = sh.mapping.rect_instance_props.rect_pos {
                            draw_call.instances
                                [inst.instance_offset + rect_pos + 1 + i * sh.mapping.instance_props.total_slots] += dy;
                        }
                    }
                }
                Area::View(view_area) => {
                    let cxview = &mut self.views[view_area.view_id];
                    cxview.rect.pos.y += dy;
                }
                // TODO(JP): Would be nice to implement this for `Align::View`, which would
                // probably require some offset field on `CxView` that gets used during rendering.
                _ => unreachable!(),
            }
        }
    }

    /// Get the [`Rect`] that contains the current [`CxTurtle::origin`], [`CxTurtle::width], and
    /// [`CxTurtle::height`]. Note that these are the inherent dimensions of the [`CxTurtle`], not
    /// what the [`CxTurtle`] has walked so far. See [`Cx::get_turtle_bounds`] for that.
    ///
    /// TODO(JP): When using [`Width::Compute`] or [`Height::Compute`], this [`Rect`] may include
    /// [`f32::NAN`]s, which is unexpected.
    pub fn get_turtle_rect(&self) -> Rect {
        if let Some(turtle) = self.turtles.last() {
            return Rect { pos: turtle.origin, size: vec2(turtle.width, turtle.height) };
        };
        Rect::default()
    }

    /// Get the bounds of what the turtle has *actually* walked (not just its
    /// inherent width/height as given by [`Cx::get_turtle_rect`]), including any padding that the
    /// layout of the current turtle specifies.
    pub fn get_turtle_bounds(&self) -> Vec2 {
        if let Some(turtle) = self.turtles.last() {
            return Vec2 {
                x: if turtle.bound_right_bottom.x < 0. { 0. } else { turtle.bound_right_bottom.x } + turtle.layout.padding.r
                    - turtle.origin.x,
                y: if turtle.bound_right_bottom.y < 0. { 0. } else { turtle.bound_right_bottom.y } + turtle.layout.padding.b
                    - turtle.origin.y,
            };
        }
        Vec2::default()
    }

    /// Overwrite the turtle bounds to pretend that it has actually walked
    /// a bunch.
    ///
    /// TODO(JP): this seems.. bad? Can we restructure all this?
    pub fn set_turtle_bounds(&mut self, bound: Vec2) {
        if let Some(turtle) = self.turtles.last_mut() {
            turtle.bound_right_bottom = Vec2 {
                x: bound.x - turtle.layout.padding.r + turtle.origin.x,
                y: bound.y - turtle.layout.padding.b + turtle.origin.y,
            }
        }
    }

    /// Same as [`Cx::get_turtle_rect().pos`].
    ///
    /// TODO(JP): Do we really need two different methods to get to the same data?
    pub fn get_turtle_origin(&self) -> Vec2 {
        if let Some(turtle) = self.turtles.last() {
            return turtle.origin;
        }
        Vec2::default()
    }

    /// Get the current [`CxTurtle::pos`] in absolute coordinates.
    ///
    /// See also [`Cx::get_rel_turtle_pos`].
    ///
    /// TODO(JP): Only used in two places currently; do we really need this?
    pub fn get_turtle_pos(&self) -> Vec2 {
        if let Some(turtle) = self.turtles.last() {
            turtle.pos
        } else {
            Vec2::default()
        }
    }

    /// Get the current [`CxTurtle::pos`] in coordinates relative to [`CxTurtle::origin`].
    ///
    /// See also [`Cx::get_turtle_pos`].
    ///
    /// TODO(JP): Only used in one place currently; do we really need this?
    pub fn get_rel_turtle_pos(&self) -> Vec2 {
        if let Some(turtle) = self.turtles.last() {
            Vec2 { x: turtle.pos.x - turtle.origin.x, y: turtle.pos.y - turtle.origin.y }
        } else {
            Vec2::default()
        }
    }

    /// Manually change [`CxTurtle::pos`]. Warning! Does not update [`CxTurtle::bound_right_bottom`],
    /// like [`Cx::walk_turtle`] does; might result in unexpected behavior.
    ///
    /// TODO(JP): Should we delete this and just always use [`Cx::walk_turtle`] instead?
    pub fn move_turtle(&mut self, dx: f32, dy: f32) {
        if let Some(turtle) = self.turtles.last_mut() {
            turtle.pos.x += dx;
            turtle.pos.y += dy;
        }
    }

    /// Manually change [`CxTurtle::pos`]. Warning! Does not update [`CxTurtle::bound_right_bottom`],
    /// like [`Cx::walk_turtle`] does; might result in unexpected behavior.
    ///
    /// TODO(JP): Should we delete this and just always use [`Cx::walk_turtle`] instead?
    pub fn set_turtle_pos(&mut self, pos: Vec2) {
        if let Some(turtle) = self.turtles.last_mut() {
            turtle.pos = pos
        }
    }

    /// Returns how many pixels we should move over based on the [`AlignX`] ratio
    /// (which is between 0 and 1). We do this by looking at the bound
    /// ([`CxTurtle::bound_right_bottom`]) to see how much we have actually drawn, and how
    /// subtract that from the width of this turtle. That "remaining width" is
    /// then multiplied with the ratio. If there is no inherent width then this
    /// will return 0.
    pub(crate) fn compute_align_turtle_x(turtle: &CxTurtle, align: AlignX) -> f32 {
        let AlignX(fx) = align;
        if fx > 0.0 {
            // TODO(Dmitry): check if we need use padding here
            let dx = fx
                * ((turtle.available_width - (turtle.layout.padding.l + turtle.layout.padding.r))
                    - (turtle.bound_right_bottom.x - (turtle.origin.x + turtle.layout.padding.l)));
            if dx.is_nan() {
                return 0.0;
            }
            dx
        } else {
            0.
        }
    }

    /// Returns how many pixels we should move over based on the [`AlignY`] ratio
    /// (which is between 0 and 1). We do this by looking at the bound
    /// ([`CxTurtle::bound_right_bottom`]) to see how much we have actually drawn, and how
    /// subtract that from the height of this turtle. That "remaining height" is
    /// then multiplied with the ratio. If there is no inherent height then this
    /// will return 0.
    pub(crate) fn compute_align_turtle_y(turtle: &CxTurtle, align: AlignY) -> f32 {
        let AlignY(fy) = align;
        if fy > 0.0 {
            // TODO(Dmitry): check if we need use padding here
            let dy = fy
                * ((turtle.available_height - (turtle.layout.padding.t + turtle.layout.padding.b))
                    - (turtle.bound_right_bottom.y - (turtle.origin.y + turtle.layout.padding.t)));
            if dy.is_nan() {
                return 0.0;
            }
            dy
        } else {
            0.
        }
    }

    /// If the height is computed, then this will set the height to however much is
    /// drawn so far, using [`CxTurtle::bound_right_bottom`].
    ///
    /// TODO(JP): This also resets the [`CxTurtle::bound_right_bottom.y`] back to 0 as if
    /// nothing has been drawn. How does that make sense exactly? Is that wrong?
    ///
    /// TODO(JP): This function is currently only used once..
    pub fn compute_turtle_height(&mut self) {
        if let Some(turtle) = self.turtles.last_mut() {
            if turtle.height.is_nan() && turtle.bound_right_bottom.y != std::f32::NEG_INFINITY {
                // nothing happened use the padding
                turtle.height = max_zero_keep_nan(turtle.bound_right_bottom.y - turtle.origin.y + turtle.layout.padding.b);
                turtle.bound_right_bottom.y = 0.;
            }
        }
    }

    /// Reset the current position of the current [`CxTurtle`] to the starting position
    /// ([`CxTurtle::origin`] + [`Layout::padding`]).
    ///
    /// TODO(JP): Note that this does *not* reset [`CxTurtle::bound_right_bottom`] or
    /// [`CxTurtle::biggest`] or stuff like that, so there is still some leftover state which
    /// might be confusing.
    pub fn reset_turtle_pos(&mut self) {
        if let Some(turtle) = self.turtles.last_mut() {
            // subtract used size so 'fill' works
            turtle.pos = Vec2 { x: turtle.origin.x + turtle.layout.padding.l, y: turtle.origin.y + turtle.layout.padding.t };
        }
    }

    /// Get some notion of the width that is "left" for the current [`CxTurtle`].
    ///
    /// See also [`Cx::get_width_total`].
    pub fn get_width_left(&self) -> f32 {
        if let Some(turtle) = self.turtles.last() {
            return max_zero_keep_nan(turtle.width - (turtle.pos.x - turtle.origin.x));
        }
        0.
    }

    fn get_available_width_left(&self) -> f32 {
        if let Some(turtle) = self.turtles.last() {
            turtle.get_available_width_left()
        } else {
            0.
        }
    }

    /// Get some notion of the total width of the current turtle. If the width
    /// is well defined, then we return it. If it's computed, then we return the
    /// bound (including padding) of how much we've drawn so far. And if we haven't
    /// drawn anything, we return NaN.
    pub fn get_width_total(&self) -> f32 {
        if let Some(turtle) = self.turtles.last() {
            let nan_val = max_zero_keep_nan(turtle.width /* - (turtle.layout.padding.l + turtle.layout.padding.r)*/);
            if nan_val.is_nan() {
                // if we are a computed width, if some value is known, use that
                if turtle.bound_right_bottom.x != std::f32::NEG_INFINITY {
                    return turtle.bound_right_bottom.x - turtle.origin.x + turtle.layout.padding.r;
                }
            }
            return nan_val;
        }
        0.
    }

    /// Get some notion of the height that is "left" for the current [`CxTurtle`].
    ///
    /// See also [`Cx::get_height_total`].
    pub fn get_height_left(&self) -> f32 {
        if let Some(turtle) = self.turtles.last() {
            return max_zero_keep_nan(turtle.height - (turtle.pos.y - turtle.origin.y));
        }
        0.
    }

    pub fn get_available_height_left(&self) -> f32 {
        if let Some(turtle) = self.turtles.last() {
            turtle.get_available_height_left()
        } else {
            0.
        }
    }

    /// Get some notion of the total height of the current turtle. If the height
    /// is well defined, then we return it. If it's computed, then we return the
    /// bound (including padding) of how much we've drawn so far. And if we haven't
    /// drawn anything, we return NaN.
    pub fn get_height_total(&self) -> f32 {
        if let Some(turtle) = self.turtles.last() {
            let nan_val = max_zero_keep_nan(turtle.height /*- (turtle.layout.padding.t + turtle.layout.padding.b)*/);
            if nan_val.is_nan() {
                // if we are a computed height, if some value is known, use that
                if turtle.bound_right_bottom.y != std::f32::NEG_INFINITY {
                    return turtle.bound_right_bottom.y - turtle.origin.y + turtle.layout.padding.b;
                }
            }
            return nan_val;
        }
        0.
    }

    /// Whether the current [`CxTurtle::layout`] uses [`Height::Compute`].
    ///
    /// TODO(JP): We only use this in one place; is this really necessary?
    pub fn is_height_computed(&self) -> bool {
        if let Some(turtle) = self.turtles.last() {
            if let Height::Compute = turtle.layout.walk.height {
                return true;
            }
        }
        false
    }

    // TODO(Dmitry): simplify all the following eval functions
    fn eval_width(&self, width: &Width) -> f32 {
        match width {
            Width::Compute => std::f32::NAN,
            Width::Fix(v) => max_zero_keep_nan(*v),
            Width::Fill => max_zero_keep_nan(self.get_width_left()),
            Width::FillUntil(v) => min_keep_nan(*v, self.get_width_left()),
        }
    }

    fn eval_absolute_width(&self, width: &Width, abs_size: f32) -> f32 {
        match width {
            Width::Compute => std::f32::NAN,
            Width::Fix(v) => max_zero_keep_nan(*v),
            Width::Fill => max_zero_keep_nan(abs_size),
            Width::FillUntil(v) => min_keep_nan(*v, abs_size),
        }
    }

    fn eval_walking_width(&self, width: &Width) -> f32 {
        match width {
            Width::Compute => panic!("Walking with Width:Compute is not supported"),
            Width::Fix(v) => v.max(0.),
            Width::Fill => self.get_available_width_left(),
            Width::FillUntil(v) => self.get_available_width_left().min(*v),
        }
    }

    fn eval_available_width_left(&self, width: &Width, abs_size: Vec2) -> f32 {
        if let Some(parent) = self.turtles.last() {
            match width {
                Width::Fix(v) => *v,
                Width::FillUntil(v) => parent.get_available_width_left().min(*v),
                Width::Compute | Width::Fill => parent.get_available_width_left(),
            }
        } else {
            abs_size.x
        }
    }

    fn eval_height(&self, height: &Height) -> f32 {
        match height {
            Height::Compute => std::f32::NAN,
            Height::Fix(v) => max_zero_keep_nan(*v),
            Height::Fill => max_zero_keep_nan(self.get_height_left()),
            Height::FillUntil(v) => min_keep_nan(*v, self.get_height_left()),
        }
    }

    fn eval_absolute_height(&self, height: &Height, abs_size: f32) -> f32 {
        match height {
            Height::Compute => std::f32::NAN,
            Height::Fix(v) => max_zero_keep_nan(*v),
            Height::Fill => max_zero_keep_nan(abs_size),
            Height::FillUntil(v) => min_keep_nan(*v, abs_size),
        }
    }

    fn eval_walking_height(&self, height: &Height) -> f32 {
        match height {
            Height::Compute => panic!("Walking with Height:Compute is not supported"),
            Height::Fix(v) => v.max(0.),
            Height::Fill => self.get_available_height_left(),
            Height::FillUntil(v) => self.get_available_height_left().min(*v),
        }
    }

    fn eval_available_height_left(&self, height: &Height, abs_size: Vec2) -> f32 {
        if let Some(parent) = self.turtles.last() {
            match height {
                Height::Fix(v) => *v,
                Height::FillUntil(v) => parent.get_available_height_left().min(*v),
                Height::Compute | Height::Fill => parent.get_available_height_left(),
            }
        } else {
            abs_size.y
        }
    }

    /// Add an `Area::InstanceRange` to the [`Cx::turtle_align_list`], so that it will get aligned,
    /// e.g. when you call [`Cx::end_turtle`].
    pub(crate) fn add_to_turtle_align_list(&mut self, area: Area) {
        match area {
            Area::InstanceRange(_) => self.turtle_align_list.push(area),
            _ => panic!("Only Area::InstanceRange can be aligned currently"),
        }
    }
}

fn max_zero_keep_nan(v: f32) -> f32 {
    if v.is_nan() {
        v
    } else {
        f32::max(v, 0.0)
    }
}

fn min_keep_nan(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        f32::NAN
    } else {
        f32::min(a, b)
    }
}
