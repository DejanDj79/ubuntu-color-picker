use adw::prelude::*;
use ashpd::desktop::Color;
use gtk::glib;
use gtk::{Align, Button, Entry, Label, Orientation, Scale};
use std::cell::Cell;
use std::rc::Rc;

#[tokio::main]
async fn main() {
    let app = adw::Application::builder()
        .application_id("com.example.UbuntuColorPicker")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Color Picker")
        .default_width(520)
        .default_height(230)
        .resizable(false)
        .build();

    let root = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    root.add_css_class("app-bg");

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
    let book_btn = icon_button("starred-symbolic", "Favorites");
    let copy_btn = icon_button("edit-copy-symbolic", "Copy color");
    let settings_btn = icon_button("emblem-system-symbolic", "Settings");

    top_bar.append(&picker_btn);
    top_bar.append(&sliders_btn);
    top_bar.append(&book_btn);
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

    top_bar.append(&minimize_btn);
    top_bar.append(&maximize_btn);
    top_bar.append(&close_btn);

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

    load_css(&css_provider, 50, 216, 122);

    let red = Rc::new(Cell::new(50u8));
    let green = Rc::new(Cell::new(216u8));
    let blue = Rc::new(Cell::new(122u8));

    r_scale.set_value(50.0);
    g_scale.set_value(216.0);
    b_scale.set_value(122.0);

    connect_slider(
        &r_scale,
        red.clone(),
        green.clone(),
        blue.clone(),
        r_entry.clone(),
        g_entry.clone(),
        b_entry.clone(),
        hex_entry.clone(),
        css_provider.clone(),
        ColorChannel::Red,
    );

    connect_slider(
        &g_scale,
        red.clone(),
        green.clone(),
        blue.clone(),
        r_entry.clone(),
        g_entry.clone(),
        b_entry.clone(),
        hex_entry.clone(),
        css_provider.clone(),
        ColorChannel::Green,
    );

    connect_slider(
        &b_scale,
        red.clone(),
        green.clone(),
        blue.clone(),
        r_entry.clone(),
        g_entry.clone(),
        b_entry.clone(),
        hex_entry.clone(),
        css_provider.clone(),
        ColorChannel::Blue,
    );

    {
        let red = red.clone();
        let green = green.clone();
        let blue = blue.clone();

        let r_scale = r_scale.clone();
        let g_scale = g_scale.clone();
        let b_scale = b_scale.clone();

        let r_entry = r_entry.clone();
        let g_entry = g_entry.clone();
        let b_entry = b_entry.clone();
        let hex_entry = hex_entry.clone();

        let css_provider = css_provider.clone();

        picker_btn.connect_clicked(move |_| {
            let red = red.clone();
            let green = green.clone();
            let blue = blue.clone();

            let r_scale = r_scale.clone();
            let g_scale = g_scale.clone();
            let b_scale = b_scale.clone();

            let r_entry = r_entry.clone();
            let g_entry = g_entry.clone();
            let b_entry = b_entry.clone();
            let hex_entry = hex_entry.clone();

            let css_provider = css_provider.clone();

            glib::MainContext::default().spawn_local(async move {
                match Color::pick().send().await.and_then(|request| request.response()) {
                    Ok(color) => {
                        let r = (color.red() * 255.0).round() as u8;
                        let g = (color.green() * 255.0).round() as u8;
                        let b = (color.blue() * 255.0).round() as u8;

                        red.set(r);
                        green.set(g);
                        blue.set(b);

                        r_scale.set_value(r as f64);
                        g_scale.set_value(g as f64);
                        b_scale.set_value(b as f64);

                        update_fields(
                            r,
                            g,
                            b,
                            &r_entry,
                            &g_entry,
                            &b_entry,
                            &hex_entry,
                            &css_provider,
                        );
                    }
                    Err(error) => {
                        hex_entry.set_text(&format!("Error: {}", error));
                    }
                }
            });
        });
    }

    window.set_content(Some(&root));
    window.present();
}

#[derive(Clone, Copy)]
enum ColorChannel {
    Red,
    Green,
    Blue,
}

fn icon_button(icon_name: &str, tooltip: &str) -> Button {
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

fn window_button(label: &str) -> Button {
    let button = Button::builder()
        .label(label)
        .width_request(34)
        .height_request(30)
        .build();

    button.add_css_class("window-button");

    button
}

fn make_scale() -> Scale {
    let scale = Scale::with_range(Orientation::Horizontal, 0.0, 255.0, 1.0);
    scale.set_draw_value(false);
    scale.set_width_request(300);
    scale.add_css_class("rgb-slider");
    scale
}

fn make_value_entry(value: &str) -> Entry {
    Entry::builder()
        .text(value)
        .editable(false)
        .width_chars(4)
        .max_width_chars(4)
        .halign(Align::End)
        .build()
}

fn make_hex_entry(value: &str) -> Entry {
    Entry::builder()
        .text(value)
        .editable(false)
        .width_chars(9)
        .max_width_chars(9)
        .halign(Align::End)
        .build()
}

fn slider_row(label: &str, scale: &Scale, value_entry: &Entry, hex_entry: Option<&Entry>) -> gtk::Box {
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

    let value_box = gtk::Box::builder()
        .width_request(64)
        .build();
    value_box.append(value_entry);

    let hex_box = gtk::Box::builder()
        .width_request(100)
        .build();

    if let Some(hex) = hex_entry {
        hex_box.append(hex);
    }

    row.append(&channel_label);
    row.append(scale);
    row.append(&value_box);
    row.append(&hex_box);

    row
}

fn connect_slider(
    scale: &Scale,
    red: Rc<Cell<u8>>,
    green: Rc<Cell<u8>>,
    blue: Rc<Cell<u8>>,
    r_entry: Entry,
    g_entry: Entry,
    b_entry: Entry,
    hex_entry: Entry,
    css_provider: Rc<gtk::CssProvider>,
    channel: ColorChannel,
) {
    scale.connect_value_changed(move |scale| {
        let value = scale.value().round() as u8;

        match channel {
            ColorChannel::Red => red.set(value),
            ColorChannel::Green => green.set(value),
            ColorChannel::Blue => blue.set(value),
        }

        update_fields(
            red.get(),
            green.get(),
            blue.get(),
            &r_entry,
            &g_entry,
            &b_entry,
            &hex_entry,
            &css_provider,
        );
    });
}

fn update_fields(
    r: u8,
    g: u8,
    b: u8,
    r_entry: &Entry,
    g_entry: &Entry,
    b_entry: &Entry,
    hex_entry: &Entry,
    css_provider: &gtk::CssProvider,
) {
    r_entry.set_text(&r.to_string());
    g_entry.set_text(&g.to_string());
    b_entry.set_text(&b.to_string());

    let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
    hex_entry.set_text(&hex);

    load_css(css_provider, r, g, b);
}

fn load_css(provider: &gtk::CssProvider, r: u8, g: u8, b: u8) {
    let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);

    let brightness = (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64) / 255.0;

    let text_color = if brightness > 0.55 {
        "#111111"
    } else {
        "#F5F5F5"
    };

    let panel_color = if brightness > 0.55 {
        "rgba(255,255,255,0.42)"
    } else {
        "rgba(0,0,0,0.28)"
    };

    let border_color = if brightness > 0.55 {
        "rgba(0,0,0,0.18)"
    } else {
        "rgba(255,255,255,0.20)"
    };

    let css = format!(
        "
        window {{
            border-radius: 0;
        }}

        .app-bg {{
            background: {};
            color: {};
            border-radius: 0;
        }}

        .top-bar {{
            border-bottom: 1px solid {};
            padding-left: 10px;
            padding-right: 10px;
        }}

        .toolbar-button,
        .window-button {{
            border-radius: 0;
            background: transparent;
            color: {};
            border: none;
            padding: 0;
            box-shadow: none;
        }}

        .toolbar-button:hover,
        .window-button:hover {{
            background: rgba(255,255,255,0.16);
        }}

        .channel-label {{
            font-weight: 700;
            color: {};
        }}

        entry {{
            border-radius: 0;
            background: {};
            color: {};
            border: 1px solid {};
            font-family: monospace;
            font-weight: 700;
        }}

        scale trough {{
            min-height: 8px;
            border-radius: 999px;
            background: {};
            border: 1px solid {};
        }}

        scale highlight {{
            border-radius: 999px;
            background: {};
        }}

        scale slider {{
            min-width: 18px;
            min-height: 18px;
            border-radius: 999px;
            background: {};
            border: 2px solid {};
        }}
        ",
        hex,
        text_color,
        border_color,
        text_color,
        text_color,
        panel_color,
        text_color,
        border_color,
        panel_color,
        border_color,
        text_color,
        text_color,
        border_color,
    );
    provider.load_from_data(&css);
}