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

use chrono::DateTime;
use glib::clone;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, CompositeTemplate};
use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::database::Task;
use crate::settings_manager;
use crate::ui::{FurTaskDetails, FurtheranceWindow};

mod imp {
    use super::*;
    use glib::subclass;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/lakoliu/Furtherance/gtk/task_row.ui")]
    pub struct FurTaskRow {
        #[template_child]
        pub row_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub task_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub task_tags_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub total_time_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub restart_task_btn: TemplateChild<gtk::Button>,

        pub tasks: Lazy<Mutex<Vec<Task>>>,
        pub total_time: RefCell<i64>,
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
        fn constructed(&self) {
            let obj = self.obj();
            obj.setup_signals();
            self.parent_constructed();
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
        glib::Object::new::<FurTaskRow>()
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
        let imp = imp::FurTaskRow::from_obj(&self);
        for task in task_list.clone() {
            imp.tasks.lock().unwrap().push(task);
        }

        // Display task's name
        imp.task_name_label
            .set_text(&imp.tasks.lock().unwrap()[0].task_name);

        // Display task's tags
        if task_list[0].tags.trim().is_empty() || !settings_manager::get_bool("show-tags") {
            imp.task_tags_label.hide();
        } else {
            let task_tags = format!("#{}", task_list[0].tags);
            imp.task_tags_label.set_text(&task_tags);
        }

        // Create right-click gesture
        let gesture = gtk::GestureClick::new();
        gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_SECONDARY as u32);
        gesture.connect_pressed(clone!(@strong task_list => move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            let window = FurtheranceWindow::default();
            window.duplicate_task(task_list[0].clone());
        }));

        self.add_controller(gesture);

        imp.restart_task_btn
            .connect_clicked(clone!(@strong task_list => move |_| {
                let window = FurtheranceWindow::default();
                window.duplicate_task(task_list[0].clone());
            }));

        // Add up all durations for task of said name to create total_time
        for task in &task_list {
            if task.task_name == task.task_name {
                let start_time = DateTime::parse_from_rfc3339(&task.start_time).unwrap();
                let stop_time = DateTime::parse_from_rfc3339(&task.stop_time).unwrap();

                let duration = stop_time - start_time;
                *imp.total_time.borrow_mut() += duration.num_seconds();
            }
        }
        // Format total time to readable string
        let h = *imp.total_time.borrow() / 3600;
        let m = *imp.total_time.borrow() % 3600 / 60;
        let s = *imp.total_time.borrow() % 60;
        let mut total_time_str = format!("{:02}:{:02}:{:02}", h, m, s);

        if !settings_manager::get_bool("show-seconds") {
            total_time_str = format!("{:02}:{:02}", h, m);
        }
        // Display task's total time
        imp.total_time_label.set_text(&total_time_str);
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        let imp = imp::FurTaskRow::from_obj(&self);
        imp.tasks.lock().unwrap().to_vec()
    }

    pub fn get_total_time(&self) -> i64 {
        let imp = imp::FurTaskRow::from_obj(&self);
        *imp.total_time.borrow()
    }
}
