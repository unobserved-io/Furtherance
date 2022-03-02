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

use gtk::subclass::prelude::*;
use gtk::{glib, gio, prelude::*, CompositeTemplate};
use chrono::DateTime;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use glib::clone;

use crate::database::Task;
use crate::ui::FurTaskDetails;


mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/lakoliu/Furtherance/gtk/task_row.ui")]
    pub struct FurTaskRow {
        #[template_child]
        pub task_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub total_time_label: TemplateChild<gtk::Label>,
        // pub tasks: Vec<database::Task>,
        pub tasks: Lazy<Mutex<Vec<Task>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FurTaskRow {
        const NAME: &'static str = "FurTaskRow";
        type ParentType = gtk::ListBoxRow;
        type Type = super::FurTaskRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FurTaskRow {
        fn constructed(&self, obj: &Self::Type) {
            obj.setup_signals();
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for FurTaskRow {}

    impl ListBoxRowImpl for FurTaskRow {}
}

glib::wrapper! {
    pub struct FurTaskRow(
        ObjectSubclass<imp::FurTaskRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl FurTaskRow {
    pub fn new() -> Self {
        /* This should take a model, object, or vec filled with the description
        details and fill out the labels based on those. */
        glib::Object::new(&[]).expect("Failed to create `FurTaskRow`.")
    }

    fn setup_signals(&self) {
        let open_details_action = gio::SimpleAction::new("open-details", None);

        open_details_action.connect_activate(clone!(@strong self as this => move |_, _| {
            let dialog = FurTaskDetails::new();
            dialog.setup_widgets(this.get_tasks());
            dialog.show();
        }));

        let actions = gio::SimpleActionGroup::new();
        self.insert_action_group("task-row", Some(&actions));
        actions.add_action(&open_details_action);
    }

    pub fn set_row_labels(&self, task_list: Vec<Task>) {
        let imp = imp::FurTaskRow::from_instance(&self);
        for task in task_list.clone() {
            imp.tasks.lock().unwrap().push(task);
        }
        imp.task_name_label.set_text(&imp.tasks.lock().unwrap()[0].task_name);

        // Add up all durations for task of said name to create total_time
        let mut total_time: i64 = 0;
        for task in &task_list {
            if task.task_name == task.task_name {
                let start_time = DateTime::parse_from_rfc3339(&task.start_time).unwrap();
                let stop_time = DateTime::parse_from_rfc3339(&task.stop_time).unwrap();

                let duration = stop_time - start_time;
                total_time += duration.num_seconds();
            }
        }
        // Format total time to readable string
        let h = total_time / 60 / 60;
        let m = (total_time / 60) - (h * 60);
        let s = total_time - (m * 60);

        let total_time_str = format!("{:02}:{:02}:{:02}", h, m, s);

        imp.total_time_label.set_text(&total_time_str);
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        let imp = imp::FurTaskRow::from_instance(&self);
        imp.tasks.lock().unwrap().to_vec()
    }
}

