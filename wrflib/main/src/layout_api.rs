use crate::*;

impl Cx {
    /// Starts alignment element that fills all remaining space by y axis and centers content by it
    pub fn begin_center_y_align(&mut self) {
        let parent = self.turtles.last().unwrap();
        let turtle = CxTurtle {
            align_list_x_start_index: self.turtle_align_list.len(),
            align_list_y_start_index: self.turtle_align_list.len(),
            origin: parent.pos,
            pos: parent.pos,
            // fills out all remaining space by y axis
            layout: Layout { walk: Walk { height: Height::Fill, ..parent.layout.walk }, ..parent.layout },
            biggest: 0.0,
            bound_right_bottom: Vec2 { x: std::f32::NEG_INFINITY, y: std::f32::NEG_INFINITY },
            width: self.get_width_left(),
            height: self.get_height_left(),
            abs_size: parent.abs_size,
            turtle_type: CxTurtleType::CenterYAlign,
            available_width: parent.get_available_width_left(),
            available_height: parent.get_available_height_left(),
        };
        self.turtles.push(turtle);
    }

    pub fn end_center_y_align(&mut self) {
        self.assert_last_turtle_type_matches(CxTurtleType::CenterYAlign);

        let turtle = self.turtles.pop().unwrap();
        let dy = Cx::compute_align_turtle_y(&turtle, AlignY::CENTER);
        let align_start = turtle.align_list_y_start_index;
        self.do_align_y(dy, align_start);

        let parent = self.turtles.last_mut().unwrap();
        // TODO(Dmitry): communicating only few updates to parent for now. It's possible we need more.
        parent.bound_right_bottom.x = parent.bound_right_bottom.x.max(turtle.bound_right_bottom.x);
        parent.pos = turtle.pos;
    }

    /// Starts alignment element that fills all remaining space in turtle and centers content by x and y
    pub fn begin_center_x_and_y_align(&mut self) {
        let parent = self.turtles.last().unwrap();
        let turtle = CxTurtle {
            align_list_x_start_index: self.turtle_align_list.len(),
            align_list_y_start_index: self.turtle_align_list.len(),
            origin: parent.pos,
            pos: parent.pos,
            // fills out all remaining space by both axis
            layout: Layout { walk: Walk { width: Width::Fill, height: Height::Fill }, ..parent.layout },
            biggest: 0.0,
            bound_right_bottom: Vec2 { x: std::f32::NEG_INFINITY, y: std::f32::NEG_INFINITY },
            width: self.get_width_left(),
            height: self.get_height_left(),
            abs_size: parent.abs_size,
            turtle_type: CxTurtleType::CenterXYAlign,
            available_width: parent.get_available_width_left(),
            available_height: parent.get_available_height_left(),
        };
        self.turtles.push(turtle);
    }

    pub fn end_center_x_and_y_align(&mut self) {
        self.assert_last_turtle_type_matches(CxTurtleType::CenterXYAlign);
        let turtle = self.turtles.pop().unwrap();

        let dx = Cx::compute_align_turtle_x(&turtle, AlignX::CENTER);
        self.do_align_x(dx, turtle.align_list_x_start_index);

        let dy = Cx::compute_align_turtle_y(&turtle, AlignY::CENTER);
        self.do_align_y(dy, turtle.align_list_y_start_index);

        // TODO(Dmitry): we are not communicating any changes back to parent since we are filling all remaining place
        // it's possible this breaks in some cases
    }

    pub fn begin_row(&mut self, width: Width, height: Height) {
        self.begin_typed_turtle(
            CxTurtleType::Row,
            Layout { direction: Direction::Right, walk: Walk { width, height }, ..Layout::default() },
        );
    }

    /// Ends the current block that was opened by [`Cx::begin_row`].
    /// Returns a [`Rect`] representing the overall area of that row
    pub fn end_row(&mut self) -> Rect {
        self.end_typed_turtle(CxTurtleType::Row)
    }

    pub fn begin_column(&mut self, width: Width, height: Height) {
        self.begin_typed_turtle(
            CxTurtleType::Column,
            Layout { direction: Direction::Down, walk: Walk { width, height }, ..Layout::default() },
        );
    }

    /// Ends the current block that was opened by [`Cx::begin_column`].
    /// Returns a [`Rect`] representing the overall area of that column
    pub fn end_column(&mut self) -> Rect {
        self.end_typed_turtle(CxTurtleType::Column)
    }

    // Start new box that will be on the bottom by y axis
    pub fn begin_bottom_box(&mut self) {
        let parent = self.turtles.last().unwrap();
        let turtle = CxTurtle {
            align_list_x_start_index: self.turtle_align_list.len(),
            align_list_y_start_index: self.turtle_align_list.len(),
            origin: parent.pos,
            pos: parent.pos,
            layout: parent.layout,
            biggest: 0.0,
            bound_right_bottom: Vec2 { x: std::f32::NEG_INFINITY, y: std::f32::NEG_INFINITY },
            width: parent.width,
            height: parent.height,
            abs_size: parent.abs_size,
            turtle_type: CxTurtleType::BottomBox,
            available_width: parent.get_available_width_left(),
            available_height: parent.get_available_height_left(),
        };
        self.turtles.push(turtle);
    }

    pub fn end_bottom_box(&mut self) {
        self.assert_last_turtle_type_matches(CxTurtleType::BottomBox);

        let turtle = self.turtles.pop().unwrap();
        let parent = self.turtles.last_mut().unwrap();

        let drawn_height = turtle.bound_right_bottom.y - turtle.origin.y;
        let last_y = parent.origin.y + parent.available_height;
        let dy = last_y - turtle.bound_right_bottom.y;
        // update parent
        parent.available_height -= drawn_height;
        parent.pos = turtle.origin;
        parent.bound_right_bottom.x = parent.bound_right_bottom.x.max(turtle.bound_right_bottom.x);
        parent.bound_right_bottom.y = last_y;

        let align_start = turtle.align_list_y_start_index;
        self.do_align_y(dy, align_start);
    }

    /// Starts alignment element that fills all remaining space by x axis and centers content by it
    pub fn begin_center_x_align(&mut self) {
        let parent = self.turtles.last().unwrap();
        let turtle = CxTurtle {
            align_list_x_start_index: self.turtle_align_list.len(),
            align_list_y_start_index: self.turtle_align_list.len(),
            origin: parent.pos,
            pos: parent.pos,
            // fills out all remaining space by x axis
            layout: Layout { walk: Walk { width: Width::Fill, ..parent.layout.walk }, ..parent.layout },
            biggest: 0.0,
            bound_right_bottom: Vec2 { x: std::f32::NEG_INFINITY, y: std::f32::NEG_INFINITY },
            width: self.get_width_left(),
            height: self.get_height_left(),
            abs_size: parent.abs_size,
            turtle_type: CxTurtleType::CenterXAlign,
            available_width: parent.get_available_width_left(),
            available_height: parent.get_available_height_left(),
        };
        self.turtles.push(turtle);
    }

    pub fn end_center_x_align(&mut self) {
        self.assert_last_turtle_type_matches(CxTurtleType::CenterXAlign);

        let turtle = self.turtles.pop().unwrap();
        let dx = Cx::compute_align_turtle_x(&turtle, AlignX::CENTER);
        let align_start = turtle.align_list_x_start_index;
        self.do_align_x(dx, align_start);

        let parent = self.turtles.last_mut().unwrap();
        // TODO(Dmitry): communicating only few updates to parent for now. It's possible we need more.
        parent.bound_right_bottom.y = parent.bound_right_bottom.y.max(turtle.bound_right_bottom.y);
        parent.pos = turtle.pos;
    }

    // Start new box that will be on the right by x axis
    pub fn begin_right_box(&mut self) {
        let parent = self.turtles.last().unwrap();
        let turtle = CxTurtle {
            align_list_x_start_index: self.turtle_align_list.len(),
            align_list_y_start_index: self.turtle_align_list.len(),
            origin: parent.pos,
            pos: parent.pos,
            layout: parent.layout,
            biggest: 0.0,
            bound_right_bottom: Vec2 { x: std::f32::NEG_INFINITY, y: std::f32::NEG_INFINITY },
            width: parent.width,
            height: parent.height,
            abs_size: parent.abs_size,
            turtle_type: CxTurtleType::RightBox,
            available_width: parent.get_available_width_left(),
            available_height: parent.get_available_height_left(),
        };
        self.turtles.push(turtle);
    }

    pub fn end_right_box(&mut self) {
        self.assert_last_turtle_type_matches(CxTurtleType::RightBox);

        let turtle = self.turtles.pop().unwrap();
        let parent = self.turtles.last_mut().unwrap();

        let drawn_width = turtle.bound_right_bottom.x - turtle.origin.x;
        let last_x = parent.origin.x + parent.available_width;
        let dx = last_x - turtle.bound_right_bottom.x;
        // update parent
        parent.available_width -= drawn_width;
        parent.pos = turtle.origin;
        parent.bound_right_bottom.x = last_x;
        parent.bound_right_bottom.y = parent.bound_right_bottom.y.max(turtle.bound_right_bottom.y);

        let align_start = turtle.align_list_x_start_index;
        self.do_align_x(dx, align_start);
    }

    /// Starts a new box that adds padding to current turtle context
    pub fn begin_padding_box(&mut self, padding: Padding) {
        let parent = self.turtles.last().expect("Using padding_box without parent is not supported");
        let direction = parent.layout.direction;
        self.begin_typed_turtle(
            CxTurtleType::PaddingBox,
            Layout { direction, walk: Walk { width: Width::Compute, height: Height::Compute }, padding, ..Layout::default() },
        );
    }

    /// Ends the current box that was opened by [`Cx::begin_padding_box`]
    pub fn end_padding_box(&mut self) -> Rect {
        self.end_typed_turtle(CxTurtleType::PaddingBox)
    }

    /// Starts new box that is absolutely positioned at (0, 0) coordinate
    pub fn begin_absolute_box(&mut self) {
        self.begin_typed_turtle(CxTurtleType::AbsoluteBox, Layout { absolute: true, ..Layout::default() });
    }

    /// Ends the current box that was opened by [`Cx::begin_absolute_box`]
    pub fn end_absolute_box(&mut self) {
        self.end_typed_turtle(CxTurtleType::AbsoluteBox);
    }

    /// Starts new box that is wrapping its content inside.
    /// This is defined in terms of child boxes (e.g. if any of the immediately nested boxes
    /// goes beyond the bounds, it would be wrapped to new line).
    /// This is only supported for horizontal direction.
    /// Note: text has its own wrapping mechanism via [`TextInsProps::wrapping`].
    pub fn begin_wrapping_box(&mut self) {
        let parent = self.turtles.last().expect("Using wrapping_box without parent is not supported");
        let direction = parent.layout.direction;
        assert_eq!(direction, Direction::Right, "Wrapping is only supported for Direction::Right");
        self.begin_typed_turtle(
            CxTurtleType::WrappingBox,
            Layout {
                direction,
                line_wrap: LineWrap::Overflow,
                walk: Walk { width: Width::Compute, height: Height::Compute },
                ..Layout::default()
            },
        );
    }

    /// Ends the current box that was opened by [`Cx::begin_wrapping_box`]
    pub fn end_wrapping_box(&mut self) {
        self.end_typed_turtle(CxTurtleType::WrappingBox);
    }

    /// Returns the full rect corresponding to current box.
    /// It uses all available_width/height plus padding.
    /// Unlike [`Cx::get_turtle_rect`] for `Compute` widths and heights this will never contain `NaN`-s.
    pub fn get_box_rect(&self) -> Rect {
        if let Some(turtle) = self.turtles.last() {
            return Rect {
                pos: turtle.origin,
                size: vec2(turtle.available_width + turtle.layout.padding.r, turtle.available_height + turtle.layout.padding.b),
            };
        };
        Rect::default()
    }
}
