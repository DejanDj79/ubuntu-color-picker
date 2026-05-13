use crate::clipboard::{copy_color, CopyFeedback};
use crate::color::{CopyFormat, Rgb};
use crate::storage::ColorCollections;
use gtk::prelude::*;
use gtk::{Align, Button, Label, Orientation};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const PALETTE_COLUMNS: usize = 12;

#[derive(Clone)]
pub struct SavedColorsPanel {
    storage: Rc<RefCell<ColorCollections>>,
    current_color: Rc<Cell<Rgb>>,
    favorite_button: Button,
    palette_button: Button,
    panel_button: Button,
    slide_revealer: gtk::Revealer,
    fade_revealer: gtk::Revealer,
    favorites_box: gtk::Box,
    palette_root: gtk::Box,
    palette_grid: gtk::Grid,
    copy_feedback: CopyFeedback,
    on_select: Rc<dyn Fn(Rgb)>,
}

impl SavedColorsPanel {
    pub fn new(
        storage: Rc<RefCell<ColorCollections>>,
        current_color: Rc<Cell<Rgb>>,
        favorite_button: Button,
        palette_button: Button,
        panel_button: Button,
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

        let palette_root = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .build();
        palette_root.add_css_class("palette-strip");

        let palette_label = Label::builder()
            .label("Recently used")
            .halign(Align::Start)
            .xalign(0.0)
            .build();
        palette_label.add_css_class("palette-label");

        let palette_grid = gtk::Grid::builder()
            .column_spacing(6)
            .row_spacing(6)
            .halign(Align::Start)
            .build();

        palette_root.append(&palette_label);
        palette_root.append(&palette_grid);

        let panel = Self {
            storage,
            current_color,
            favorite_button,
            palette_button,
            panel_button,
            slide_revealer,
            fade_revealer,
            favorites_box,
            palette_root,
            palette_grid,
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
            let palette_button = panel.palette_button.clone();
            palette_button.connect_clicked(move |_| {
                panel.add_current_to_palette();
            });
        }

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

    pub fn favorites_widget(&self) -> gtk::Revealer {
        self.slide_revealer.clone()
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

    pub fn refresh(&self) {
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

        if storage.palette().contains(&current) {
            self.palette_button
                .set_tooltip_text(Some("Already in palette"));
            self.palette_button.add_css_class("palette-active");
        } else {
            self.palette_button.set_tooltip_text(Some("Add to palette"));
            self.palette_button.remove_css_class("palette-active");
        }

        populate_color_list(
            &self.favorites_box,
            storage.favorites(),
            "No favorites yet",
            self.clone(),
            true,
        );
        populate_palette(&self.palette_grid, storage.palette(), self.clone());
    }

    fn toggle_panel(&self) {
        self.set_panel_visible(!self.slide_revealer.reveals_child());
    }

    fn set_panel_visible(&self, visible: bool) {
        self.fade_revealer.set_reveal_child(visible);
        self.slide_revealer.set_reveal_child(visible);

        if visible {
            self.panel_button.add_css_class("panel-active");
        } else {
            self.panel_button.remove_css_class("panel-active");
        }
    }

    fn select_color(&self, color: Rgb) {
        (self.on_select)(color);
        self.set_current(color);
        self.set_panel_visible(false);
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

fn populate_palette(grid: &gtk::Grid, colors: &[Rgb], panel: SavedColorsPanel) {
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }

    for (index, color) in colors.iter().enumerate() {
        let column = index % PALETTE_COLUMNS;
        let row = index / PALETTE_COLUMNS;
        grid.attach(
            &palette_color_button(*color, panel.clone()),
            column as i32,
            row as i32,
            1,
            1,
        );
    }
}

fn palette_color_button(color: Rgb, panel: SavedColorsPanel) -> Button {
    let button = Button::builder()
        .width_request(26)
        .height_request(26)
        .tooltip_text(color.hex())
        .build();

    button.add_css_class("palette-color-button");

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
