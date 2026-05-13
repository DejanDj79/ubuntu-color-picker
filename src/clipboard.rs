use gtk::glib::object::IsA;
use gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::color::{CopyFormat, Rgb};

#[derive(Clone)]
pub struct CopyFeedback {
    toast: gtk::Box,
    toast_label: gtk::Label,
    generation: Rc<Cell<u32>>,
    duration_ms: Rc<Cell<u64>>,
}

impl CopyFeedback {
    pub fn new(toast: &gtk::Box, toast_label: &gtk::Label, duration_ms: u64) -> Self {
        Self {
            toast: toast.clone(),
            toast_label: toast_label.clone(),
            generation: Rc::new(Cell::new(0)),
            duration_ms: Rc::new(Cell::new(duration_ms)),
        }
    }

    pub fn set_duration_ms(&self, duration_ms: u64) {
        self.duration_ms.set(duration_ms);
    }

    pub fn show_message(&self, text: &str) {
        self.show(text);
    }

    fn show(&self, text: &str) {
        let generation = self.generation.get().wrapping_add(1);
        self.generation.set(generation);

        self.toast_label.set_text(text);
        self.toast.set_visible(true);

        let toast = self.toast.clone();
        let toast_label = self.toast_label.clone();
        let current_generation = self.generation.clone();
        let duration = Duration::from_millis(self.duration_ms.get());
        gtk::glib::timeout_add_local_once(duration, move || {
            if current_generation.get() == generation {
                toast_label.set_text("");
                toast.set_visible(false);
            }
        });
    }
}

pub fn copy_color(
    widget: &impl IsA<gtk::Widget>,
    feedback: &CopyFeedback,
    color: Rgb,
    format: CopyFormat,
) {
    let text = color.format(format);
    widget.clipboard().set_text(&text);

    feedback.show(&format!("Copied {}", text));
}
