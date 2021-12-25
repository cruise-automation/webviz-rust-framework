use wrflib::*;
use wrflib_widget::*;

struct ChartItem {
    name: String,
    checked: bool,
    checkbox: Checkbox,
}

impl ChartItem {
    fn with_name(name: &str) -> Self {
        Self { name: String::from(name), checked: false, checkbox: Checkbox::default() }
    }
}

pub(crate) enum ChartListEvent<'a> {
    None,
    ChartSelected(&'a str),
}

pub(crate) struct ChartList {
    background: Background,
    view: View,
    items: Vec<ChartItem>,
}

impl ChartList {
    pub fn with_items(items: Vec<&str>) -> Self {
        let mut items: Vec<_> = items.iter().map(|name| ChartItem::with_name(name)).collect();
        items[0].checked = true;

        Self { background: Background::default(), view: View::default(), items }
    }
}

impl ChartList {
    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ChartListEvent {
        let mut last_selected = -1;
        let mut selected = -1;
        for i in 0..self.items.len() {
            let item = &mut self.items[i];
            if let CheckboxEvent::Toggled = item.checkbox.handle(cx, event) {
                if item.checked {
                    last_selected = i as i32;
                }
                item.checked = !item.checked;
                if item.checked {
                    selected = i as i32;
                }
            }
        }

        if selected < 0 {
            selected = last_selected;
        }

        if selected >= 0 {
            for i in 0..self.items.len() {
                self.items[i].checked = i == selected as usize;
            }
            cx.request_draw();
            return ChartListEvent::ChartSelected(&self.items[selected as usize].name);
        }

        ChartListEvent::None
    }

    fn draw_item_list(&mut self, cx: &mut Cx) {
        let turtle = cx.begin_turtle(Layout { direction: Direction::Down, padding: Padding::top(20.), ..Layout::default() });

        for item in &mut self.items {
            item.checkbox.draw(cx, item.checked, true, false, &item.name, 0.);
        }

        cx.end_turtle(turtle);
    }

    pub(crate) fn draw(&mut self, cx: &mut Cx) {
        self.view.begin_view(cx, Layout { direction: Direction::Down, ..Layout::default() });

        self.background.draw(cx, cx.get_turtle_rect(), COLOR_DARKSLATEGRAY);
        self.draw_item_list(cx);

        self.view.end_view(cx);
    }
}
