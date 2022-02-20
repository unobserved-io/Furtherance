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

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/lakoliu/Furtherance/gtk/preferences_window.ui")]
    pub struct FurPreferencesWindow {
        #[template_child]
        pub idle_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub notify_of_idle_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub notify_of_idle_spin: TemplateChild<gtk::SpinButton>,
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

    }

    fn setup_signals(&self) {
        let imp = imp::FurPreferencesWindow::from_instance(self);

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
    }

}

