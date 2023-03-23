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
use chrono::{offset::TimeZone, Date, DateTime, Datelike, Duration, Local, NaiveDate};
use gettextrs::*;
use glib::clone;
use gtk::{glib, prelude::*, CompositeTemplate};
use itertools::Itertools;

use crate::database::{self, SortOrder, TaskSort};
use crate::ui::FurtheranceWindow;
use crate::FurtheranceApplication;

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/lakoliu/Furtherance/gtk/report.ui")]
    pub struct FurReport {
        #[template_child]
        pub range_combo: TemplateChild<gtk::ComboBoxText>,
        #[template_child]
        pub date_range_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub start_date_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub end_date_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub format_error: TemplateChild<gtk::Label>,
        #[template_child]
        pub start_end_error: TemplateChild<gtk::Label>,
        #[template_child]
        pub filter_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub filter_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub filter_combo: TemplateChild<gtk::ComboBoxText>,
        #[template_child]
        pub filter_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub results_tree: TemplateChild<gtk::TreeView>,
        #[template_child]
        pub sort_by_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub sort_by_task: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub sort_by_tag: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub refresh_btn: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FurReport {
        const NAME: &'static str = "FurReport";
        type ParentType = adw::Window;
        type Type = super::FurReport;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FurReport {
        fn constructed(&self) {
            let obj = self.obj();
            obj.setup_widgets();
            self.parent_constructed();
        }
    }

    impl WidgetImpl for FurReport {}

    impl WindowImpl for FurReport {}

    impl AdwWindowImpl for FurReport {}
}

glib::wrapper! {
    pub struct FurReport(ObjectSubclass<imp::FurReport>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl FurReport {
    pub fn new() -> Self {
        let dialog: Self = glib::Object::new::<FurReport>();

        let window = FurtheranceWindow::default();
        dialog.set_transient_for(Some(&window));

        let app = FurtheranceApplication::default();
        app.add_window(&window);

        dialog
    }

    pub fn setup_widgets(&self) {
        let imp = imp::FurReport::from_obj(self);

        imp.range_combo.set_active_id(Some("week_item"));
        imp.filter_combo.set_active_id(Some("tasks_item"));

        imp.range_combo
            .connect_changed(clone!(@weak self as this => move |combo|{
                let imp = imp::FurReport::from_obj(&this);
                if combo.active_id().unwrap() != "date_range_item" {
                    imp.date_range_box.set_visible(false);
                    this.refresh_report();
                } else {
                    imp.date_range_box.set_visible(true);
                }
            }));

        imp.filter_check
            .connect_toggled(clone!(@weak self as this => move |_|{
                let imp = imp::FurReport::from_obj(&this);
                if imp.filter_box.get_visible() {
                    imp.filter_box.set_visible(false);
                } else {
                    imp.filter_box.set_visible(true);
                }
            }));

        imp.filter_combo
            .connect_changed(clone!(@weak self as this => move |combo|{
                let imp = imp::FurReport::from_obj(&this);
                if combo.active_id().unwrap() == "tasks_item" {
                    imp.filter_entry.set_placeholder_text(Some(&gettext("Task, Task 2")));
                } else {
                    imp.filter_entry.set_placeholder_text(Some(&gettext("tag, tag 2")));
                }
            }));

        imp.refresh_btn
            .connect_clicked(clone!(@weak self as this => move |_|{
                this.refresh_report();
            }));

        let renderer = gtk::CellRendererText::new();
        let task_column =
            gtk::TreeViewColumn::with_attributes(&gettext("Task"), &renderer, &[("text", 0)]);
        task_column.set_expand(true);
        task_column.set_fixed_width(100);
        task_column.set_resizable(true);
        let duration_column =
            gtk::TreeViewColumn::with_attributes(&gettext("Duration"), &renderer, &[("text", 1)]);
        duration_column.set_expand(false);
        duration_column.set_resizable(true);
        imp.results_tree.append_column(&task_column);
        imp.results_tree.append_column(&duration_column);
        imp.results_tree.set_enable_search(false);

        self.refresh_report();
    }

    fn refresh_report(&self) {
        let imp = imp::FurReport::from_obj(self);
        imp.format_error.set_visible(false);
        imp.start_end_error.set_visible(false);

        let results_model = gtk::TreeStore::new(&[String::static_type(), String::static_type()]);

        let task_list = database::retrieve(TaskSort::StartTime, SortOrder::Descending).unwrap();

        // Get date range
        let active_range = imp.range_combo.active_id().unwrap();
        let today = Local::today();
        let range_start_date: Date<Local>;
        let mut range_end_date = today;
        if active_range == "week_item" {
            range_start_date = today - Duration::days(6);
        } else if active_range == "month_item" {
            let days_ago = today.day() - 1;
            range_start_date = today - Duration::days(days_ago.into());
        } else if active_range == "30_days_item" {
            range_start_date = today - Duration::days(29);
        } else if active_range == "six_months_item" {
            range_start_date = today - Duration::days(179);
        } else if active_range == "year_item" {
            range_start_date = today - Duration::days(364);
        } else {
            let input_start_date =
                NaiveDate::parse_from_str(&imp.start_date_entry.text(), "%m/%d/%Y");
            let input_end_date = NaiveDate::parse_from_str(&imp.end_date_entry.text(), "%m/%d/%Y");
            // Check if user entered dates properly
            if let Err(_) = input_start_date {
                imp.format_error.set_visible(true);
                results_model.clear();
                imp.results_tree.set_model(Some(&results_model));
                return;
            }
            if let Err(_) = input_end_date {
                imp.format_error.set_visible(true);
                results_model.clear();
                imp.results_tree.set_model(Some(&results_model));
                return;
            }
            // Start date cannot be after end date
            if (input_end_date.unwrap() - input_start_date.unwrap()).num_days() < 0 {
                imp.start_end_error.set_visible(true);
                results_model.clear();
                return;
            }
            range_start_date = Local.from_local_date(&input_start_date.unwrap()).unwrap();
            range_end_date = Local.from_local_date(&input_end_date.unwrap()).unwrap();
        }

        let mut total_time: i64 = 0;
        let mut tasks_in_range: Vec<(database::Task, i64)> = Vec::new();
        let mut user_chosen_tags: Vec<String> = Vec::new();
        let mut only_this_tag = false;
        for task in task_list {
            let start = DateTime::parse_from_rfc3339(&task.start_time)
                .unwrap()
                .with_timezone(&Local);
            let stop = DateTime::parse_from_rfc3339(&task.stop_time)
                .unwrap()
                .with_timezone(&Local);
            // Check if start time is in date range and if not remove it from task_list
            let start_date = start.date();
            if start_date >= range_start_date && start_date <= range_end_date {
                // Sort by only selected tasks or tags if filter is selected
                if imp.filter_check.is_active() && !imp.filter_entry.text().trim().is_empty() {
                    if imp.filter_combo.active_id().unwrap() == "tasks_item" {
                        // Create a vec of tasks to match from written tasks
                        let chosen_tasks = imp.filter_entry.text();
                        let mut split_tasks: Vec<&str> = chosen_tasks.trim().split(",").collect();
                        // Trim whitespace around each task
                        split_tasks = split_tasks.iter().map(|x| x.trim()).collect();
                        // Don't allow empty tasks
                        split_tasks.retain(|&x| !x.trim().is_empty());
                        // Handle duplicate tasks
                        split_tasks = split_tasks.into_iter().unique().collect();
                        // Lowercase tags
                        let lower_tasks: Vec<String> =
                            split_tasks.iter().map(|x| x.to_lowercase()).collect();

                        if lower_tasks.contains(&task.task_name.to_lowercase()) {
                            let duration = stop - start;
                            let duration = duration.num_seconds();
                            tasks_in_range.push((task, duration));
                            total_time += duration;
                        }
                    } else if imp.filter_combo.active_id().unwrap() == "tags_item" {
                        // Split user chosen tags
                        let chosen_tasgs = imp.filter_entry.text();
                        let mut split_tags: Vec<&str> = chosen_tasgs.trim().split(",").collect();
                        // Trim whitespace around each tag
                        split_tags = split_tags.iter().map(|x| x.trim()).collect();
                        // Don't allow empty tags
                        split_tags.retain(|&x| !x.trim().is_empty());
                        // Handle duplicate tags
                        split_tags = split_tags.into_iter().unique().collect();
                        // Lowercase tags
                        user_chosen_tags = split_tags.iter().map(|x| x.to_lowercase()).collect();

                        // Split task's tags
                        let mut split_tags: Vec<&str> = task.tags.trim().split("#").collect();
                        // Trim whitespace around each tag
                        split_tags = split_tags.iter().map(|x| x.trim()).collect();

                        // Only keep tasks that contain the user's chosen tags
                        split_tags.retain(|&x| user_chosen_tags.contains(&x.to_string()));
                        if !split_tags.is_empty() {
                            let duration = stop - start;
                            let duration = duration.num_seconds();
                            tasks_in_range.push((task, duration));
                            total_time += duration;
                        }

                        only_this_tag = true;
                    }
                } else {
                    let duration = stop - start;
                    let duration = duration.num_seconds();
                    tasks_in_range.push((task, duration));
                    total_time += duration;
                }
            }
        }

        let all_tasks_iter: gtk::TreeIter;
        if tasks_in_range.is_empty() {
            all_tasks_iter = results_model.insert_with_values(
                None,
                None,
                &[(0, &gettext("No Results")), (1, &"")],
            );
        } else {
            let total_time_str = FurReport::format_duration(total_time);
            all_tasks_iter = results_model.insert_with_values(
                None,
                None,
                &[(0, &gettext("All Results")), (1, &total_time_str)],
            );
        }

        if imp.sort_by_task.is_active() {
            let mut tasks_by_name: Vec<Vec<(database::Task, i64)>> = Vec::new();
            for (task, task_duration) in tasks_in_range {
                let mut unique = true;
                for i in 0..tasks_by_name.len() {
                    let (tbn, _) = &tasks_by_name[i][0];
                    if tbn.task_name == task.task_name {
                        tasks_by_name[i].push((task.clone(), task_duration));
                        unique = false;
                    }
                }
                if unique {
                    // Add unique task to list for group name
                    let mut new_name_list: Vec<(database::Task, i64)> = Vec::new();
                    new_name_list.push((task.clone(), task_duration));
                    tasks_by_name.push(new_name_list);
                }
            }

            let mut sorted_tasks_by_duration: Vec<(String, i64, Vec<(String, i64)>)> = Vec::new();
            for tbn in tasks_by_name {
                let mut total_duration: i64 = 0;
                let mut tags_dur: Vec<(String, i64)> = Vec::new();
                let task_name = tbn[0].0.task_name.to_string();
                for tbn_tuple in tbn {
                    let (task, task_duration) = tbn_tuple;
                    total_duration += task_duration;

                    let mut split_tags: Vec<&str> = task.tags.split("#").collect();
                    split_tags = split_tags.iter().map(|x| x.trim()).collect();
                    split_tags.retain(|&x| !x.trim().is_empty());
                    if !split_tags.is_empty() {
                        let mut formatted_tags = split_tags.join(" #");
                        formatted_tags = format!("#{}", formatted_tags);
                        let mut unique = true;
                        for i in 0..tags_dur.len() {
                            let (tags, dur) = &tags_dur[i];
                            if tags == &formatted_tags {
                                let new_dur = dur + task_duration;
                                tags_dur[i] = (formatted_tags.clone(), new_dur);
                                unique = false;
                            }
                        }
                        if unique {
                            tags_dur.push((formatted_tags, task_duration))
                        }
                    }
                }

                // Sort tasks and tags in descending order by duration
                tags_dur.sort_by_key(|k| k.1);
                tags_dur.reverse();
                sorted_tasks_by_duration.push((task_name, total_duration, tags_dur));
                sorted_tasks_by_duration.sort_by_key(|k| k.1);
                sorted_tasks_by_duration.reverse();
            }

            for stbd in sorted_tasks_by_duration {
                let header_iter = results_model.append(Some(&all_tasks_iter));
                for (task, task_duration) in stbd.2 {
                    let _child_iter = results_model.insert_with_values(
                        Some(&header_iter),
                        None,
                        &[(0, &task), (1, &FurReport::format_duration(task_duration))],
                    );
                }
                results_model.set(
                    &header_iter,
                    &[(0, &stbd.0), (1, &FurReport::format_duration(stbd.1))],
                );
            }
        } else if imp.sort_by_tag.is_active() {
            let mut tasks_by_tag: Vec<Vec<(String, database::Task, i64)>> = Vec::new();
            for (task, task_duration) in tasks_in_range {
                let mut split_tags: Vec<&str> = task.tags.split("#").collect();
                // Trim whitespace around each tag
                split_tags = split_tags.iter().map(|x| x.trim()).collect();
                for tag in split_tags {
                    let mut unique = true;
                    for i in 0..tasks_by_tag.len() {
                        let (tbt_tag, _, _) = &tasks_by_tag[i][0];
                        if tbt_tag == tag {
                            tasks_by_tag[i].push((tag.to_string(), task.clone(), task_duration));
                            unique = false;
                        }
                    }
                    if unique {
                        // Add unique task to list for group name
                        let mut new_name_list: Vec<(String, database::Task, i64)> = Vec::new();
                        new_name_list.push((tag.to_string(), task.clone(), task_duration));
                        tasks_by_tag.push(new_name_list);
                    }
                }
            }

            let mut sorted_tasks_by_duration: Vec<(String, i64, Vec<(String, i64)>)> = Vec::new();
            for tbt in tasks_by_tag {
                let mut total_duration: i64 = 0;
                let mut tasks_dur: Vec<(String, i64)> = Vec::new();
                let tag_name = format!("#{}", tbt[0].0);
                for tbt_tuple in tbt {
                    let (_, task, tag_duration) = tbt_tuple;
                    total_duration += tag_duration;

                    let mut unique = true;
                    for i in 0..tasks_dur.len() {
                        let (td_task, dur) = &tasks_dur[i];
                        if td_task == &task.task_name {
                            let new_dur = dur + tag_duration;
                            tasks_dur[i] = (task.task_name.clone(), new_dur);
                            unique = false;
                        }
                    }
                    if unique {
                        tasks_dur.push((task.task_name.clone(), tag_duration))
                    }
                }

                // Sort tags and tasks in descending order by duration
                tasks_dur.sort_by_key(|k| k.1);
                tasks_dur.reverse();
                sorted_tasks_by_duration.push((tag_name, total_duration, tasks_dur));
                sorted_tasks_by_duration.sort_by_key(|k| k.1);
                sorted_tasks_by_duration.reverse();
            }

            for mut stbd in sorted_tasks_by_duration {
                if !only_this_tag
                    || (only_this_tag && user_chosen_tags.contains(&stbd.0[1..].to_string()))
                {
                    let header_iter = results_model.append(Some(&all_tasks_iter));
                    for (task, task_duration) in stbd.2 {
                        let _child_iter = results_model.insert_with_values(
                            Some(&header_iter),
                            None,
                            &[(0, &task), (1, &FurReport::format_duration(task_duration))],
                        );
                    }
                    if stbd.0 == "#" {
                        stbd.0 = gettext("no tags").to_string();
                    }
                    results_model.set(
                        &header_iter,
                        &[(0, &stbd.0), (1, &FurReport::format_duration(stbd.1))],
                    );
                }
            }
        }

        imp.results_tree.set_model(Some(&results_model));
        // Automatically expand All Tasks row
        let all_tasks_path = gtk::TreePath::new_first();
        imp.results_tree.expand_row(&all_tasks_path, false);
    }

    fn format_duration(total_time: i64) -> String {
        // Format total time to readable string
        let h = total_time / 3600;
        let m = total_time % 3600 / 60;
        let s = total_time % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    }
}

