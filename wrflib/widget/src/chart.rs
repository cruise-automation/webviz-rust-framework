// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use std::{
    collections::HashMap,
    f32::{INFINITY, NEG_INFINITY},
};

use crate::*;
use wrflib::*;

/// TODO(hernan): Implement other chart types like bar, pie, etc...
pub enum ChartType {
    Line,
}

type ChartDatum = Vec2;

#[derive(Debug, Clone)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<ChartDatum>,
    pub point_background_color: Vec4,
    pub point_radius: f32,
    pub point_style: DrawPoints3dStyle,
    pub border_color: Vec4,
    pub border_width: f32,
    pub show_line: bool,
}

impl Default for ChartDataset {
    fn default() -> Self {
        Self {
            label: String::new(),
            data: vec![],
            point_background_color: COLOR_WHITE,
            point_radius: 10.,
            point_style: DrawPoints3dStyle::Circle,
            border_color: COLOR_WHITE,
            border_width: 2.,
            show_line: true,
        }
    }
}

pub struct ChartScale {
    pub min: f32,
    pub max: f32,
}

/// These options are based on the ones provided by ChartJS
pub struct ChartConfig {
    pub chart_type: ChartType,
    pub datasets: Vec<ChartDataset>,
    pub scales: HashMap<String, ChartScale>,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self { chart_type: ChartType::Line, datasets: vec![], scales: HashMap::new() }
    }
}

#[derive(Debug, Clone)]
pub struct ChartCurrentElement {
    pub dataset_index: usize,
    pub datum_index: usize,
    pub data_point: Vec2,
    pub normalized_data_point: Vec2,
}

#[derive(Debug, Clone)]
pub enum ChartEvent {
    None,
    FingerOut,
    FingerHover {
        /// Current mouse position relative to the chart boundaries.
        /// Useful for positioning tooltips.
        cursor: Vec2,
        /// The value at the current mouse position
        /// This value might not be part of the input data. Instead, it's interpolated
        /// based on normalized values.
        cursor_value: Vec2,
        /// If exists, we also retreive the element in the input data that is closest
        /// to the current mouse position
        current_element: Option<ChartCurrentElement>,
    },
}

#[derive(Default)]
pub struct Chart {
    bounds: Rect,
    min: Vec2,
    max: Vec2,
    areas: Vec<Area>,
}

impl Chart {
    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event, component_base: &ComponentBase) -> ChartEvent {
        match event.hits(cx, component_base, HitOpt::default()) {
            Event::FingerHover(fe) => {
                if fe.hover_state == HoverState::Out {
                    return ChartEvent::FingerOut;
                }

                let mouse_pos_rel = fe.rel;
                let cursor = mouse_pos_rel.clamp(&self.bounds.pos, &(self.bounds.pos + self.bounds.size));
                let cursor_value = self.denormalize_data_point(cursor);
                let current_element = self.get_element_at(cx, cursor_value);

                return ChartEvent::FingerHover { cursor, cursor_value, current_element };
            }
            _ => (),
        }

        ChartEvent::None
    }

    /// Gets the nearest element for a given value
    /// `stride` is used to jump between elements, in case they are represented by
    /// more than one value.
    fn get_nearest_element_index(data: &[DrawPoints3dInstance], value: f32) -> Option<usize> {
        if data.is_empty() {
            return None;
        }

        let partition_index = {
            let mut i = 0;
            // find the first element higher than the given value.
            while i < data.len() && data[i].user_info.x <= value {
                i += 1;
            }
            i
        };

        if partition_index >= data.len() {
            // The dataset cannot be partitioned, meaning the reference time is beyond
            // the available data (which might happen because the data is being loaded).
            // In this case, just return the last element, which should be the nearest
            // one since data is assumed to be sorted.
            return Some(data.len() - 1);
        }

        if partition_index == 0 {
            // All values are greater than the reference time, so just return the first
            // element.
            return Some(0);
        }

        // Compare values with the previous one
        if (data[partition_index].user_info.x - value).abs() >= (data[partition_index - 1].user_info.x - value).abs() {
            return Some(partition_index - 1);
        }

        Some(partition_index)
    }

    fn get_element_at(&self, cx: &mut Cx, cursor: Vec2) -> Option<ChartCurrentElement> {
        let mut ret: Option<ChartCurrentElement> = None;
        let mut min_distance = INFINITY;

        for dataset_index in 0..self.areas.len() {
            let area = &self.areas[dataset_index];
            let points = area.get_slice::<DrawPoints3dInstance>(cx);
            if let Some(datum_index) = Self::get_nearest_element_index(points, cursor.x) {
                let data_point = points[datum_index].user_info;
                let distance = (data_point.y - cursor.y).abs();
                if distance < min_distance {
                    ret = Some(ChartCurrentElement {
                        dataset_index,
                        datum_index,
                        data_point,
                        normalized_data_point: self.normalize_data_point(data_point),
                    });
                    min_distance = distance;
                }
            }
        }

        ret
    }

    pub(crate) fn get_bounds(&self) -> Rect {
        self.bounds
    }

    fn remap(value: f32, lo0: f32, hi0: f32, lo1: f32, hi1: f32) -> f32 {
        lo1 + (value - lo0) / (hi0 - lo0) * (hi1 - lo1)
    }

    fn draw_grid(&mut self, cx: &mut Cx) {
        let color = vec4(0.25, 0.25, 0.25, 1.);
        let scale = 1.;

        let min_x = self.bounds.pos.x;
        let max_x = min_x + self.bounds.size.x;
        let min_y = self.bounds.pos.y;
        let max_y = min_y + self.bounds.size.y;

        let mut lines = vec![];

        let columns = 4.;
        let rows = 10.;

        // TODO(hernan): Grids should be dynamic, adding or removing vertical/horizontal
        // divisions as needed depending on zoom and pan options. For now, just use 5 divisions for both.
        let step = (max_x - min_x) / columns;
        let mut x = min_x;
        while x <= max_x {
            lines.push(DrawLines3dInstance::from_segment(vec3(x, min_y, 0.), vec3(x, max_y + 10., 0.), color, scale));

            let col_value = Self::remap(x, min_x, max_x, self.min.x, self.max.x);

            TextIns::draw_str(
                cx,
                &format!("{:.0}", col_value),
                cx.get_turtle_origin() + Vec2 { x, y: max_y + 10. },
                &TextInsProps { position_anchoring: TEXT_ANCHOR_CENTER_H, ..TextInsProps::DEFAULT },
            );

            x += step;
        }

        let step = (max_y - min_y) / rows;
        let mut y = min_y;
        while y <= max_y {
            lines.push(DrawLines3dInstance::from_segment(vec3(min_x - 10., y, 0.), vec3(max_x, y, 0.), color, scale));

            // Flip min_y/max_y since y coordinate is inverted
            let row_value = Self::remap(y, max_y, min_y, self.min.y, self.max.y);

            TextIns::draw_str(
                cx,
                &format!("{:.0}", row_value),
                cx.get_turtle_origin() + Vec2 { x: min_x - 15., y },
                &TextInsProps { position_anchoring: TEXT_ANCHOR_RIGHT + TEXT_ANCHOR_CENTER_V, ..TextInsProps::DEFAULT },
            );

            y += step;
        }

        // Draw axes a bit brighter than columns/rows
        let axis_color = vec4(0.5, 0.5, 0.5, 1.);
        lines.push(DrawLines3dInstance::from_segment(vec3(min_x, max_y, 0.), vec3(max_x, max_y, 0.), axis_color, scale));
        lines.push(DrawLines3dInstance::from_segment(vec3(min_x, min_y, 0.), vec3(min_x, max_y, 0.), axis_color, scale));

        DrawLines3d::draw(cx, &lines);
    }

    fn draw_lines(&mut self, cx: &mut Cx, data: &[Vec2], color: Vec4, scale: f32) {
        let mut lines = vec![];
        for i in 0..(data.len() - 1) {
            let p0 = data[i].to_vec3();
            let p1 = data[i + 1].to_vec3();

            lines.push(DrawLines3dInstance::from_segment(p0, p1, color, scale));
        }

        DrawLines3d::draw(cx, &lines);
    }

    fn draw_points(
        &mut self,
        cx: &mut Cx,
        normalized_data: &[Vec2],
        original_data: &[Vec2],
        color: Vec4,
        scale: f32,
        point_style: DrawPoints3dStyle,
    ) -> Area {
        let color = color.to_vec3();
        let size = scale;
        let mut points = Vec::<DrawPoints3dInstance>::with_capacity(normalized_data.len());
        for i in 0..normalized_data.len() {
            points.push(DrawPoints3dInstance {
                position: normalized_data[i].to_vec3(),
                color,
                size,
                user_info: original_data[i],
            });
        }

        DrawPoints3d::draw(cx, &points, DrawPoints3dOptions { use_screen_space: true, point_style })
    }

    /// Transform a data point from data coordinates to normalized screen coordinates
    fn normalize_data_point(&self, data_point: Vec2) -> Vec2 {
        vec2(
            // For x axis, we want values to be in the range [bounds.pos.x, bounds.pos.x + bounds.size.x],
            // using (p.x - min.x) / (max.x - min.x) for interpolation.
            self.bounds.pos.x + (data_point.x - self.min.x) / (self.max.x - self.min.x) * self.bounds.size.x,
            // For y axis, it's a similar process except that we want charts to start at the bottom instead.
            // Then, we add bounds.size.y and subtract the interpolated value.
            (self.bounds.pos.y + self.bounds.size.y)
                - (data_point.y - self.min.y) / (self.max.y - self.min.y) * self.bounds.size.y,
        )
    }

    /// Transform a normalized data point from screen coordinates to data coordinates
    fn denormalize_data_point(&self, normalized_data_point: Vec2) -> Vec2 {
        vec2(
            Self::remap(
                normalized_data_point.x,
                self.bounds.pos.x,
                self.bounds.pos.x + self.bounds.size.x,
                self.min.x,
                self.max.x,
            ),
            Self::remap(
                normalized_data_point.y,
                self.bounds.pos.y,
                self.bounds.pos.y + self.bounds.size.y,
                self.max.y,
                self.min.y,
            ),
        )
    }

    fn normalize(&self, data: &[Vec2]) -> Vec<Vec2> {
        data.iter().map(|p| self.normalize_data_point(*p)).collect()
    }

    /// Rounds a number to the closest power of 10, rounded up
    fn round_up_to_10s(value: f32) -> f32 {
        let exp = value.log10().ceil();
        10_f32.powf(exp)
    }

    /// Rounds a number to the closest power of 10, rounded down
    fn round_down_to_10s(value: f32) -> f32 {
        let exp = value.log10().floor();
        10_f32.powf(exp)
    }

    fn round_up(value: Vec2) -> Vec2 {
        vec2(value.x, if value.y < 0. { -Self::round_down_to_10s(value.y.abs()) } else { Self::round_up_to_10s(value.y) })
    }

    fn round_down(value: Vec2) -> Vec2 {
        vec2(value.x, if value.y < 0. { -Self::round_up_to_10s(value.y.abs()) } else { Self::round_down_to_10s(value.y) })
    }

    fn get_min_max(config: &ChartConfig) -> (Vec2, Vec2) {
        let mut min = vec2(INFINITY, INFINITY);
        let mut max = vec2(NEG_INFINITY, NEG_INFINITY);

        if let Some(x_scale) = config.scales.get("x") {
            min.x = x_scale.min;
            max.x = x_scale.max;
        }

        for dataset in &config.datasets {
            for datum in &dataset.data {
                min = min.min(datum);
                max = max.max(datum);
            }
        }

        // Force either bound to be zero (but not both)
        if max.y < 0. {
            max.y = 0.;
        } else if min.y > 0. {
            min.y = 0.;
        }

        (Self::round_down(min), Self::round_up(max))
    }

    pub fn draw(&mut self, cx: &mut Cx, config: &ChartConfig) {
        let current_dpi = cx.current_dpi_factor;
        let measured_size = vec2(cx.get_width_total(), cx.get_height_total());
        self.bounds = Rect { pos: vec2(60., 5.), size: measured_size - vec2(80., 40.) };

        // Compute min/max for all datasets before rendering
        let (min, max) = Self::get_min_max(config);
        self.min = min;
        self.max = max;

        self.draw_grid(cx);

        self.areas = vec![];

        for dataset in &config.datasets {
            let normalized_data = self.normalize(&dataset.data);
            if !normalized_data.is_empty() {
                self.draw_lines(cx, &normalized_data, dataset.border_color, dataset.border_width * current_dpi);
                let area = self.draw_points(
                    cx,
                    &normalized_data,
                    &dataset.data,
                    dataset.point_background_color,
                    dataset.point_radius * current_dpi,
                    dataset.point_style.clone(),
                );
                self.areas.push(area);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use wrflib::vec2;

    use crate::Chart;

    #[test]
    fn it_rounds_up() {
        assert_eq!(Chart::round_up(vec2(10., 943.)), vec2(10., 1000.));
        assert_eq!(Chart::round_up(vec2(10., -943.)), vec2(10., -100.));
        assert_eq!(Chart::round_up(vec2(10., 478.)), vec2(10., 1000.));
        assert_eq!(Chart::round_up(vec2(10., 5623.)), vec2(10., 10000.));
        assert_eq!(Chart::round_up(vec2(10., -876.)), vec2(10., -100.));
        assert_eq!(Chart::round_up(vec2(10., 33.)), vec2(10., 100.));
        assert_eq!(Chart::round_up(vec2(10., 7.)), vec2(10., 10.));
        assert_eq!(Chart::round_up(vec2(10., -7.)), vec2(10., -1.));
        assert_eq!(Chart::round_up(vec2(10., 99.)), vec2(10., 100.));
        assert_eq!(Chart::round_up(vec2(10., -99.)), vec2(10., -10.));
        assert_eq!(Chart::round_up(vec2(10., 100.)), vec2(10., 100.));
        assert_eq!(Chart::round_up(vec2(10., -100.)), vec2(10., -100.));
        assert_eq!(Chart::round_up(vec2(10., 1001.)), vec2(10., 10000.));
        assert_eq!(Chart::round_up(vec2(10., -1001.)), vec2(10., -1000.));
    }

    #[test]
    fn it_rounds_down() {
        assert_eq!(Chart::round_down(vec2(10., 943.)), vec2(10., 100.));
        assert_eq!(Chart::round_down(vec2(10., -943.)), vec2(10., -1000.));
        assert_eq!(Chart::round_down(vec2(10., 478.)), vec2(10., 100.));
        assert_eq!(Chart::round_down(vec2(10., 5623.)), vec2(10., 1000.));
        assert_eq!(Chart::round_down(vec2(10., -876.)), vec2(10., -1000.));
        assert_eq!(Chart::round_down(vec2(10., 33.)), vec2(10., 10.));
        assert_eq!(Chart::round_down(vec2(10., 7.)), vec2(10., 1.));
        assert_eq!(Chart::round_down(vec2(10., -7.)), vec2(10., -10.));
        assert_eq!(Chart::round_down(vec2(10., 99.)), vec2(10., 10.));
        assert_eq!(Chart::round_down(vec2(10., -99.)), vec2(10., -100.));
        assert_eq!(Chart::round_down(vec2(10., 100.)), vec2(10., 100.));
        assert_eq!(Chart::round_down(vec2(10., -100.)), vec2(10., -100.));
        assert_eq!(Chart::round_down(vec2(10., 1001.)), vec2(10., 1000.));
        assert_eq!(Chart::round_down(vec2(10., -1001.)), vec2(10., -10000.));
    }
}
