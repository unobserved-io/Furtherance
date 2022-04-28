// Furtherance - Track your time without being tracked
// Copyright (C) 2022  Ricky Kresslein <rk@lakoliu.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use gettextrs::*;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use log::debug;
use std::sync::Mutex;

use crate::config;
use crate::ui::{FurtheranceWindow, FurPreferencesWindow};
use crate::database;
use crate::settings_manager;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct FurtheranceApplication {
        pub pomodoro_dialog: Mutex<gtk::MessageDialog>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FurtheranceApplication {
        const NAME: &'static str = "FurtheranceApplication";
        type Type = super::FurtheranceApplication;
        type ParentType = gtk::Application;
    }

    impl ObjectImpl for FurtheranceApplication {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_gactions();
            obj.setup_application();
            obj.set_accels_for_action("app.quit", &["<primary>Q", "<primary>W"]);
        }
    }

    impl ApplicationImpl for FurtheranceApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self, application: &Self::Type) {
            // Initialize the database
            let _ = database::db_init();
            let _ = database::upgrade_old_db();

            // Get the current window or create one if necessary
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = FurtheranceWindow::new(application);
                window.set_default_size(400, 600);
                window.set_title(Some("Furtherance"));
                window.upcast()
            };

            // Load style.css
            let css_file = gtk::CssProvider::new();
            gtk::CssProvider::load_from_resource(&css_file, "/com/lakoliu/Furtherance/gtk/style.css");
            gtk::StyleContext::add_provider_for_display(&gdk::Display::default().unwrap(), &css_file, 500);

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for FurtheranceApplication {}
}

glib::wrapper! {
    pub struct FurtheranceApplication(ObjectSubclass<imp::FurtheranceApplication>)
        @extends gio::Application, gtk::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl FurtheranceApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::new(&[("application-id", &application_id), ("flags", flags)])
            .expect("Failed to create FurtheranceApplication")
    }

    fn setup_gactions(&self) {
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.quit();
        }));
        self.add_action(&quit_action);

        let preferences_action = gio::SimpleAction::new("preferences", None);
        preferences_action.connect_activate(clone!(@weak self as app => move |_, _| {
            FurPreferencesWindow::new().show();
        }));
        self.set_accels_for_action("app.preferences", &["<primary>comma"]);
        self.add_action(&preferences_action);

        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.show_about();
        }));
        self.add_action(&about_action);

        let discard_idle_action = gio::SimpleAction::new("discard-idle-action", None);
        discard_idle_action.connect_activate(clone!(@weak self as app => move |_, _| {
            let window = FurtheranceWindow::default();
            let imp = window.imp();
            if *imp.running.lock().unwrap() && *imp.idle_time_reached.lock().unwrap() {
                window.imp().idle_dialog.lock().unwrap().response(gtk::ResponseType::Reject);
            }
        }));
        self.add_action(&discard_idle_action);

        let continue_idle_action = gio::SimpleAction::new("continue-idle-action", None);
        continue_idle_action.connect_activate(clone!(@weak self as app => move |_, _| {
            let window = FurtheranceWindow::default();
            if *window.imp().running.lock().unwrap() {
                window.imp().idle_dialog.lock().unwrap().response(gtk::ResponseType::Accept);
            }
        }));
        self.add_action(&continue_idle_action);

        let continue_pomodoro_action = gio::SimpleAction::new("continue-pomodoro-action", None);
        continue_pomodoro_action.connect_activate(clone!(@weak self as app => move |_, _| {
            let imp = imp::FurtheranceApplication::from_instance(&app);
            imp.pomodoro_dialog.lock().unwrap().response(gtk::ResponseType::Accept);
        }));
        self.add_action(&continue_pomodoro_action);

        let stop_pomodoro_action = gio::SimpleAction::new("stop-pomodoro-action", None);
        stop_pomodoro_action.connect_activate(clone!(@weak self as app => move |_, _| {
            let imp = imp::FurtheranceApplication::from_instance(&app);
            imp.pomodoro_dialog.lock().unwrap().response(gtk::ResponseType::Reject);
        }));
        self.add_action(&stop_pomodoro_action);
    }

    fn setup_application(&self) {
        self.update_light_dark();
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let dialog = gtk::AboutDialog::builder()
            .transient_for(&window)
            .modal(true)
            .program_name("Furtherance")
            .logo_icon_name(config::APP_ID)
            .version(config::VERSION)
            .comments(&gettext("Track your time without being tracked."))
            .copyright("© 2022 Ricky Kresslein")
            .authors(vec!["Ricky Kresslein <rk@lakoliu.com>".into()])
            .translator_credits(&gettext("translator-credits"))
            .website("https://furtherance.app")
            .license_type(gtk::License::Gpl30)
            .build();

        dialog.present();
    }

    fn delete_history(&self) {
        // Show dialog to delete all history
        let window = FurtheranceWindow::default();
        let dialog = gtk::MessageDialog::with_markup(
            Some(&window),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Question,
            gtk::ButtonsType::None,
            Some(&format!("<span size='x-large' weight='bold'>{}</span>", &gettext("Delete history?"))),
        );
        dialog.add_buttons(&[
            (&gettext("Cancel"), gtk::ResponseType::Reject),
            (&gettext("Delete"), gtk::ResponseType::Accept)
        ]);
        dialog.set_default_response(gtk::ResponseType::Accept);
        let delete_btn = dialog.widget_for_response(gtk::ResponseType::Accept).unwrap();
        delete_btn.add_css_class("destructive-action");

        let message_area = dialog.message_area().downcast::<gtk::Box>().unwrap();
        let explanation = gtk::Label::new(Some(&gettext("This will delete ALL of your task history.")));
        let instructions = gtk::Label::new(Some(
            &gettext("Type DELETE in the box below then click Delete to proceed.")));
        let delete_entry = gtk::Entry::new();
        delete_entry.set_activates_default(true);
        message_area.append(&explanation);
        message_area.append(&instructions);
        message_area.append(&delete_entry);

        dialog.connect_response(clone!(@weak dialog = > move |_, resp| {
            if resp == gtk::ResponseType::Accept {
                if delete_entry.text().to_uppercase() == gettext("DELETE") {
                    let _ = database::delete_all();
                    window.reset_history_box();
                    dialog.close();
                }
            } else {
                dialog.close();
            }
        }));

        dialog.show();
    }

    pub fn delete_enabled(&self, enabled: bool) {
        if enabled {
            let delete_history_action = gio::SimpleAction::new("delete-history", None);
            delete_history_action.connect_activate(clone!(@weak self as app => move |_, _| {
                app.delete_history();
            }));
            self.add_action(&delete_history_action);
        } else {
            self.remove_action("delete-history");
        }
    }

    pub fn update_light_dark(&self) {
        let manager = adw::StyleManager::default();

        if !manager.system_supports_color_schemes() {
            let color_scheme = if settings_manager::get_bool("dark-mode") {
                adw::ColorScheme::PreferDark
            } else {
                adw::ColorScheme::PreferLight
            };
            manager.set_color_scheme(color_scheme);
        }
    }

    pub fn system_idle_notification(&self, title: &str, subtitle: &str) {
        let icon = Some("appointment-missed-symbolic");
        let notification = gio::Notification::new(title.as_ref());
        notification.set_body(Some(subtitle.as_ref()));

        if let Some(icon) = icon {
            match gio::Icon::for_string(icon) {
                Ok(gicon) => notification.set_icon(&gicon),
                Err(err) => debug!("Unable to display notification: {:?}", err),
            }
        }

        notification.add_button(&gettext("Discard"), "app.discard-idle-action");
        notification.add_button(&gettext("Continue"), "app.continue-idle-action");

        notification.set_priority(gio::NotificationPriority::High);

        self.send_notification(Some("idle"), &notification);
    }

    pub fn system_pomodoro_notification(&self, dialog: gtk::MessageDialog) {
        let imp = imp::FurtheranceApplication::from_instance(self);
        *imp.pomodoro_dialog.lock().unwrap() = dialog;
        let icon = Some("alarm-symbolic");
        let notification = gio::Notification::new("Time's up!");
        notification.set_body(Some("Your Furtherance timer ended."));

        if let Some(icon) = icon {
            match gio::Icon::for_string(icon) {
                Ok(gicon) => notification.set_icon(&gicon),
                Err(err) => debug!("Unable to display notification: {:?}", err),
            }
        }

        notification.add_button(&gettext("Continue"), "app.continue-pomodoro-action");
        notification.add_button(&gettext("Stop"), "app.stop-pomodoro-action");

        notification.set_priority(gio::NotificationPriority::High);

        self.withdraw_notification("idle");
        self.send_notification(Some("pomodoro"), &notification);
    }
}

impl Default for FurtheranceApplication {
    fn default() -> Self {
        gio::Application::default()
            .expect("Could not get default GApplication")
            .downcast()
            .unwrap()
    }
}

