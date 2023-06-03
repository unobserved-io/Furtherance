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

use glib::subclass;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::database;
use crate::ui::{FurTasksPage, FurtheranceWindow};
use crate::FurtheranceApplication;

enum View {
    Loading,
    Empty,
    Tasks,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/lakoliu/Furtherance/gtk/history_box.ui")]
    pub struct FurHistoryBox {
        // Template widgets
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub tasks_page: TemplateChild<FurTasksPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FurHistoryBox {
        const NAME: &'static str = "FurHistoryBox";
        type Type = super::FurHistoryBox;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FurHistoryBox {
        fn constructed(&self) {
            let obj = self.obj();
            obj.setup_widgets();
            self.parent_constructed();
        }
    }
    impl WidgetImpl for FurHistoryBox {}
    impl BoxImpl for FurHistoryBox {}
}

glib::wrapper! {
    pub struct FurHistoryBox(
        ObjectSubclass<imp::FurHistoryBox>)
        @extends gtk::Widget, gtk::Box;
}

impl FurHistoryBox {
    fn setup_widgets(&self) {
        self.set_view(View::Loading);
        let is_saved_task: bool = match database::check_for_tasks() {
            Ok(_) => true,
            Err(_) => false,
        };
        if is_saved_task {
            self.set_view(View::Tasks);
        } else {
            self.set_view(View::Empty);
        }
    }

    fn set_view(&self, view: View) {
        let imp = imp::FurHistoryBox::from_obj(self);
        let app = FurtheranceApplication::default();
        app.delete_enabled(false);
        app.export_csv_enabled(false);
        app.backup_database_enabled(false);
        imp.spinner.set_spinning(false);

        let name = match view {
            View::Loading => {
                imp.spinner.set_spinning(true);
                "loading"
            }
            View::Empty => "empty",
            View::Tasks => {
                app.delete_enabled(true);
                app.export_csv_enabled(true);
                app.backup_database_enabled(true);
                "tasks"
            }
        };

        imp.stack.set_visible_child_name(name);
    }

    pub fn create_tasks_page(&self) {
        let imp = imp::FurHistoryBox::from_obj(self);
        let window = FurtheranceWindow::default();
        imp.tasks_page.clear_task_list();
        let is_saved_task: bool = match database::check_for_tasks() {
            Ok(_) => true,
            Err(_) => false,
        };
        if is_saved_task {
            window.vertical_align(gtk::Align::Start);
            self.set_view(View::Loading);
            imp.tasks_page.build_task_list();
            self.set_view(View::Tasks);
        } else {
            self.set_view(View::Empty);
            window.vertical_align(gtk::Align::Center);
        }
    }

    pub fn set_todays_time(&self, added_time: i32) {
        let imp = imp::FurHistoryBox::from_obj(self);
        imp.tasks_page.add_to_todays_time(added_time);
    }

    pub fn set_todays_stored_secs(&self, new_time: i32) {
        let imp = imp::FurHistoryBox::from_obj(self);
        imp.tasks_page.set_todays_stored_secs(new_time)
    }

    pub fn empty_view(&self) {
        self.set_view(View::Empty);
        let window = FurtheranceWindow::default();
        window.vertical_align(gtk::Align::Center);
    }
}
