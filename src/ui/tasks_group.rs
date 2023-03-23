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

use adw::subclass::prelude::*;
use gtk::{glib, prelude::*};

use crate::database;
use crate::settings_manager;
use crate::ui::FurTaskRow;

mod imp {
    use super::*;
    use glib::subclass;
    use gtk::CompositeTemplate;

    use std::cell::RefCell;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/lakoliu/Furtherance/gtk/tasks_group.ui")]
    pub struct FurTasksGroup {
        #[template_child]
        pub listbox_box: TemplateChild<gtk::Box>,

        pub models: RefCell<Vec<gtk::SortListModel>>,
        pub day_total_time: RefCell<i64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FurTasksGroup {
        const NAME: &'static str = "FurTasksGroup";
        type ParentType = adw::PreferencesGroup;
        type Type = super::FurTasksGroup;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FurTasksGroup {}
    impl WidgetImpl for FurTasksGroup {}
    impl PreferencesGroupImpl for FurTasksGroup {}
}

glib::wrapper! {
    pub struct FurTasksGroup(
        ObjectSubclass<imp::FurTasksGroup>)
        @extends gtk::Widget, adw::PreferencesGroup;
}

impl FurTasksGroup {
    pub fn new() -> Self {
        glib::Object::new::<FurTasksGroup>()
    }

    pub fn add_task_model(&self, tasks: Vec<database::Task>) {
        let imp = imp::FurTasksGroup::from_obj(&self);

        let listbox = gtk::ListBox::new();
        listbox.add_css_class("content");
        listbox.set_selection_mode(gtk::SelectionMode::None);
        imp.listbox_box.append(&listbox);

        // Check if tasks have the same name. If they do, make one listbox row for all of them.
        // If they don't, move on.
        let mut tasks_by_name: Vec<Vec<database::Task>> = Vec::new();

        for task in &tasks {
            let mut unique = true;
            for i in 0..tasks_by_name.len() {
                if tasks_by_name[i][0].task_name == task.task_name
                    && ((settings_manager::get_bool("show-tags")
                        && tasks_by_name[i][0].tags == task.tags)
                        || !settings_manager::get_bool("show-tags"))
                {
                    tasks_by_name[i].push(task.clone());
                    unique = false;
                }
            }
            if unique {
                // Add unique task to list for group name
                let mut new_name_list: Vec<database::Task> = Vec::new();
                new_name_list.push(task.clone());
                tasks_by_name.push(new_name_list);
            }
        }

        for same_name in tasks_by_name {
            let listbox_row = FurTaskRow::new();
            listbox_row.set_row_labels(same_name);
            *imp.day_total_time.borrow_mut() += listbox_row.get_total_time();
            listbox.append(&listbox_row);
        }

        listbox.connect_row_activated(move |_, row| {
            row.activate_action("task-row.open-details", None).unwrap();
        });
    }

    pub fn get_total_day_time(&self) -> i64 {
        let imp = imp::FurTasksGroup::from_obj(&self);
        *imp.day_total_time.borrow()
    }
}
