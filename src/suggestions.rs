use crate::clipboard::{copy_color, CopyFeedback};
use crate::color::{CopyFormat, Rgb};
use gtk::prelude::*;
use gtk::{Align, Button, Label, Orientation};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
pub struct SuggestionsPanel {
    current_color: Rc<Cell<Rgb>>,
    panel_button: Button,
    slide_revealer: gtk::Revealer,
    fade_revealer: gtk::Revealer,
    content_box: gtk::Box,
    copy_feedback: CopyFeedback,
    on_select: Rc<dyn Fn(Rgb)>,
}

impl SuggestionsPanel {
    pub fn new(
        current_color: Rc<Cell<Rgb>>,
        panel_button: Button,
        copy_feedback: CopyFeedback,
        on_select: Rc<dyn Fn(Rgb)>,
    ) -> Self {
        let content_box = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();

        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .child(&content_box)
            .build();

        let root = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .width_request(238)
            .vexpand(true)
            .build();
        root.add_css_class("suggestions-root");
        root.append(&scroller);

        let fade_revealer = gtk::Revealer::builder()
            .transition_duration(420)
            .transition_type(gtk::RevealerTransitionType::Crossfade)
            .build();
        fade_revealer.set_child(Some(&root));

        let slide_revealer = gtk::Revealer::builder()
            .halign(Align::End)
            .valign(Align::Fill)
            .transition_duration(480)
            .transition_type(gtk::RevealerTransitionType::SlideRight)
            .build();
        slide_revealer.set_child(Some(&fade_revealer));

        let panel = Self {
            current_color,
            panel_button,
            slide_revealer,
            fade_revealer,
            content_box,
            copy_feedback,
            on_select,
        };

        {
            let panel = panel.clone();
            let panel_button = panel.panel_button.clone();
            panel_button.connect_clicked(move |_| {
                panel.toggle_panel();
            });
        }

        panel.refresh();
        panel
    }

    pub fn suggestions_widget(&self) -> gtk::Revealer {
        self.slide_revealer.clone()
    }

    pub fn set_current(&self, color: Rgb) {
        self.current_color.set(color);

        if self.slide_revealer.reveals_child() {
            self.refresh();
        }
    }

    fn toggle_panel(&self) {
        self.set_panel_visible(!self.slide_revealer.reveals_child());
    }

    fn set_panel_visible(&self, visible: bool) {
        if visible {
            self.refresh();
        }

        self.fade_revealer.set_reveal_child(visible);
        self.slide_revealer.set_reveal_child(visible);

        if visible {
            self.panel_button.add_css_class("suggestions-active");
        } else {
            self.panel_button.remove_css_class("suggestions-active");
        }
    }

    fn refresh(&self) {
        while let Some(child) = self.content_box.first_child() {
            self.content_box.remove(&child);
        }

        let color = self.current_color.get();
        self.content_box.append(&contrast_summary(color));

        for group in suggestion_groups(color) {
            self.content_box.append(&section_label(group.title));

            for color in group.colors {
                self.content_box
                    .append(&suggestion_row(color, self.clone()));
            }
        }
    }

    fn select_color(&self, color: Rgb) {
        (self.on_select)(color);
        self.set_current(color);
    }

    fn copy_color(&self, source: &Button, color: Rgb) {
        copy_color(source, &self.copy_feedback, color, CopyFormat::Hex);
    }
}

struct SuggestionGroup {
    title: &'static str,
    colors: Vec<Rgb>,
}

#[derive(Clone, Copy)]
struct Hsl {
    hue: f64,
    saturation: f64,
    lightness: f64,
}

fn suggestion_groups(color: Rgb) -> Vec<SuggestionGroup> {
    let hsl = rgb_to_hsl(color);

    vec![
        SuggestionGroup {
            title: "Complementary",
            colors: vec![
                hsl_to_rgb(hsl.with_hue_offset(180.0)),
                hsl_to_rgb(hsl.with_hue_offset(150.0)),
                hsl_to_rgb(hsl.with_hue_offset(210.0)),
            ],
        },
        SuggestionGroup {
            title: "Analogous",
            colors: [-30.0, -15.0, 15.0, 30.0]
                .iter()
                .map(|offset| hsl_to_rgb(hsl.with_hue_offset(*offset)))
                .collect(),
        },
        SuggestionGroup {
            title: "Pastels",
            colors: [-24.0, -8.0, 8.0, 24.0]
                .iter()
                .map(|offset| {
                    hsl_to_rgb(Hsl {
                        hue: wrap_hue(hsl.hue + offset),
                        saturation: (hsl.saturation * 0.42).clamp(0.18, 0.38),
                        lightness: (hsl.lightness * 0.35 + 0.62).clamp(0.68, 0.88),
                    })
                })
                .collect(),
        },
        SuggestionGroup {
            title: "Tints",
            colors: [0.18, 0.36, 0.54, 0.72]
                .iter()
                .map(|amount| mix(color, Rgb::new(255, 255, 255), *amount))
                .collect(),
        },
        SuggestionGroup {
            title: "Shades",
            colors: [0.18, 0.34, 0.50, 0.66]
                .iter()
                .map(|amount| mix(color, Rgb::new(0, 0, 0), *amount))
                .collect(),
        },
    ]
}

impl Hsl {
    fn with_hue_offset(self, offset: f64) -> Self {
        Self {
            hue: wrap_hue(self.hue + offset),
            ..self
        }
    }
}

fn contrast_summary(color: Rgb) -> gtk::Box {
    let white_ratio = contrast_ratio(color, Rgb::new(255, 255, 255));
    let black_ratio = contrast_ratio(color, Rgb::new(0, 0, 0));

    let box_ = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    box_.add_css_class("suggestions-contrast");

    box_.append(&section_label("Contrast"));
    box_.append(&contrast_row("White", white_ratio));
    box_.append(&contrast_row("Black", black_ratio));
    box_
}

fn contrast_row(label: &str, ratio: f64) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();

    let label = Label::builder()
        .label(label)
        .hexpand(true)
        .halign(Align::Start)
        .xalign(0.0)
        .build();
    label.add_css_class("suggestions-contrast-label");

    let value = Label::builder()
        .label(&format!("{:.1}:1 {}", ratio, contrast_grade(ratio)))
        .halign(Align::End)
        .xalign(1.0)
        .build();
    value.add_css_class("suggestions-contrast-value");

    row.append(&label);
    row.append(&value);
    row
}

fn suggestion_row(color: Rgb, panel: SuggestionsPanel) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    row.add_css_class("suggestion-color-row");

    let select_button = Button::builder().hexpand(true).halign(Align::Fill).build();
    select_button.add_css_class("suggestion-color-button");

    let content = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(Align::Fill)
        .build();

    let swatch = gtk::DrawingArea::builder()
        .width_request(18)
        .height_request(18)
        .build();

    swatch.set_draw_func(move |_, cr, width, height| {
        let radius = width.min(height) as f64 / 2.0 - 1.0;
        cr.arc(
            width as f64 / 2.0,
            height as f64 / 2.0,
            radius,
            0.0,
            std::f64::consts::TAU,
        );
        cr.set_source_rgb(
            color.r as f64 / 255.0,
            color.g as f64 / 255.0,
            color.b as f64 / 255.0,
        );
        let _ = cr.fill_preserve();
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.set_line_width(2.0);
        let _ = cr.stroke();
    });

    let label = Label::builder()
        .label(color.hex())
        .hexpand(true)
        .halign(Align::Start)
        .xalign(0.0)
        .build();
    label.add_css_class("suggestion-hex-label");

    content.append(&swatch);
    content.append(&label);
    select_button.set_child(Some(&content));

    {
        let panel = panel.clone();
        select_button.connect_clicked(move |_| {
            panel.select_color(color);
        });
    }

    let copy_button = Button::builder()
        .width_request(28)
        .height_request(28)
        .tooltip_text("Copy HEX color")
        .build();
    copy_button.set_child(Some(&gtk::Image::from_icon_name("edit-copy-symbolic")));
    copy_button.add_css_class("suggestion-copy");

    {
        let panel = panel.clone();
        copy_button.connect_clicked(move |button| {
            panel.copy_color(button, color);
        });
    }

    row.append(&select_button);
    row.append(&copy_button);
    row
}

fn section_label(label: &str) -> Label {
    let label = Label::builder()
        .label(label)
        .halign(Align::Start)
        .xalign(0.0)
        .build();

    label.add_css_class("suggestions-section-label");
    label
}

fn rgb_to_hsl(color: Rgb) -> Hsl {
    let r = color.r as f64 / 255.0;
    let g = color.g as f64 / 255.0;
    let b = color.b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let lightness = (max + min) / 2.0;

    if delta == 0.0 {
        return Hsl {
            hue: 0.0,
            saturation: 0.0,
            lightness,
        };
    }

    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue = if max == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    Hsl {
        hue: wrap_hue(hue),
        saturation,
        lightness,
    }
}

fn hsl_to_rgb(hsl: Hsl) -> Rgb {
    let chroma = (1.0 - (2.0 * hsl.lightness - 1.0).abs()) * hsl.saturation;
    let hue_prime = hsl.hue / 60.0;
    let x = chroma * (1.0 - (hue_prime.rem_euclid(2.0) - 1.0).abs());

    let (r1, g1, b1) = match hue_prime as i32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };

    let m = hsl.lightness - chroma / 2.0;
    Rgb::new(
        channel_from_float(r1 + m),
        channel_from_float(g1 + m),
        channel_from_float(b1 + m),
    )
}

fn mix(source: Rgb, target: Rgb, amount: f64) -> Rgb {
    let amount = amount.clamp(0.0, 1.0);
    Rgb::new(
        mix_channel(source.r, target.r, amount),
        mix_channel(source.g, target.g, amount),
        mix_channel(source.b, target.b, amount),
    )
}

fn mix_channel(source: u8, target: u8, amount: f64) -> u8 {
    ((source as f64 * (1.0 - amount)) + (target as f64 * amount)).round() as u8
}

fn channel_from_float(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn wrap_hue(hue: f64) -> f64 {
    hue.rem_euclid(360.0)
}

fn contrast_grade(ratio: f64) -> &'static str {
    if ratio >= 7.0 {
        "AAA"
    } else if ratio >= 4.5 {
        "AA"
    } else if ratio >= 3.0 {
        "AA Large"
    } else {
        "Fail"
    }
}

fn contrast_ratio(a: Rgb, b: Rgb) -> f64 {
    let a_luminance = relative_luminance(a);
    let b_luminance = relative_luminance(b);
    let lighter = a_luminance.max(b_luminance);
    let darker = a_luminance.min(b_luminance);

    (lighter + 0.05) / (darker + 0.05)
}

fn relative_luminance(color: Rgb) -> f64 {
    0.2126 * linear_channel(color.r)
        + 0.7152 * linear_channel(color.g)
        + 0.0722 * linear_channel(color.b)
}

fn linear_channel(channel: u8) -> f64 {
    let value = channel as f64 / 255.0;

    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}
