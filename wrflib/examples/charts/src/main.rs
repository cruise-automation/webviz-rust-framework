use wrflib::*;
use wrflib_widget::*;

mod chart_list;
use chart_list::*;

mod lines_basic;
use lines_basic::*;
mod tooltip_custom;
use tooltip_custom::*;

pub(crate) trait ChartExample {
    fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ChartEvent;
    fn draw(&mut self, cx: &mut Cx);
}

pub struct ChartsExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    splitter: Splitter,
    chart_list: ChartList,
    chart: Box<dyn ChartExample>,
}

impl ChartsExampleApp {
    pub fn new(_: &mut Cx) -> Self {
        let mut splitter = Splitter::new();
        splitter.set_splitter_state(SplitterAlign::First, 300., Axis::Vertical);
        Self {
            window: Window { create_inner_size: Some(vec2(1000., 700.)), ..Window::default() },
            pass: Pass::default(),
            main_view: View::default(),
            splitter,
            chart_list: ChartList::with_items(vec!["Lines", "Lines - Dark", "Lines - Styling (TODO)", "Tooltip - Custom"]),
            chart: Box::new(LinesBasic::default()),
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        match self.splitter.handle(cx, event) {
            SplitterEvent::Moving { .. } => {
                cx.request_draw();
            }
            _ => (),
        }

        if let ChartListEvent::ChartSelected(selected) = self.chart_list.handle(cx, event) {
            match selected {
                "Lines" => self.chart = Box::new(LinesBasic::default()),
                "Lines - Dark" => self.chart = Box::new(LinesBasic::with_dark_style()),
                "Tooltip - Custom" => self.chart = Box::new(TooltipCustomExample::default()),
                _ => (),
            }
            cx.request_draw();
        }

        self.chart.handle(cx, event);
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, COLOR_WHITE);
        self.main_view.begin_view(cx, Layout::default());

        let turtle = cx.begin_turtle(Layout { direction: Direction::Right, ..Layout::default() });

        self.splitter.begin_draw(cx);
        self.chart_list.draw(cx);
        self.splitter.mid_draw(cx);
        self.chart.draw(cx);
        self.splitter.end_draw(cx);

        cx.end_turtle(turtle);

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
        cx.request_draw();
    }
}

main_app!(ChartsExampleApp);
