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
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::settings_manager;
use crate::ui::FurtheranceWindow;
use crate::FurtheranceApplication;

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/lakoliu/Furtherance/gtk/preferences_window.ui")]
    pub struct FurPreferencesWindow {
        #[template_child]
        pub appearance_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub dark_theme_switch: TemplateChild<gtk::Switch>,

        #[template_child]
        pub idle_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub notify_of_idle_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub notify_of_idle_spin: TemplateChild<gtk::SpinButton>,

        #[template_child]
        pub task_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub limit_tasks_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub limit_days_spin: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub delete_confirmation_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub show_seconds_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub show_daily_sums_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub show_tags_switch: TemplateChild<gtk::Switch>,

        #[template_child]
        pub timer_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub pomodoro_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub pomodoro_spin: TemplateChild<gtk::SpinButton>,

        #[template_child]
        pub autosave_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub autosave_spin: TemplateChild<gtk::SpinButton>,
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

        fn constructed(&self, obj: &Self::Type) {
            let window = FurtheranceWindow::default();
            obj.set_transient_for(Some(&window));

            obj.setup_signals();
            obj.setup_widgets();

            self.parent_constructed(obj);
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
        glib::Object::new::<FurPreferencesWindow>(&[]).unwrap()
    }

    fn setup_widgets(&self) {
        self.set_search_enabled(false);

        let imp = imp::FurPreferencesWindow::from_instance(self);

        let manager = adw::StyleManager::default();
        let support_darkmode = manager.system_supports_color_schemes();
        imp.appearance_group.set_visible(!support_darkmode);
    }

    fn setup_signals(&self) {
        let imp = imp::FurPreferencesWindow::from_instance(self);

        settings_manager::bind_property(
            "dark-mode",
            &*imp.dark_theme_switch,
            "active"
        );

        settings_manager::bind_property(
            "notify-of-idle",
            &*imp.notify_of_idle_expander,
            "enable-expansion",
        );

        settings_manager::bind_property(
            "idle-time",
            &*imp.notify_of_idle_spin,
            "value",
        );

        settings_manager::bind_property(
            "limit-tasks",
            &*imp.limit_tasks_expander,
            "enable-expansion",
        );

        settings_manager::bind_property(
            "limit-days",
            &*imp.limit_days_spin,
            "value",
        );

        settings_manager::bind_property(
            "delete-confirmation",
            &*imp.delete_confirmation_switch,
            "active"
        );

        settings_manager::bind_property(
            "show-seconds",
            &*imp.show_seconds_switch,
            "active"
        );

        settings_manager::bind_property(
            "show-daily-sums",
            &*imp.show_daily_sums_switch,
            "active"
        );

        settings_manager::bind_property(
            "show-tags",
            &*imp.show_tags_switch,
            "active"
        );

        settings_manager::bind_property(
            "pomodoro",
            &*imp.pomodoro_expander,
            "enable-expansion"
        );

        settings_manager::bind_property(
            "pomodoro-time",
            &*imp.pomodoro_spin,
            "value"
        );

        settings_manager::bind_property(
            "autosave",
            &*imp.autosave_expander,
            "enable-expansion"
        );

        settings_manager::bind_property(
            "autosave-time",
            &*imp.autosave_spin,
            "value"
        );

        imp.dark_theme_switch.connect_active_notify(move |_|{
            let app = FurtheranceApplication::default();
            app.update_light_dark();
        });

        imp.limit_tasks_expander.connect_enable_expansion_notify(move |_|{
            let window = FurtheranceWindow::default();
            window.reset_history_box();
        });

        imp.limit_days_spin.connect_value_changed(move |_|{
            let window = FurtheranceWindow::default();
            window.reset_history_box();
        });

        imp.show_seconds_switch.connect_active_notify(move |_|{
            let window = FurtheranceWindow::default();
            window.reset_history_box();
        });

        imp.show_daily_sums_switch.connect_active_notify(move |_|{
            let window = FurtheranceWindow::default();
            window.reset_history_box();
        });

        imp.show_tags_switch.connect_active_notify(move |_|{
            let window = FurtheranceWindow::default();
            window.reset_history_box();
        });

        imp.pomodoro_expander.connect_enable_expansion_notify(move |_|{
            let window = FurtheranceWindow::default();
            window.refresh_timer();
        });

        imp.pomodoro_spin.connect_value_changed(move |_|{
            let window = FurtheranceWindow::default();
            window.refresh_timer();
        });
    }
}


