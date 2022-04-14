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
use adw::prelude::{PreferencesPageExt, PreferencesGroupExt};
use gettextrs::*;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*};
use chrono::{DateTime, Local, Duration};

use crate::ui::FurTasksGroup;
use crate::database;
use crate::settings_manager;

mod imp {
    use super::*;
    use glib::subclass;
    use gtk::CompositeTemplate;
    use std::cell::RefCell;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/lakoliu/Furtherance/gtk/tasks_page.ui")]
    pub struct FurTasksPage {
        pub all_groups: RefCell<Vec<FurTasksGroup>>,
    }


    #[glib::object_subclass]
    impl ObjectSubclass for FurTasksPage {
        const NAME: &'static str = "FurTasksPage";
        type ParentType = adw::PreferencesPage;
        type Type = super::FurTasksPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FurTasksPage {
        fn constructed(&self, obj: &Self::Type) {
            obj.setup_widgets();
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for FurTasksPage {}
    impl PreferencesPageImpl for FurTasksPage {}
}

glib::wrapper! {
    pub struct FurTasksPage(
        ObjectSubclass<imp::FurTasksPage>)
        @extends gtk::Widget, adw::PreferencesPage;
}

impl FurTasksPage {
    fn setup_widgets(&self) {
        self.build_task_list();
    }

    pub fn clear_task_list(&self) {
        let imp = imp::FurTasksPage::from_instance(&self);

        for group in &*imp.all_groups.borrow() {
            self.remove(group);
        }

        imp.all_groups.borrow_mut().clear();
    }

    pub fn build_task_list(&self) {
        let imp = imp::FurTasksPage::from_instance(&self);

        let mut tasks_list = database::retrieve().unwrap();

        // Reversing chronological order of tasks_list
        tasks_list.reverse();
        let mut uniq_date_list: Vec<String> = Vec::new();
        let mut same_date_list: Vec<database::Task> = Vec::new();
        let mut tasks_sorted_by_day: Vec<Vec<database::Task>> = Vec::new();

        // Go through tasks list and look at all dates
        let mut i: u32 = 1;
        let len = tasks_list.len() as u32;
        for task in tasks_list {
            let task_clone = task.clone();
            let date = DateTime::parse_from_rfc3339(&task.start_time).unwrap();
            let date = date.format("%h %e").to_string();
            if !uniq_date_list.contains(&date) {
                // if same_date_list is empty, push "date" to it
                // if it is not empty, push it to a vec of vecs, and then clear it
                // and push "date" to it
                if same_date_list.is_empty() {
                    same_date_list.push(task_clone);
                } else {
                    tasks_sorted_by_day.push(same_date_list.clone());
                    same_date_list.clear();
                    same_date_list.push(task_clone);
                }
                uniq_date_list.push(date);
            } else {
                // otherwise push the task to the list of others with the same date
                same_date_list.push(task_clone);
            }
            // If this is the last iteration, push the list of objects to sorted_by_day
            if i == len {
                tasks_sorted_by_day.push(same_date_list.clone());
            }
            i += 1;
            if settings_manager::get_bool("limit-tasks") {
                if uniq_date_list.len() > settings_manager::get_int("limit-days") as usize {
                    if same_date_list.len() > 0 && i != len {
                        tasks_sorted_by_day.push(same_date_list.clone());
                    }
                    uniq_date_list.pop();
                    break;
                }
            }
        }

        // Create FurTasksGroups for all unique days
        let now = Local::now();
        let yesterday = now - Duration::days(1);
        let yesterday = yesterday.format("%h %e").to_string();
        let today = now.format("%h %e").to_string();
        for i in 0..uniq_date_list.len() {
            let group = FurTasksGroup::new();
            if uniq_date_list[i] == today {
                group.set_title(&gettext("Today"));
            } else if uniq_date_list[i] == yesterday{
                group.set_title(&gettext("Yesterday"));
            } else {
                group.set_title(&uniq_date_list[i]);
            }

            self.add(&group);
            group.add_task_model(tasks_sorted_by_day[i].clone());

            // Set total time for each day
            if settings_manager::get_bool("show-daily-sums") {
                let day_total_time = group.get_total_day_time();
                // Format total time to readable string
                let h = day_total_time / 3600;
                let m = day_total_time % 3600 / 60;
                let s = day_total_time % 60;
                let mut total_time_str = format!("{:02}:{:02}:{:02}", h, m, s);
                if !settings_manager::get_bool("show-seconds") {
                    total_time_str = format!("{:02}:{:02}", h, m);
                }
                group.set_description(Some(&total_time_str));
            }

            imp.all_groups.borrow_mut().push(group);
        }
    }
}

