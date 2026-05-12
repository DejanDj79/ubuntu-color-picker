use gtk::prelude::*;
use gtk::{Align, Button, Entry, Label, MenuButton, Orientation, Popover, Scale};
use std::rc::Rc;

use crate::color::CopyFormat;

pub fn icon_button(icon_name: &str, tooltip: &str) -> Button {
    let button = Button::builder()
        .width_request(34)
        .height_request(34)
        .tooltip_text(tooltip)
        .build();

    let icon = gtk::Image::from_icon_name(icon_name);
    button.set_child(Some(&icon));
    button.add_css_class("toolbar-button");

    button
}

pub fn menu_icon_button(icon_name: &str, tooltip: &str, popover: &Popover) -> MenuButton {
    let button = MenuButton::builder()
        .width_request(34)
        .height_request(34)
        .tooltip_text(tooltip)
        .has_frame(false)
        .build();

    button.set_icon_name(icon_name);
    button.set_popover(Some(popover));
    button.add_css_class("toolbar-button");

    button
}

pub fn window_button(label: &str) -> Button {
    let button = Button::builder()
        .label(label)
        .width_request(34)
        .height_request(30)
        .build();

    button.add_css_class("window-button");

    button
}

pub fn make_scale() -> Scale {
    let scale = Scale::with_range(Orientation::Horizontal, 0.0, 255.0, 1.0);
    scale.set_draw_value(false);
    scale.set_width_request(300);
    scale.add_css_class("rgb-slider");
    scale
}

pub fn make_value_entry(value: &str) -> Entry {
    let entry = Entry::builder()
        .text(value)
        .editable(true)
        .width_chars(4)
        .max_width_chars(4)
        .tooltip_text("Edit RGB value")
        .halign(Align::End)
        .build();

    add_copy_icon(&entry, "Copy RGB color");
    entry
}

pub fn make_hex_entry(value: &str) -> Entry {
    let entry = Entry::builder()
        .text(value)
        .editable(true)
        .width_chars(9)
        .max_width_chars(9)
        .tooltip_text("Edit HEX color")
        .halign(Align::End)
        .build();

    add_copy_icon(&entry, "Copy HEX color");
    entry
}

fn add_copy_icon(entry: &Entry, tooltip: &str) {
    entry.set_icon_from_icon_name(
        gtk::EntryIconPosition::Secondary,
        Some("edit-copy-symbolic"),
    );
    entry.set_icon_activatable(gtk::EntryIconPosition::Secondary, true);
    entry.set_icon_sensitive(gtk::EntryIconPosition::Secondary, true);
    entry.set_icon_tooltip_text(gtk::EntryIconPosition::Secondary, Some(tooltip));
}

pub fn copy_toast() -> (gtk::Box, Label) {
    let toast = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .halign(Align::Center)
        .valign(Align::End)
        .margin_bottom(14)
        .build();

    toast.add_css_class("copy-toast");

    let label = Label::builder()
        .width_chars(22)
        .max_width_chars(22)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .halign(Align::Center)
        .xalign(0.5)
        .build();

    label.add_css_class("copy-toast-label");
    toast.append(&label);
    toast.set_visible(false);

    (toast, label)
}

pub fn slider_row(
    label: &str,
    scale: &Scale,
    value_entry: &Entry,
    hex_entry: Option<&Entry>,
) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    let channel_label = Label::builder()
        .label(label)
        .width_request(18)
        .halign(Align::Center)
        .build();

    channel_label.add_css_class("channel-label");

    let value_box = gtk::Box::builder().width_request(64).build();
    value_box.append(value_entry);

    let hex_box = gtk::Box::builder().width_request(100).build();

    if let Some(hex) = hex_entry {
        hex_box.append(hex);
    }

    row.append(&channel_label);
    row.append(scale);
    row.append(&value_box);
    row.append(&hex_box);

    row
}

pub fn copy_format_popover(on_copy: Rc<dyn Fn(CopyFormat)>) -> Popover {
    let popover = Popover::builder().autohide(true).has_arrow(true).build();

    let list = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    for format in CopyFormat::ALL {
        let row = Button::builder()
            .label(format.label())
            .halign(Align::Fill)
            .build();

        row.add_css_class("copy-format-row");

        let popover = popover.clone();
        let on_copy = on_copy.clone();
        row.connect_clicked(move |_| {
            on_copy.as_ref()(format);
            popover.popdown();
        });

        list.append(&row);
    }

    popover.set_child(Some(&list));
    popover
}
