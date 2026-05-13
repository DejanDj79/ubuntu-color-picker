use crate::clipboard::{copy_color, CopyFeedback};
use crate::color::{CopyFormat, Rgb};
use crate::storage::ColorCollections;
use gtk::prelude::*;
use gtk::{Align, Button, Label, Orientation, Popover};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Clone)]
pub struct SavedColorsPanel {
    storage: Rc<RefCell<ColorCollections>>,
    current_color: Rc<Cell<Rgb>>,
    popover: Popover,
    favorite_button: Button,
    favorites_box: gtk::Box,
    palette_root: gtk::Box,
    palette_colors_box: gtk::Box,
    copy_feedback: CopyFeedback,
    on_select: Rc<dyn Fn(Rgb)>,
}

impl SavedColorsPanel {
    pub fn new(
        popover: Popover,
        storage: Rc<RefCell<ColorCollections>>,
        current_color: Rc<Cell<Rgb>>,
        favorite_button: Button,
        copy_feedback: CopyFeedback,
        on_select: Rc<dyn Fn(Rgb)>,
    ) -> Self {
        let root = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        root.add_css_class("saved-colors-root");

        let favorites_label = section_label("Favorites");
        let favorites_box = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .build();

        root.append(&favorites_label);
        root.append(&favorites_box);
        popover.set_child(Some(&root));

        let palette_root = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .height_request(36)
            .build();
        palette_root.add_css_class("palette-strip");

        let palette_label = Label::builder()
            .label("Palette")
            .halign(Align::Start)
            .xalign(0.0)
            .build();
        palette_label.add_css_class("palette-label");

        let add_palette_button = Button::builder()
            .width_request(30)
            .height_request(30)
            .tooltip_text("Add to palette")
            .build();
        let add_palette_icon = gtk::Image::from_icon_name("list-add-symbolic");
        add_palette_button.set_child(Some(&add_palette_icon));
        add_palette_button.add_css_class("palette-add-button");

        let palette_colors_box = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .hexpand(true)
            .halign(Align::Start)
            .build();

        palette_root.append(&palette_label);
        palette_root.append(&add_palette_button);
        palette_root.append(&palette_colors_box);

        let panel = Self {
            storage,
            current_color,
            popover,
            favorite_button,
            favorites_box,
            palette_root,
            palette_colors_box,
            copy_feedback,
            on_select,
        };

        {
            let panel = panel.clone();
            let favorite_button = panel.favorite_button.clone();
            favorite_button.connect_clicked(move |_| {
                panel.toggle_current_favorite();
            });
        }

        {
            let panel = panel.clone();
            add_palette_button.connect_clicked(move |_| {
                panel.add_current_to_palette();
            });
        }

        panel.refresh();
        panel
    }

    pub fn palette_widget(&self) -> gtk::Box {
        self.palette_root.clone()
    }

    pub fn set_current(&self, color: Rgb) {
        self.current_color.set(color);
        self.refresh();
    }

    pub fn toggle_current_favorite(&self) {
        let color = self.current_color.get();

        {
            let mut storage = self.storage.borrow_mut();
            storage.toggle_favorite(color);
            storage.save();
        }

        self.refresh();
    }

    fn add_current_to_palette(&self) {
        let color = self.current_color.get();

        {
            let mut storage = self.storage.borrow_mut();
            storage.add_palette_color(color);
            storage.save();
        }

        self.refresh();
    }

    fn remove_favorite(&self, color: Rgb) {
        {
            let mut storage = self.storage.borrow_mut();
            storage.remove_favorite(color);
            storage.save();
        }

        self.refresh();
    }

    fn refresh(&self) {
        let current = self.current_color.get();
        let storage = self.storage.borrow();

        if storage.is_favorite(current) {
            self.favorite_button
                .set_tooltip_text(Some("Remove favorite"));
            self.favorite_button.add_css_class("favorite-active");
            set_button_icon(&self.favorite_button, "starred-symbolic");
        } else {
            self.favorite_button.set_tooltip_text(Some("Add favorite"));
            self.favorite_button.remove_css_class("favorite-active");
            set_button_icon(&self.favorite_button, "non-starred-symbolic");
        }

        populate_color_list(
            &self.favorites_box,
            storage.favorites(),
            "No favorites yet",
            self.clone(),
            true,
        );
        populate_palette(&self.palette_colors_box, storage.palette(), self.clone());
    }

    fn select_color(&self, color: Rgb) {
        (self.on_select)(color);
        self.set_current(color);
        self.popover.popdown();
    }

    fn copy_color(&self, source: &Button, color: Rgb) {
        copy_color(source, &self.copy_feedback, color, CopyFormat::Hex);
    }
}

fn set_button_icon(button: &Button, icon_name: &str) {
    let icon = gtk::Image::from_icon_name(icon_name);
    button.set_child(Some(&icon));
}

fn section_label(label: &str) -> Label {
    let label = Label::builder()
        .label(label)
        .halign(Align::Start)
        .xalign(0.0)
        .build();

    label.add_css_class("saved-section-label");
    label
}

fn populate_color_list(
    container: &gtk::Box,
    colors: &[Rgb],
    empty_text: &str,
    panel: SavedColorsPanel,
    can_remove: bool,
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    if colors.is_empty() {
        let empty = Label::builder()
            .label(empty_text)
            .halign(Align::Start)
            .xalign(0.0)
            .build();
        empty.add_css_class("saved-empty-label");
        container.append(&empty);
        return;
    }

    for color in colors {
        container.append(&color_row(*color, panel.clone(), can_remove));
    }
}

fn populate_palette(container: &gtk::Box, colors: &[Rgb], panel: SavedColorsPanel) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    if colors.is_empty() {
        let empty = Label::builder()
            .label("Empty")
            .halign(Align::Start)
            .xalign(0.0)
            .build();
        empty.add_css_class("palette-empty-label");
        container.append(&empty);
        return;
    }

    for color in colors {
        container.append(&palette_color_button(*color, panel.clone()));
    }
}

fn palette_color_button(color: Rgb, panel: SavedColorsPanel) -> Button {
    let button = Button::builder()
        .width_request(28)
        .height_request(28)
        .tooltip_text(color.hex())
        .build();

    button.add_css_class("palette-color-button");

    let swatch = gtk::DrawingArea::builder()
        .width_request(18)
        .height_request(18)
        .build();

    swatch.set_draw_func(move |_, cr, width, height| {
        cr.set_source_rgb(
            color.r as f64 / 255.0,
            color.g as f64 / 255.0,
            color.b as f64 / 255.0,
        );
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        let _ = cr.fill();
    });

    button.set_child(Some(&swatch));

    {
        let panel = panel.clone();
        button.connect_clicked(move |_| {
            panel.select_color(color);
        });
    }

    button
}

fn color_row(color: Rgb, panel: SavedColorsPanel, can_remove: bool) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();

    row.add_css_class("saved-color-row");

    let select_button = Button::builder().hexpand(true).halign(Align::Fill).build();
    select_button.add_css_class("saved-color-button");

    let row_content = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(Align::Fill)
        .build();

    let swatch = gtk::DrawingArea::builder()
        .width_request(18)
        .height_request(18)
        .build();

    swatch.set_draw_func(move |_, cr, width, height| {
        cr.set_source_rgb(
            color.r as f64 / 255.0,
            color.g as f64 / 255.0,
            color.b as f64 / 255.0,
        );
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        let _ = cr.fill();
    });

    let label = Label::builder()
        .label(color.hex())
        .hexpand(true)
        .halign(Align::Start)
        .xalign(0.0)
        .build();

    row_content.append(&swatch);
    row_content.append(&label);
    select_button.set_child(Some(&row_content));

    {
        let panel = panel.clone();
        select_button.connect_clicked(move |_| {
            panel.select_color(color);
        });
    }

    let copy_button = Button::builder()
        .width_request(30)
        .height_request(30)
        .tooltip_text("Copy HEX color")
        .build();

    let copy_icon = gtk::Image::from_icon_name("edit-copy-symbolic");
    copy_button.set_child(Some(&copy_icon));
    copy_button.add_css_class("saved-color-copy");

    {
        let panel = panel.clone();
        copy_button.connect_clicked(move |button| {
            panel.copy_color(button, color);
        });
    }

    row.append(&select_button);
    row.append(&copy_button);

    if can_remove {
        let delete_button = Button::builder()
            .width_request(30)
            .height_request(30)
            .tooltip_text("Remove favorite")
            .build();

        let delete_icon = gtk::Image::from_icon_name("edit-delete-symbolic");
        delete_button.set_child(Some(&delete_icon));
        delete_button.add_css_class("saved-color-delete");

        {
            let panel = panel.clone();
            delete_button.connect_clicked(move |_| {
                panel.remove_favorite(color);
            });
        }

        row.append(&delete_button);
    }

    row
}
