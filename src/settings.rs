use crate::clipboard::CopyFeedback;
use crate::storage::{ColorCollections, ToastDuration, PALETTE_LIMIT_OPTIONS};
use gtk::prelude::*;
use gtk::{Align, Button, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct SettingsPanel {
    storage: Rc<RefCell<ColorCollections>>,
    settings_button: Button,
    slide_revealer: gtk::Revealer,
    fade_revealer: gtk::Revealer,
    palette_limit_buttons: Vec<(usize, Button)>,
    toast_duration_buttons: Vec<(ToastDuration, Button)>,
    copy_feedback: CopyFeedback,
    on_storage_changed: Rc<dyn Fn()>,
}

impl SettingsPanel {
    pub fn new(
        storage: Rc<RefCell<ColorCollections>>,
        settings_button: Button,
        copy_feedback: CopyFeedback,
        on_storage_changed: Rc<dyn Fn()>,
    ) -> Self {
        let root = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        root.add_css_class("settings-root");

        root.append(&section_label("Palette limit"));
        let palette_limit_row = button_row();
        let mut palette_limit_buttons = Vec::new();
        for limit in PALETTE_LIMIT_OPTIONS {
            let button = option_button(&limit.to_string());
            palette_limit_row.append(&button);
            palette_limit_buttons.push((limit, button));
        }
        root.append(&palette_limit_row);

        root.append(&section_label("Toast duration"));
        let toast_duration_row = button_row();
        let mut toast_duration_buttons = Vec::new();
        for duration in ToastDuration::OPTIONS {
            let button = option_button(duration.label());
            toast_duration_row.append(&button);
            toast_duration_buttons.push((duration, button));
        }
        root.append(&toast_duration_row);

        root.append(&section_label("Data"));
        let data_row = button_row();
        let clear_palette_button = data_button("Clear palette");
        let clear_favorites_button = data_button("Clear favorites");
        data_row.append(&clear_palette_button);
        data_row.append(&clear_favorites_button);
        root.append(&data_row);

        let fade_revealer = gtk::Revealer::builder()
            .transition_duration(420)
            .transition_type(gtk::RevealerTransitionType::Crossfade)
            .build();
        fade_revealer.set_child(Some(&root));

        let slide_revealer = gtk::Revealer::builder()
            .transition_duration(480)
            .transition_type(gtk::RevealerTransitionType::SlideDown)
            .build();
        slide_revealer.set_child(Some(&fade_revealer));

        let panel = Self {
            storage,
            settings_button,
            slide_revealer,
            fade_revealer,
            palette_limit_buttons,
            toast_duration_buttons,
            copy_feedback,
            on_storage_changed,
        };

        {
            let panel = panel.clone();
            let settings_button = panel.settings_button.clone();
            settings_button.connect_clicked(move |_| {
                panel.toggle_panel();
            });
        }

        for (limit, button) in panel.palette_limit_buttons.clone() {
            let panel = panel.clone();
            button.connect_clicked(move |_| {
                panel.set_palette_limit(limit);
            });
        }

        for (duration, button) in panel.toast_duration_buttons.clone() {
            let panel = panel.clone();
            button.connect_clicked(move |_| {
                panel.set_toast_duration(duration);
            });
        }

        {
            let panel = panel.clone();
            clear_palette_button.connect_clicked(move |_| {
                panel.clear_palette();
            });
        }

        {
            let panel = panel.clone();
            clear_favorites_button.connect_clicked(move |_| {
                panel.clear_favorites();
            });
        }

        panel.refresh();
        panel
    }

    pub fn settings_widget(&self) -> gtk::Revealer {
        self.slide_revealer.clone()
    }

    fn set_palette_limit(&self, limit: usize) {
        {
            let mut storage = self.storage.borrow_mut();
            storage.set_palette_limit(limit);
            storage.save();
        }

        (self.on_storage_changed)();
        self.refresh();
        self.copy_feedback.show_message("Palette limit updated");
    }

    fn set_toast_duration(&self, duration: ToastDuration) {
        {
            let mut storage = self.storage.borrow_mut();
            storage.set_toast_duration(duration);
            storage.save();
        }

        self.copy_feedback.set_duration_ms(duration.millis());
        self.refresh();
        self.copy_feedback.show_message("Toast duration updated");
    }

    fn clear_palette(&self) {
        {
            let mut storage = self.storage.borrow_mut();
            storage.clear_palette();
            storage.save();
        }

        (self.on_storage_changed)();
        self.refresh();
        self.copy_feedback.show_message("Palette cleared");
    }

    fn clear_favorites(&self) {
        {
            let mut storage = self.storage.borrow_mut();
            storage.clear_favorites();
            storage.save();
        }

        (self.on_storage_changed)();
        self.refresh();
        self.copy_feedback.show_message("Favorites cleared");
    }

    fn refresh(&self) {
        let settings = self.storage.borrow().settings().clone();

        for (limit, button) in &self.palette_limit_buttons {
            set_active(button, settings.palette_limit() == *limit);
        }

        for (duration, button) in &self.toast_duration_buttons {
            set_active(button, settings.toast_duration() == *duration);
        }
    }

    fn toggle_panel(&self) {
        self.set_panel_visible(!self.slide_revealer.reveals_child());
    }

    fn set_panel_visible(&self, visible: bool) {
        self.fade_revealer.set_reveal_child(visible);
        self.slide_revealer.set_reveal_child(visible);

        if visible {
            self.settings_button.add_css_class("settings-active");
        } else {
            self.settings_button.remove_css_class("settings-active");
        }
    }
}

fn section_label(label: &str) -> Label {
    let label = Label::builder()
        .label(label)
        .halign(Align::Start)
        .xalign(0.0)
        .build();

    label.add_css_class("settings-section-label");
    label
}

fn button_row() -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .halign(Align::Start)
        .build();

    row.add_css_class("settings-button-row");
    row
}

fn option_button(label: &str) -> Button {
    let button = Button::builder().label(label).build();
    button.add_css_class("settings-option-button");
    button
}

fn data_button(label: &str) -> Button {
    let button = Button::builder().label(label).build();
    button.add_css_class("settings-data-button");
    button
}

fn set_active(button: &Button, active: bool) {
    if active {
        button.add_css_class("settings-option-active");
    } else {
        button.remove_css_class("settings-option-active");
    }
}
