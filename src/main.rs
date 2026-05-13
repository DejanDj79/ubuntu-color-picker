mod clipboard;
mod color;
mod saved_colors;
mod storage;
mod style;
mod widgets;

use adw::prelude::*;
use ashpd::desktop::Color;
use gtk::{glib, Entry, Orientation, Scale};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use clipboard::{copy_color, CopyFeedback};
use color::{parse_rgb_channel, CopyFormat, Rgb};
use saved_colors::SavedColorsPanel;
use storage::ColorCollections;
use style::load_css;
use widgets::{
    copy_format_popover, copy_toast, icon_button, make_hex_entry, make_scale, make_value_entry,
    menu_icon_button, slider_row, window_button,
};

const INITIAL_COLOR: Rgb = Rgb::new(50, 216, 122);

#[derive(Clone)]
struct ColorState {
    red: Rc<Cell<u8>>,
    green: Rc<Cell<u8>>,
    blue: Rc<Cell<u8>>,
    syncing: Rc<Cell<bool>>,
}

impl ColorState {
    fn new(color: Rgb) -> Self {
        Self {
            red: Rc::new(Cell::new(color.r)),
            green: Rc::new(Cell::new(color.g)),
            blue: Rc::new(Cell::new(color.b)),
            syncing: Rc::new(Cell::new(false)),
        }
    }

    fn get(&self) -> Rgb {
        Rgb::new(self.red.get(), self.green.get(), self.blue.get())
    }

    fn set(&self, color: Rgb) {
        self.red.set(color.r);
        self.green.set(color.g);
        self.blue.set(color.b);
    }

    fn is_syncing(&self) -> bool {
        self.syncing.get()
    }

    fn set_syncing(&self, syncing: bool) {
        self.syncing.set(syncing);
    }
}

#[tokio::main]
async fn main() {
    let app = adw::Application::builder()
        .application_id("com.example.UbuntuColorPicker")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let color_state = ColorState::new(INITIAL_COLOR);
    let current_color = Rc::new(Cell::new(INITIAL_COLOR));
    let collections = Rc::new(RefCell::new(ColorCollections::load()));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Color Picker")
        .default_width(520)
        .default_height(282)
        .resizable(false)
        .build();

    let root = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    root.add_css_class("app-bg");

    let overlay = gtk::Overlay::new();
    let (copy_toast, copy_toast_label) = copy_toast();
    overlay.add_overlay(&copy_toast);

    let top_bar = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .height_request(42)
        .margin_top(0)
        .margin_bottom(0)
        .margin_start(0)
        .margin_end(0)
        .build();

    top_bar.add_css_class("top-bar");

    let picker_btn = icon_button("color-select-symbolic", "Pick color");
    let sliders_btn = icon_button("view-list-symbolic", "RGB sliders");
    let favorite_btn = icon_button("non-starred-symbolic", "Add favorite");
    let palette_btn = icon_button("list-add-symbolic", "Add to palette");
    let saved_popover = gtk::Popover::builder()
        .autohide(true)
        .has_arrow(true)
        .build();
    let saved_btn = menu_icon_button("user-bookmarks-symbolic", "Favorites", &saved_popover);
    let copy_feedback = CopyFeedback::new(&copy_toast, &copy_toast_label);
    let copy_action: Rc<dyn Fn(CopyFormat)> = {
        let color_state = color_state.clone();
        let copy_feedback = copy_feedback.clone();
        let window = window.clone();

        Rc::new(move |format| {
            copy_color(&window, &copy_feedback, color_state.get(), format);
        })
    };
    let copy_popover = copy_format_popover(copy_action);
    let copy_btn = menu_icon_button("edit-copy-symbolic", "Copy color", &copy_popover);
    let settings_btn = icon_button("emblem-system-symbolic", "Settings");

    top_bar.append(&picker_btn);
    top_bar.append(&sliders_btn);
    top_bar.append(&favorite_btn);
    top_bar.append(&palette_btn);
    top_bar.append(&saved_btn);
    top_bar.append(&copy_btn);
    top_bar.append(&settings_btn);

    let spacer = gtk::Box::builder().hexpand(true).build();
    top_bar.append(&spacer);

    let minimize_btn = window_button("−");
    let maximize_btn = window_button("□");
    let close_btn = window_button("×");

    {
        let window = window.clone();
        minimize_btn.connect_clicked(move |_| {
            window.minimize();
        });
    }

    {
        let window = window.clone();
        maximize_btn.connect_clicked(move |_| {
            if window.is_maximized() {
                window.unmaximize();
            } else {
                window.maximize();
            }
        });
    }

    {
        let window = window.clone();
        close_btn.connect_clicked(move |_| {
            window.close();
        });
    }

    let window_controls = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .build();

    window_controls.append(&minimize_btn);
    window_controls.append(&maximize_btn);
    window_controls.append(&close_btn);
    top_bar.append(&window_controls);

    let content = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(14)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(18)
        .build();

    let r_scale = make_scale();
    let g_scale = make_scale();
    let b_scale = make_scale();

    let r_entry = make_value_entry("50");
    let g_entry = make_value_entry("216");
    let b_entry = make_value_entry("122");
    let hex_entry = make_hex_entry("#32D87A");

    connect_entry_copy_icon(
        &r_entry,
        color_state.clone(),
        copy_feedback.clone(),
        CopyFormat::Rgb,
    );
    connect_entry_copy_icon(
        &g_entry,
        color_state.clone(),
        copy_feedback.clone(),
        CopyFormat::Rgb,
    );
    connect_entry_copy_icon(
        &b_entry,
        color_state.clone(),
        copy_feedback.clone(),
        CopyFormat::Rgb,
    );
    connect_entry_copy_icon(
        &hex_entry,
        color_state.clone(),
        copy_feedback.clone(),
        CopyFormat::Hex,
    );

    let r_row = slider_row("R", &r_scale, &r_entry, None);
    let g_row = slider_row("G", &g_scale, &g_entry, None);
    let b_row = slider_row("B", &b_scale, &b_entry, Some(&hex_entry));

    content.append(&r_row);
    content.append(&g_row);
    content.append(&b_row);

    let window_handle = gtk::WindowHandle::new();
    window_handle.set_child(Some(&top_bar));

    root.append(&window_handle);
    root.append(&content);

    let css_provider = Rc::new(gtk::CssProvider::new());

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        css_provider.as_ref(),
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    load_css(&css_provider, color_state.get());

    let saved_panel = {
        let color_state = color_state.clone();

        let r_scale = r_scale.clone();
        let g_scale = g_scale.clone();
        let b_scale = b_scale.clone();

        let r_entry = r_entry.clone();
        let g_entry = g_entry.clone();
        let b_entry = b_entry.clone();
        let hex_entry = hex_entry.clone();

        let css_provider = css_provider.clone();

        SavedColorsPanel::new(
            saved_popover.clone(),
            collections.clone(),
            current_color.clone(),
            favorite_btn.clone(),
            palette_btn.clone(),
            copy_feedback.clone(),
            Rc::new(move |color| {
                apply_selected_color(
                    color,
                    &color_state,
                    &r_scale,
                    &g_scale,
                    &b_scale,
                    &r_entry,
                    &g_entry,
                    &b_entry,
                    &hex_entry,
                    &css_provider,
                );
            }),
        )
    };

    content.append(&saved_panel.palette_widget());

    saved_panel.set_current(color_state.get());

    r_scale.set_value(INITIAL_COLOR.r as f64);
    g_scale.set_value(INITIAL_COLOR.g as f64);
    b_scale.set_value(INITIAL_COLOR.b as f64);

    connect_slider(
        &r_scale,
        color_state.clone(),
        r_entry.clone(),
        g_entry.clone(),
        b_entry.clone(),
        hex_entry.clone(),
        css_provider.clone(),
        saved_panel.clone(),
        ColorChannel::Red,
    );

    connect_slider(
        &g_scale,
        color_state.clone(),
        r_entry.clone(),
        g_entry.clone(),
        b_entry.clone(),
        hex_entry.clone(),
        css_provider.clone(),
        saved_panel.clone(),
        ColorChannel::Green,
    );

    connect_slider(
        &b_scale,
        color_state.clone(),
        r_entry.clone(),
        g_entry.clone(),
        b_entry.clone(),
        hex_entry.clone(),
        css_provider.clone(),
        saved_panel.clone(),
        ColorChannel::Blue,
    );

    connect_rgb_entry(
        &r_entry,
        color_state.clone(),
        r_scale.clone(),
        g_scale.clone(),
        b_scale.clone(),
        r_entry.clone(),
        g_entry.clone(),
        b_entry.clone(),
        hex_entry.clone(),
        css_provider.clone(),
        saved_panel.clone(),
        ColorChannel::Red,
    );

    connect_rgb_entry(
        &g_entry,
        color_state.clone(),
        r_scale.clone(),
        g_scale.clone(),
        b_scale.clone(),
        r_entry.clone(),
        g_entry.clone(),
        b_entry.clone(),
        hex_entry.clone(),
        css_provider.clone(),
        saved_panel.clone(),
        ColorChannel::Green,
    );

    connect_rgb_entry(
        &b_entry,
        color_state.clone(),
        r_scale.clone(),
        g_scale.clone(),
        b_scale.clone(),
        r_entry.clone(),
        g_entry.clone(),
        b_entry.clone(),
        hex_entry.clone(),
        css_provider.clone(),
        saved_panel.clone(),
        ColorChannel::Blue,
    );

    connect_hex_entry(
        &hex_entry,
        color_state.clone(),
        r_scale.clone(),
        g_scale.clone(),
        b_scale.clone(),
        r_entry.clone(),
        g_entry.clone(),
        b_entry.clone(),
        hex_entry.clone(),
        css_provider.clone(),
        saved_panel.clone(),
    );

    {
        let color_state = color_state.clone();

        let r_scale = r_scale.clone();
        let g_scale = g_scale.clone();
        let b_scale = b_scale.clone();

        let r_entry = r_entry.clone();
        let g_entry = g_entry.clone();
        let b_entry = b_entry.clone();
        let hex_entry = hex_entry.clone();

        let css_provider = css_provider.clone();
        let saved_panel = saved_panel.clone();

        picker_btn.connect_clicked(move |_| {
            let color_state = color_state.clone();

            let r_scale = r_scale.clone();
            let g_scale = g_scale.clone();
            let b_scale = b_scale.clone();

            let r_entry = r_entry.clone();
            let g_entry = g_entry.clone();
            let b_entry = b_entry.clone();
            let hex_entry = hex_entry.clone();

            let css_provider = css_provider.clone();
            let saved_panel = saved_panel.clone();

            glib::MainContext::default().spawn_local(async move {
                match Color::pick()
                    .send()
                    .await
                    .and_then(|request| request.response())
                {
                    Ok(color) => {
                        let color = Rgb::from_normalized(color.red(), color.green(), color.blue());

                        apply_selected_color(
                            color,
                            &color_state,
                            &r_scale,
                            &g_scale,
                            &b_scale,
                            &r_entry,
                            &g_entry,
                            &b_entry,
                            &hex_entry,
                            &css_provider,
                        );
                        saved_panel.set_current(color);
                    }
                    Err(error) => {
                        hex_entry.set_text(&format!("Error: {}", error));
                    }
                }
            });
        });
    }

    overlay.set_child(Some(&root));
    window.set_content(Some(&overlay));
    window.present();
}

#[derive(Clone, Copy)]
enum ColorChannel {
    Red,
    Green,
    Blue,
}

fn connect_entry_copy_icon(
    entry: &Entry,
    color_state: ColorState,
    copy_feedback: CopyFeedback,
    format: CopyFormat,
) {
    entry.connect_icon_press(move |entry, position| {
        if position == gtk::EntryIconPosition::Secondary {
            copy_color(entry, &copy_feedback, color_state.get(), format);
        }
    });
}

fn connect_rgb_entry(
    entry: &Entry,
    color_state: ColorState,
    r_scale: Scale,
    g_scale: Scale,
    b_scale: Scale,
    r_entry: Entry,
    g_entry: Entry,
    b_entry: Entry,
    hex_entry: Entry,
    css_provider: Rc<gtk::CssProvider>,
    saved_panel: SavedColorsPanel,
    channel: ColorChannel,
) {
    entry.connect_changed(move |entry| {
        if color_state.is_syncing() {
            return;
        }

        let text = entry.text();
        let Some(value) = parse_rgb_channel(text.as_str()) else {
            set_entry_error(entry, true);
            return;
        };

        set_entry_error(entry, false);
        let cursor_position = entry.position();
        let mut color = color_state.get();

        match channel {
            ColorChannel::Red => color.r = value,
            ColorChannel::Green => color.g = value,
            ColorChannel::Blue => color.b = value,
        }

        apply_selected_color(
            color,
            &color_state,
            &r_scale,
            &g_scale,
            &b_scale,
            &r_entry,
            &g_entry,
            &b_entry,
            &hex_entry,
            &css_provider,
        );
        saved_panel.set_current(color);

        restore_entry_position(entry, cursor_position);
    });
}

fn connect_hex_entry(
    entry: &Entry,
    color_state: ColorState,
    r_scale: Scale,
    g_scale: Scale,
    b_scale: Scale,
    r_entry: Entry,
    g_entry: Entry,
    b_entry: Entry,
    hex_entry: Entry,
    css_provider: Rc<gtk::CssProvider>,
    saved_panel: SavedColorsPanel,
) {
    entry.connect_changed(move |entry| {
        if color_state.is_syncing() {
            return;
        }

        let text = entry.text();
        let Some(color) = Rgb::from_hex_input(text.as_str()) else {
            set_entry_error(entry, true);
            return;
        };

        set_entry_error(entry, false);
        let cursor_offset = if text.trim_start().starts_with('#') {
            0
        } else {
            1
        };
        let cursor_position = entry.position() + cursor_offset;

        apply_selected_color(
            color,
            &color_state,
            &r_scale,
            &g_scale,
            &b_scale,
            &r_entry,
            &g_entry,
            &b_entry,
            &hex_entry,
            &css_provider,
        );
        saved_panel.set_current(color);

        restore_entry_position(entry, cursor_position);
    });
}

fn set_entry_error(entry: &Entry, has_error: bool) {
    if has_error {
        entry.add_css_class("input-error");
    } else {
        entry.remove_css_class("input-error");
    }
}

fn restore_entry_position(entry: &Entry, position: i32) {
    let max_position = entry.text().chars().count() as i32;
    entry.set_position(position.clamp(0, max_position));
}

fn connect_slider(
    scale: &Scale,
    color_state: ColorState,
    r_entry: Entry,
    g_entry: Entry,
    b_entry: Entry,
    hex_entry: Entry,
    css_provider: Rc<gtk::CssProvider>,
    saved_panel: SavedColorsPanel,
    channel: ColorChannel,
) {
    scale.connect_value_changed(move |scale| {
        if color_state.is_syncing() {
            return;
        }

        let value = scale.value().round() as u8;
        let mut color = color_state.get();

        match channel {
            ColorChannel::Red => color.r = value,
            ColorChannel::Green => color.g = value,
            ColorChannel::Blue => color.b = value,
        }

        color_state.set(color);
        update_fields_guarded(
            color,
            &color_state,
            &r_entry,
            &g_entry,
            &b_entry,
            &hex_entry,
            &css_provider,
        );
        saved_panel.set_current(color);
    });
}

fn apply_selected_color(
    color: Rgb,
    color_state: &ColorState,
    r_scale: &Scale,
    g_scale: &Scale,
    b_scale: &Scale,
    r_entry: &Entry,
    g_entry: &Entry,
    b_entry: &Entry,
    hex_entry: &Entry,
    css_provider: &gtk::CssProvider,
) {
    color_state.set(color);

    color_state.set_syncing(true);
    r_scale.set_value(color.r as f64);
    g_scale.set_value(color.g as f64);
    b_scale.set_value(color.b as f64);

    update_fields(color, r_entry, g_entry, b_entry, hex_entry, css_provider);
    color_state.set_syncing(false);
}

fn update_fields_guarded(
    color: Rgb,
    color_state: &ColorState,
    r_entry: &Entry,
    g_entry: &Entry,
    b_entry: &Entry,
    hex_entry: &Entry,
    css_provider: &gtk::CssProvider,
) {
    color_state.set_syncing(true);
    update_fields(color, r_entry, g_entry, b_entry, hex_entry, css_provider);
    color_state.set_syncing(false);
}

fn update_fields(
    color: Rgb,
    r_entry: &Entry,
    g_entry: &Entry,
    b_entry: &Entry,
    hex_entry: &Entry,
    css_provider: &gtk::CssProvider,
) {
    set_entry_error(r_entry, false);
    set_entry_error(g_entry, false);
    set_entry_error(b_entry, false);
    set_entry_error(hex_entry, false);

    r_entry.set_text(&color.r.to_string());
    g_entry.set_text(&color.g.to_string());
    b_entry.set_text(&color.b.to_string());

    hex_entry.set_text(&color.hex());

    load_css(css_provider, color);
}
