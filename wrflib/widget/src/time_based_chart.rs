// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::*;
use wrflib::*;

#[derive(Default)]
pub struct TimeBasedChart {
    base: Chart,
    hover_time_line: Option<DrawLines3dInstance>,
}

impl TimeBasedChart {
    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event, component_base: &ComponentBase) -> ChartEvent {
        let event = self.base.handle(cx, event, component_base);
        match event {
            ChartEvent::FingerHover { cursor, .. } => {
                let bounds = &self.base.get_bounds();
                let hover_time_x = cursor.x;
                let y0 = bounds.pos.y + bounds.size.y;
                let y1 = bounds.pos.y;
                self.hover_time_line = Some(DrawLines3dInstance::from_segment(
                    vec3(hover_time_x, y0, 0.),
                    vec3(hover_time_x, y1, 0.),
                    COLOR_ORANGE,
                    1.,
                ));
                cx.request_draw();
            }
            _ => (),
        }
        event
    }

    pub fn draw(&mut self, cx: &mut Cx, config: &ChartConfig, time: f32) {
        self.base.draw(cx, config);

        let mut lines = vec![];
        if let Some(time_scale) = config.scales.get("time") {
            let start_time = time_scale.min;
            let end_time = time_scale.max;

            let bounds = &self.base.get_bounds();

            let x = bounds.pos.x + (time - start_time) / (end_time - start_time) * bounds.size.x;
            let y0 = bounds.pos.y + bounds.size.y;
            let y1 = bounds.pos.y;

            let p0 = vec3(x, y0, 0.);
            let p1 = vec3(x, y1, 0.);
            let line = DrawLines3dInstance::from_segment(p0, p1, COLOR_WHITE, 1.);
            lines.push(line);
        }

        if let Some(hover_time_line) = &self.hover_time_line {
            lines.push(hover_time_line.clone());

            let arrow_pointer_size = vec2(10., 10.);

            ArrowPointerIns::draw(
                cx,
                hover_time_line.position_start.to_vec2() - vec2(0., arrow_pointer_size.y - 1.),
                COLOR_ORANGE,
                ArrowPointerDirection::Up,
                arrow_pointer_size,
            );

            ArrowPointerIns::draw(
                cx,
                hover_time_line.position_end.to_vec2() + vec2(0., arrow_pointer_size.y - 1.),
                COLOR_ORANGE,
                ArrowPointerDirection::Down,
                arrow_pointer_size,
            );
        }

        DrawLines3d::draw(cx, &lines);
    }
}
