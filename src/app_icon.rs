pub const APP_ICON_NAME: &str = "picker-icon";

pub fn register() {
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };

    let icon_theme = gtk::IconTheme::for_display(&display);

    if let Ok(current_dir) = std::env::current_dir() {
        icon_theme.add_search_path(current_dir);
    }

    icon_theme.add_search_path(env!("CARGO_MANIFEST_DIR"));
    gtk::Window::set_default_icon_name(APP_ICON_NAME);
}
