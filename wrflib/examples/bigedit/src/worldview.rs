// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

// a bunch o buttons to select the world
use crate::fieldworld::FieldWorld;
use crate::treeworld::TreeWorld;
use wrflib::*;
use wrflib_widget::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Ord)]
enum WorldType {
    TreeWorld,
    FieldWorld,
}

impl WorldType {
    fn name(&self) -> String {
        match self {
            Self::TreeWorld => "TreeWorld".to_string(),
            Self::FieldWorld => "FieldWorld".to_string(),
        }
    }
}

const WORLD_TYPES: &[WorldType] = &[WorldType::TreeWorld, WorldType::FieldWorld];

pub struct WorldView {
    select_view: ScrollView,
    buttons: Vec<NormalButton>,
    view: View,
    bg: Background,
    xr_is_presenting: bool,
    viewport_3d: Viewport3D,
    world_type: WorldType,
    tree_world: TreeWorld,
    field_world: FieldWorld,
}

const COLOR_BG: Vec4 = vec4(34.0 / 255.0, 34.0 / 255.0, 34.0 / 255.0, 1.0);

impl WorldView {
    pub fn new() -> Self {
        Self {
            view: View::default(),
            bg: Background::default(),
            select_view: ScrollView::new_standard_hv(),
            viewport_3d: Viewport3D::default(),
            buttons: WORLD_TYPES.iter().map(|_| NormalButton::default()).collect(),
            world_type: WorldType::TreeWorld,
            xr_is_presenting: false,
            tree_world: TreeWorld::default(),
            field_world: FieldWorld::default(),
        }
    }

    pub fn handle_world_select(&mut self, cx: &mut Cx, event: &mut Event) {
        if self.select_view.handle(cx, event) {}
        for (index, btn) in self.buttons.iter_mut().enumerate() {
            if let ButtonEvent::Clicked = btn.handle(cx, event) {
                self.world_type = WORLD_TYPES[index];
                cx.request_draw();
            }
        }
    }

    pub fn draw_world_select(&mut self, cx: &mut Cx) {
        self.select_view.begin_view(cx, Layout::default());
        let turtle = self.bg.begin_turtle(cx, Layout::default(), COLOR_BG);

        for (index, button) in self.buttons.iter_mut().enumerate() {
            button.draw(cx, &WORLD_TYPES[index].name());
        }

        self.bg.end_turtle(cx, turtle);
        self.select_view.end_view(cx);
    }

    pub fn handle_world_view(&mut self, cx: &mut Cx, event: &mut Event) {
        // do 2D camera interaction.
        if !self.xr_is_presenting {
            self.viewport_3d.handle_viewport_2d(cx, event);
        }

        match &self.world_type {
            WorldType::TreeWorld => {
                self.tree_world.handle(cx, event);
            }
            WorldType::FieldWorld => {
                self.field_world.handle(cx, event);
            }
        }
    }

    pub fn draw_world_view_2d(&mut self, cx: &mut Cx) {
        self.viewport_3d.begin_viewport_3d(cx);
        self.draw_world_view_3d(cx);
        self.viewport_3d.end_viewport_3d(cx);
    }

    pub fn draw_world_view_3d(&mut self, cx: &mut Cx) {
        self.view.begin_view(cx, Layout::abs_origin_zero());

        match &self.world_type {
            WorldType::TreeWorld => {
                self.tree_world.draw(cx);
            }
            WorldType::FieldWorld => {
                self.field_world.draw(cx);
            }
        }

        self.view.end_view(cx);
    }
}
