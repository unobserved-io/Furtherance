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

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::*;
use glib::clone;
use gtk::glib;
use gtk::CompositeTemplate;

use crate::settings_manager;
use crate::ui::FurtheranceWindow;
use crate::FurtheranceApplication;
use crate::database;

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/lakoliu/Furtherance/gtk/preferences_window.ui")]
    pub struct FurPreferencesWindow {
        // General Page
        // Appearance Group
        #[template_child]
        pub appearance_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub dark_theme_switch: TemplateChild<adw::SwitchRow>,

        // Idle Group
        #[template_child]
        pub notify_of_idle_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub notify_of_idle_spin: TemplateChild<adw::SpinRow>,

        // Timer Group
        #[template_child]
        pub pomodoro_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub pomodoro_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub autosave_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub autosave_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub inclusive_total_switch: TemplateChild<adw::SwitchRow>,

        // Tasks Page
        // Task List Group
        #[template_child]
        pub delete_confirmation_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub limit_tasks_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub limit_days_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub show_daily_sums_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub show_seconds_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub show_tags_switch: TemplateChild<adw::SwitchRow>,

        // Task Input Group
        #[template_child]
        pub autocomplete_switch: TemplateChild<adw::SwitchRow>,

        // Data Page
        // Reports Group
        #[template_child]
        pub week_start_combo: TemplateChild<adw::ComboRow>,

        // Database Group
        #[template_child]
        pub database_loc_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub database_browse_btn: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FurPreferencesWindow {
        const NAME: &'static str = "FurPreferencesWindow";
        type ParentType = adw::PreferencesWindow;
        type Type = super::FurPreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FurPreferencesWindow {
        fn constructed(&self) {
            let window = FurtheranceWindow::default();
            let obj = self.obj();
            obj.set_transient_for(Some(&window));

            obj.setup_signals();
            obj.setup_widgets();

            self.parent_constructed();
        }
    }

    impl WidgetImpl for FurPreferencesWindow {}

    impl WindowImpl for FurPreferencesWindow {}

    impl AdwWindowImpl for FurPreferencesWindow {}

    impl PreferencesWindowImpl for FurPreferencesWindow {}
}

glib::wrapper! {
    pub struct FurPreferencesWindow(
        ObjectSubclass<imp::FurPreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow;

}

impl FurPreferencesWindow {
    pub fn new() -> Self {
        glib::Object::new::<FurPreferencesWindow>()
    }

    fn setup_widgets(&self) {
        self.set_search_enabled(false);

        let imp = imp::FurPreferencesWindow::from_obj(self);

        let manager = adw::StyleManager::default();
        let support_darkmode = manager.system_supports_color_schemes();
        imp.appearance_group.set_visible(!support_darkmode);

        let db_dir = database::get_directory().to_string_lossy().to_string();
        imp.database_loc_row.set_subtitle(&db_dir);
    }

    fn setup_signals(&self) {
        let imp = imp::FurPreferencesWindow::from_obj(self);

        settings_manager::bind_property("dark-mode", &*imp.dark_theme_switch, "active");

        settings_manager::bind_property(
            "notify-of-idle",
            &*imp.notify_of_idle_expander,
            "enable-expansion",
        );

        settings_manager::bind_property("idle-time", &*imp.notify_of_idle_spin, "value");

        settings_manager::bind_property(
            "limit-tasks",
            &*imp.limit_tasks_expander,
            "enable-expansion",
        );

        settings_manager::bind_property("limit-days", &*imp.limit_days_spin, "value");

        settings_manager::bind_property(
            "delete-confirmation",
            &*imp.delete_confirmation_switch,
            "active",
        );

        settings_manager::bind_property("show-seconds", &*imp.show_seconds_switch, "active");

        settings_manager::bind_property("show-daily-sums", &*imp.show_daily_sums_switch, "active");

        settings_manager::bind_property("show-tags", &*imp.show_tags_switch, "active");

        settings_manager::bind_property("autocomplete", &*imp.autocomplete_switch, "active");

        settings_manager::bind_property("pomodoro", &*imp.pomodoro_expander, "enable-expansion");

        settings_manager::bind_property("pomodoro-time", &*imp.pomodoro_spin, "value");

        settings_manager::bind_property("autosave", &*imp.autosave_expander, "enable-expansion");

        settings_manager::bind_property("autosave-time", &*imp.autosave_spin, "value");

        settings_manager::bind_property("inclusive-total", &*imp.inclusive_total_switch, "active");

        settings_manager::bind_property("week-starts", &*imp.week_start_combo, "selected");

        imp.dark_theme_switch.connect_active_notify(move |_| {
            let app = FurtheranceApplication::default();
            app.update_light_dark();
        });

        imp.limit_tasks_expander
            .connect_enable_expansion_notify(move |_| {
                let window = FurtheranceWindow::default();
                window.reset_history_box();
            });

        imp.limit_days_spin.connect_value_notify(move |_| {
            let window = FurtheranceWindow::default();
            window.reset_history_box();
        });

        imp.show_seconds_switch.connect_active_notify(move |_| {
            let window = FurtheranceWindow::default();
            window.reset_history_box();
        });

        imp.show_daily_sums_switch.connect_active_notify(move |_| {
            let window = FurtheranceWindow::default();
            window.reset_history_box();
        });

        imp.show_tags_switch.connect_active_notify(move |_| {
            let window = FurtheranceWindow::default();
            window.reset_history_box();
        });

        imp.autocomplete_switch.connect_active_notify(move |_| {
            let window = FurtheranceWindow::default();
            window.reset_autocomplete();
        });

        imp.pomodoro_expander
            .connect_enable_expansion_notify(move |_| {
                let window = FurtheranceWindow::default();
                window.refresh_timer();
        });

        imp.pomodoro_spin.connect_value_notify(move |new_val| {
            settings_manager::set_int("pomodoro-time", new_val.value() as i32);
            let window = FurtheranceWindow::default();
            window.refresh_timer();
        });

        imp.inclusive_total_switch.connect_active_notify(move |_| {
            let window = FurtheranceWindow::default();
            window.reset_history_box();
        });

        imp.database_browse_btn.connect_clicked(clone!(@weak self as this => move |_| {
            let window = FurtheranceApplication::default().active_window().unwrap();
            let dialog = gtk::FileChooserDialog::new(
                Some(&gettext("Backup Database")),
                Some(&window),
                gtk::FileChooserAction::Save,
                &[
                    (&gettext("Cancel"), gtk::ResponseType::Reject),
                    (&gettext("Save"), gtk::ResponseType::Accept),
                ]
            );
            dialog.set_modal(true);

            // Set a filter to show only SQLite files
            let filter = gtk::FileFilter::new();
            gtk::FileFilter::set_name(&filter, Some("*.db"));
            filter.add_mime_type("application/x-sqlite3");
            dialog.add_filter(&filter);
            dialog.set_current_name("furtherance.db");

            dialog.connect_response(
                clone!(@strong dialog, @weak this as this2 => move |filechooser, resp| {
                    if resp == gtk::ResponseType::Accept {
                        if let Some(path) = filechooser.file().and_then(|file| file.path()) {
                            let path = &path.to_string_lossy();
                            let _bkup = database::backup_db(path.to_string());

                            let settings = settings_manager::get_settings();
                            let _ = settings.set_string("database-loc", &path.to_string());

                            let imp2 = imp::FurPreferencesWindow::from_obj(&this2);
                            imp2.database_loc_row.set_subtitle(&path.to_string());

                            let window = FurtheranceWindow::default();
                            window.reset_history_box();
                        }
                        dialog.close();
                    } else {
                        dialog.close();
                    }
                }),
            );

            dialog.show();

        }));
    }
}
