use gtk::prelude::*;

use crate::app_icon;

pub fn show(parent: &adw::ApplicationWindow, app: &adw::Application) {
    let about = gtk::AboutDialog::builder()
        .application(app)
        .transient_for(parent)
        .modal(true)
        .program_name("D-Pick")
        .logo_icon_name(app_icon::APP_ICON_NAME)
        .version(env!("CARGO_PKG_VERSION"))
        .comments("Pick, copy, save, and explore colors.")
        .authors(["Dejan"])
        .build();

    about.present();
}
