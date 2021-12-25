use wrflib::*;
use wrflib_widget::*;

use crate::ChartExample;

pub(crate) struct LinesBasic {
    pub(crate) chart: Chart,
    pub(crate) datasets: Vec<Vec<f32>>,
    pub(crate) randomize_btn: NormalButton,
    pub(crate) add_dataset_btn: NormalButton,
    pub(crate) add_data_btn: NormalButton,
    pub(crate) remove_dataset_btn: NormalButton,
    pub(crate) remove_data_btn: NormalButton,
    pub(crate) style: ChartStyle,
    pub(crate) tooltip: ChartTooltipConfig,
}

impl Default for LinesBasic {
    fn default() -> Self {
        let mut ret = Self {
            chart: Chart::default(),
            datasets: vec![],
            randomize_btn: NormalButton::default(),
            add_dataset_btn: NormalButton::default(),
            add_data_btn: NormalButton::default(),
            remove_dataset_btn: NormalButton::default(),
            remove_data_btn: NormalButton::default(),
            style: CHART_STYLE_LIGHT,
            tooltip: Default::default(),
        };

        // Add two initial datasets
        ret.add_dataset();
        ret.add_dataset();

        ret
    }
}

impl LinesBasic {
    pub(crate) fn with_dark_style() -> Self {
        Self { style: CHART_STYLE_DARK, ..Self::default() }
    }

    fn get_random_data(count: usize) -> Vec<f32> {
        if count == 0 {
            vec![]
        } else {
            (0..count).into_iter().map(|_| -100. + 200. * (universal_rand::random_128() as f32 / f32::MAX)).collect()
        }
    }

    fn randomize(&mut self) {
        if self.datasets.is_empty() {
            return;
        }

        let data_count = self.datasets[0].len();
        for data in &mut self.datasets {
            *data = Self::get_random_data(data_count);
        }
    }

    fn add_dataset(&mut self) {
        let data_count = {
            if self.datasets.is_empty() {
                7 // some arbitrary size
            } else {
                self.datasets[0].len()
            }
        };
        self.datasets.push(Self::get_random_data(data_count));
    }

    fn add_data(&mut self) {
        for data in &mut self.datasets {
            data.push(Self::get_random_data(1)[0])
        }
    }

    fn remove_dataset(&mut self) {
        if self.datasets.is_empty() {
            return;
        }

        self.datasets.pop();
    }

    fn remove_data(&mut self) {
        for data in &mut self.datasets {
            data.pop();
        }
    }

    fn draw_chart(&mut self, cx: &mut Cx) {
        let turtle = cx.begin_turtle(Layout {
            direction: Direction::Right,
            walk: Walk { width: Width::Fill, height: Height::Fix(cx.get_height_left() - 70.) },
            padding: Padding::top(20.),
            ..Layout::default()
        });

        let colors = vec![COLOR_RED, COLOR_ORANGE, COLOR_YELLOW, COLOR_GREEN, COLOR_BLUE, COLOR_PURPLE, COLOR_GRAY];
        let months = vec![
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];

        let datasets: Vec<ChartDataset> = self
            .datasets
            .iter()
            .enumerate()
            .map(|(i, data)| ChartDataset {
                label: format!("Dataset {}", i),
                data: ChartData::from_values(data),
                point_background_color: colors[i % colors.len()],
                point_radius: 4.,
                border_color: colors[i % colors.len()],
                border_width: 2.,
                ..ChartDataset::default()
            })
            .collect();

        // Generate labels
        let mut labels = vec![];
        if let Some(data_count) = datasets.iter().map(|ds| ds.data.len()).max() {
            for i in 0..data_count {
                labels.push(months[i % months.len()].to_string());
            }
        }

        let config = ChartConfig {
            labels,
            chart_type: ChartType::Line,
            datasets,
            style: self.style.clone(),
            tooltip: self.tooltip.clone(),
            ..ChartConfig::default()
        };

        self.chart.draw(cx, &config);

        cx.end_turtle(turtle);
    }

    pub fn draw_bottom_bar(&mut self, cx: &mut Cx) {
        let turtle = cx.begin_turtle(Layout {
            direction: Direction::Right,
            walk: Walk { width: Width::Fill, height: Height::Fix(50.) },
            ..Layout::default()
        });

        self.randomize_btn.draw(cx, "Randomize");
        self.add_dataset_btn.draw(cx, "Add Dataset");
        self.add_data_btn.draw(cx, "Add Data");
        self.remove_dataset_btn.draw(cx, "Remove Dataset");
        self.remove_data_btn.draw(cx, "Remove Data");

        cx.end_turtle(turtle);
    }
}

impl ChartExample for LinesBasic {
    fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ChartEvent {
        if let ButtonEvent::Clicked = self.randomize_btn.handle(cx, event) {
            self.randomize();
        }

        if let ButtonEvent::Clicked = self.add_dataset_btn.handle(cx, event) {
            self.add_dataset();
        }

        if let ButtonEvent::Clicked = self.add_data_btn.handle(cx, event) {
            self.add_data();
        }

        if let ButtonEvent::Clicked = self.remove_dataset_btn.handle(cx, event) {
            self.remove_dataset();
        }

        if let ButtonEvent::Clicked = self.remove_data_btn.handle(cx, event) {
            self.remove_data();
        }

        self.chart.handle(cx, event)
    }

    fn draw(&mut self, cx: &mut Cx) {
        let turtle = cx.begin_turtle(Layout { direction: Direction::Down, ..Layout::default() });

        self.draw_chart(cx);
        self.draw_bottom_bar(cx);

        cx.end_turtle(turtle);
    }
}
