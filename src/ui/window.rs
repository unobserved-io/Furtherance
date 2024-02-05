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
use adw::subclass::prelude::AdwApplicationWindowImpl;
use chrono::{offset::TimeZone, DateTime, Duration as ChronDur, Local, NaiveDateTime, ParseError};
use dbus::blocking::Connection;
use directories::ProjectDirs;
use gettextrs::*;
use glib::{clone, timeout_add_local, ControlFlow};
use gtk::subclass::prelude::*;
use gtk::{Application, gio, glib, CompositeTemplate};
use itertools::Itertools;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::fs::{create_dir_all, remove_file, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;

use crate::config;
use crate::database::{self, SortOrder, TaskSort};
use crate::settings_manager;
use crate::ui::FurHistoryBox;
use crate::FurtheranceApplication;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/lakoliu/Furtherance/gtk/window.ui")]
    pub struct FurtheranceWindow {
        // Template widgets
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub add_task: TemplateChild<gtk::Button>,

        #[template_child]
        pub win_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub watch: TemplateChild<gtk::Label>,
        #[template_child]
        pub task_input: TemplateChild<gtk::Entry>,
        #[template_child]
        pub start_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub history_box: TemplateChild<FurHistoryBox>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,

        pub stored_idle: Mutex<u64>,
        pub idle_notified: Mutex<bool>,
        pub idle_time_reached: Mutex<bool>,
        pub subtract_idle: Mutex<bool>,
        pub idle_start_time: Mutex<String>,
        pub running: Mutex<bool>,
        pub pomodoro_continue: Mutex<bool>,
        pub idle_dialog: Mutex<gtk::MessageDialog>,

        // We have to keep a reference to the current popped up filechooser dialog
        pub filechooser: RefCell<gtk::FileChooserNative>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FurtheranceWindow {
        const NAME: &'static str = "FurtheranceWindow";
        type Type = super::FurtheranceWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            FurHistoryBox::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FurtheranceWindow {
        fn constructed(&self) {
            let obj = self.obj();
            obj.setup_widgets();
            obj.setup_signals();
            obj.setup_settings();
            self.parent_constructed();
        }
    }
    impl WidgetImpl for FurtheranceWindow {}
    impl WindowImpl for FurtheranceWindow {}
    impl ApplicationWindowImpl for FurtheranceWindow {}
    impl AdwApplicationWindowImpl for FurtheranceWindow {}
}

glib::wrapper! {
    pub struct FurtheranceWindow(ObjectSubclass<imp::FurtheranceWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl FurtheranceWindow {
    const MIN_PREFIX_LENGTH: i32 = 3;

    pub fn new(app: &Application) -> Self {
        glib::Object::builder()
            .property("application", Some(app))
            .build()
    }


    pub fn display_toast(&self, text: &str) {
        // Display in-app notifications
        let imp = imp::FurtheranceWindow::from_obj(self);
        let toast = adw::Toast::new(text);
        imp.toast_overlay.add_toast(toast);
    }

    fn set_watch_time(&self, text: &str) {
        // Update watch time while timer is running
        let imp = imp::FurtheranceWindow::from_obj(self);
        imp.watch.set_text(text);
        if settings_manager::get_bool("notify-of-idle") {
            self.check_user_idle();
        }
    }

    pub fn save_task(&self, start_time: DateTime<Local>, mut stop_time: DateTime<Local>) {
        // Save the most recent task to the database and clear the task_input field
        let imp = imp::FurtheranceWindow::from_obj(self);

        if *imp.subtract_idle.lock().unwrap() {
            let idle_start =
                DateTime::parse_from_rfc3339(&imp.idle_start_time.lock().unwrap()).unwrap();
            stop_time = idle_start.with_timezone(&Local);
            *imp.subtract_idle.lock().unwrap() = false;
        }

        let (task_name, tag_list) = self.split_tags_and_task();
        let _ = database::db_write(&task_name, start_time, stop_time, tag_list);
        imp.task_input.set_text("");
        imp.history_box.create_tasks_page();
        self.reset_idle();
    }

    pub fn reset_history_box(&self) {
        let imp = imp::FurtheranceWindow::from_obj(self);
        imp.history_box.create_tasks_page();
    }

    pub fn reset_autocomplete(&self) {
        let imp = imp::FurtheranceWindow::from_obj(self);
        if settings_manager::get_bool("autocomplete") {
            imp.task_input.set_completion(Some(&FurtheranceWindow::create_autocomplete()));
        } else {
            imp.task_input.set_completion(None);
        }
    }

    fn setup_widgets(&self) {
        let imp = imp::FurtheranceWindow::from_obj(self);

        // Set initial minimum height and alignment
        let is_saved_task: bool = match database::check_for_tasks() {
            Ok(_) => true,
            Err(_) => false,
        };
        if is_saved_task {
            self.vertical_align(gtk::Align::Start);
        }

        // Development mode
        if config::PROFILE == "development" {
            self.add_css_class("devel");
        }

        *imp.pomodoro_continue.lock().unwrap() = false;
        imp.start_button.set_sensitive(false);
        imp.start_button.add_css_class("suggested-action");
        self.refresh_timer();

        if settings_manager::get_bool("autocomplete") {
            imp.task_input.set_completion(Some(&FurtheranceWindow::create_autocomplete()));
        }

        imp.task_input.grab_focus();

        if settings_manager::get_bool("autosave") {
            self.check_for_autosave();
        }
    }

    fn setup_signals(&self) {
        let imp = imp::FurtheranceWindow::from_obj(self);
        *imp.running.lock().unwrap() = false;
        let start_time = Rc::new(RefCell::new(Local::now()));
        let stop_time = Rc::new(RefCell::new(Local::now()));

        imp.task_input
            .connect_changed(clone!(@weak self as this => move |task_input| {
                let imp2 = imp::FurtheranceWindow::from_obj(&this);
                let task_input_text = task_input.text();
                let mut split_tags: Vec<String> = task_input_text.split('#').map(|tag| String::from(tag.trim())).collect();
                let task_name = split_tags.remove(0);
                if task_name.is_empty() {
                    imp2.start_button.set_sensitive(false);
                } else {
                    imp2.start_button.set_sensitive(true);
                }

                if settings_manager::get_bool("autocomplete") {
                    if task_input.text().len() >= FurtheranceWindow::MIN_PREFIX_LENGTH.try_into().unwrap() {
                        if task_input.completion().is_some() {
                            let task_autocomplete = task_input.completion().unwrap();
                            let model = Self::update_list_model(task_name.to_string(), split_tags).unwrap();
                            task_autocomplete.set_model(Some(&model));
                        }
                    }
                }
            }));

        imp.start_button.connect_clicked(clone!(@weak self as this => move |button| {
            let imp2 = imp::FurtheranceWindow::from_obj(&this);
            if !*imp2.running.lock().unwrap() {
                // Remove auto-complete to prevent drop-down from sticking
                imp2.task_input.set_completion(None);

                if settings_manager::get_bool("pomodoro") && !*imp2.pomodoro_continue.lock().unwrap() {
                    let pomodoro_time = settings_manager::get_int("pomodoro-time");
                    let mut secs: i32 = 0;
                    let mut secs_only: i32 = 0;
                    let mut mins: i32 = pomodoro_time;
                    let mut hrs: i32 = mins / 60;
                    mins %= 60;

                    *imp2.running.lock().unwrap() = true;
                    *start_time.borrow_mut() = Local::now();
                    let timer_start = *start_time.borrow();
                    imp2.task_input.set_sensitive(false);
                    let duration = Duration::new(1,0);
                    timeout_add_local(duration, clone!(@strong this as this_clone => move || {
                        let imp3 = imp::FurtheranceWindow::from_obj(&this_clone);
                        if *imp3.running.lock().unwrap() {
                            secs -= 1;
                            if secs < 0 {
                                secs = 59;
                                mins -= 1;
                                if mins < 0 {
                                    mins = 59;
                                    hrs -= 1;
                                }
                            }
                            let watch_text: &str = &format!("{:02}:{:02}:{:02}", hrs, mins, secs).to_string();
                            this_clone.set_watch_time(watch_text);

                            if settings_manager::get_bool("inclusive-total") {
                                secs_only += 1;
                                imp3.history_box.set_todays_time(secs_only);
                            }
                        }
                        if settings_manager::get_bool("autosave") {
                            let autosave_mins = settings_manager::get_int("autosave-time");
                            let total_elapsed = (pomodoro_time * 60) - (hrs * 3600) - (mins * 60) - secs;
                            if total_elapsed % (autosave_mins * 60) == 0 {
                                this_clone.write_autosave(timer_start);
                            }
                        }
                        if hrs == 0 && mins == 0 && secs == 0 {
                            let timer_stop = Local::now();
                            *imp3.running.lock().unwrap() = false;
                            this_clone.pomodoro_over(timer_start, timer_stop);
                        }
                        if *imp3.running.lock().unwrap() {
                            ControlFlow::Continue
                        } else {
                            ControlFlow::Break
                        }
                    }));
                } else {
                    let mut secs: i32 = 0;
                    let mut secs_only: i32 = 0;
                    let mut mins: i32 = 0;
                    let mut hrs: i32 = 0;

                    if *imp2.pomodoro_continue.lock().unwrap() {
                        let pomodoro_start_time = *start_time.borrow();
                        let now_time = Local::now();
                        let continue_time = now_time - pomodoro_start_time;
                        let continue_time = continue_time.num_seconds() as i32;
                        hrs = continue_time / 3600;
                        mins = continue_time % 3600 / 60;
                        secs = continue_time % 60;
                        let watch_text: &str = &format!("{:02}:{:02}:{:02}", hrs, mins, secs).to_string();
                        this.set_watch_time(watch_text);

                        if settings_manager::get_bool("inclusive-total") {
                            imp2.history_box.set_todays_stored_secs(continue_time);
                        }

                        *imp2.pomodoro_continue.lock().unwrap() = false;
                    } else {
                        *start_time.borrow_mut() = Local::now();
                    }

                    *imp2.running.lock().unwrap() = true;
                    imp2.task_input.set_sensitive(false);
                    let autosave_start = *start_time.borrow();
                    let duration = Duration::new(1,0);
                    timeout_add_local(duration, clone!(@strong this as this_clone => move || {
                        let imp3 = imp::FurtheranceWindow::from_obj(&this_clone);
                        if *imp3.running.lock().unwrap() {
                            secs += 1;
                            if secs > 59 {
                                secs = 0;
                                mins += 1;
                                if mins > 59 {
                                    mins = 0;
                                    hrs += 1;
                                }
                            }
                            let watch_text: &str = &format!("{:02}:{:02}:{:02}", hrs, mins, secs).to_string();
                            this_clone.set_watch_time(watch_text);

                            if settings_manager::get_bool("inclusive-total") {
                                secs_only +=1;
                                imp3.history_box.set_todays_time(secs_only);
                            }

                            if settings_manager::get_bool("autosave") {
                                let autosave_mins = settings_manager::get_int("autosave-time");
                                let total_elapsed = (hrs * 3600) + (mins * 60) + secs;
                                if total_elapsed % (autosave_mins * 60) == 0 {
                                    this_clone.write_autosave(autosave_start);
                                }
                            }
                        }
                        if *imp3.running.lock().unwrap() {
                            ControlFlow::Continue
                        } else {
                            ControlFlow::Break
                        }
                    }));
                }
                button.set_icon_name("media-playback-stop-symbolic");
            } else {
                *stop_time.borrow_mut() = Local::now();
                *imp2.running.lock().unwrap() = false;
                button.set_icon_name("media-playback-start-symbolic");
                this.refresh_timer();
                imp2.task_input.set_sensitive(true);

                // Re-add auto-complete
                if settings_manager::get_bool("autocomplete") {
                    imp2.task_input.set_completion(Some(&FurtheranceWindow::create_autocomplete()));
                }

                this.save_task(*start_time.borrow(), *stop_time.borrow());
                FurtheranceWindow::delete_autosave();
            }
        }));

        imp.add_task.connect_clicked(clone!(@weak self as this => move |_| {
            let dialog = gtk::MessageDialog::new(
                Some(&this),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Question,
                gtk::ButtonsType::None,
                &format!("<span size='x-large' weight='bold'>{}</span>", &gettext("New Task")),
            );
            dialog.set_use_markup(true);
            dialog.add_buttons(&[
                (&gettext("Cancel"), gtk::ResponseType::Cancel),
                (&gettext("Add"), gtk::ResponseType::Ok)
            ]);

            let message_area = dialog.message_area().downcast::<gtk::Box>().unwrap();
            let vert_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
            let task_name_edit = gtk::Entry::new();
            task_name_edit.set_placeholder_text(Some(&gettext("Task Name")));
            let task_tags_edit = gtk::Entry::new();
            let tags_placeholder = format!("#{}", &gettext("tags"));
            task_tags_edit.set_placeholder_text(Some(&tags_placeholder));

            let labels_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
            labels_box.set_homogeneous(true);
            let start_label = gtk::Label::new(Some(&gettext("Start")));
            start_label.add_css_class("title-4");
            let stop_label = gtk::Label::new(Some(&gettext("Stop")));
            stop_label.add_css_class("title-4");
            let times_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
            times_box.set_homogeneous(true);

            let stop_time = Local::now();
            let start_time = stop_time - ChronDur::seconds(1);

            let time_formatter = "%F %H:%M:%S";
            let time_formatter_no_secs = "%F %H:%M";

            let mut start_time_w_year = start_time.format(time_formatter).to_string();
            if !settings_manager::get_bool("show-seconds") {
                start_time_w_year = start_time.format(time_formatter_no_secs).to_string();
            }
            let mut stop_time_w_year = stop_time.format(time_formatter).to_string();
            if !settings_manager::get_bool("show-seconds") {
                stop_time_w_year = stop_time.format(time_formatter_no_secs).to_string();
            }
            let start_time_edit = gtk::Entry::new();
            start_time_edit.set_text(&start_time_w_year);
            let stop_time_edit = gtk::Entry::new();
            stop_time_edit.set_text(&stop_time_w_year);

            let instructions = gtk::Label::new(Some(
                &gettext("*Use the format YYYY-MM-DD HH:MM:SS")));
            if !settings_manager::get_bool("show-seconds") {
                instructions.set_text(&gettext("*Use the format YYYY-MM-DD HH:MM"));
            }
            instructions.set_visible(false);
            instructions.add_css_class("error_message");

            let time_error = gtk::Label::new(Some(
                &gettext("*Start time cannot be later than stop time.")));
            time_error.set_visible(false);
            time_error.add_css_class("error_message");

            let future_error = gtk::Label::new(Some(
                &gettext("*Time cannot be in the future.")));
            future_error.set_visible(false);
            future_error.add_css_class("error_message");

            let name_error = gtk::Label::new(Some(
                &gettext("*Task name cannot be blank.")));
            name_error.set_visible(false);
            name_error.add_css_class("error_message");

            vert_box.append(&task_name_edit);
            vert_box.append(&task_tags_edit);
            labels_box.append(&start_label);
            labels_box.append(&stop_label);
            times_box.append(&start_time_edit);
            times_box.append(&stop_time_edit);
            vert_box.append(&labels_box);
            vert_box.append(&times_box);
            vert_box.append(&instructions);
            vert_box.append(&time_error);
            vert_box.append(&future_error);
            vert_box.append(&name_error);
            message_area.append(&vert_box);

            dialog.connect_response(clone!(@strong dialog => move |_ , resp| {
                if resp == gtk::ResponseType::Ok {
                    instructions.set_visible(false);
                    time_error.set_visible(false);
                    future_error.set_visible(false);
                    name_error.set_visible(false);
                    let mut do_not_close = false;
                    let mut new_start_time_local = Local::now();
                    let mut new_stop_time_local = Local::now();

                    // Task Name
                    if task_name_edit.text().trim().is_empty() {
                        name_error.set_visible(true);
                        do_not_close = true;
                    }

                    // Start Time
                    let new_start_time_str = start_time_edit.text();
                    let new_start_time: Result<NaiveDateTime, ParseError>;
                    if settings_manager::get_bool("show-seconds") {
                        new_start_time = NaiveDateTime::parse_from_str(
                                            &new_start_time_str,
                                            time_formatter);
                    } else {
                        new_start_time = NaiveDateTime::parse_from_str(
                                                &new_start_time_str,
                                                time_formatter_no_secs);
                    }
                    if let Err(_) = new_start_time {
                        instructions.set_visible(true);
                        do_not_close = true;
                    } else {
                        new_start_time_local = Local.from_local_datetime(&new_start_time.unwrap()).unwrap();
                        if (Local::now() - new_start_time_local).num_seconds() < 0 {
                            future_error.set_visible(true);
                            do_not_close = true;
                        }
                    }

                    // Stop Time
                    let new_stop_time_str = stop_time_edit.text();
                    let new_stop_time: Result<NaiveDateTime, ParseError>;
                    if settings_manager::get_bool("show-seconds") {
                        new_stop_time = NaiveDateTime::parse_from_str(
                                            &new_stop_time_str,
                                            time_formatter);
                    } else {
                        new_stop_time = NaiveDateTime::parse_from_str(
                                                &new_stop_time_str,
                                                time_formatter_no_secs);
                    }
                    if let Err(_) = new_stop_time {
                        instructions.set_visible(true);
                        do_not_close = true;
                    } else {
                        new_stop_time_local = Local.from_local_datetime(&new_stop_time.unwrap()).unwrap();
                        if (Local::now() - new_stop_time_local).num_seconds() < 0 {
                            future_error.set_visible(true);
                            do_not_close = true;
                        }
                    }

                    // Start time can't be later than stop time
                    if !do_not_close && (new_stop_time_local - new_start_time_local).num_seconds() < 0 {
                        time_error.set_visible(true);
                        do_not_close = true;
                    }

                    // Tags
                    let mut new_tag_list = "".to_string();
                    if !task_tags_edit.text().trim().is_empty() {
                        let new_tags = task_tags_edit.text();
                        let mut split_tags: Vec<&str> = new_tags.trim().split("#").collect();
                        split_tags = split_tags.iter().map(|x| x.trim()).collect();
                        // Don't allow empty tags
                        split_tags.retain(|&x| !x.trim().is_empty());
                        // Handle duplicate tags before they are saved
                        split_tags = split_tags.into_iter().unique().collect();
                        // Lowercase tags
                        let lower_tags: Vec<String> = split_tags.iter().map(|x| x.to_lowercase()).collect();
                        new_tag_list = lower_tags.join(" #");
                    }

                    if !do_not_close {
                        let _ = database::db_write(task_name_edit.text().trim(),
                                                    new_start_time_local,
                                                    new_stop_time_local,
                                                    new_tag_list);
                        this.reset_history_box();
                        dialog.close();
                    }

                } else if resp == gtk::ResponseType::Cancel {
                    dialog.close();
                }
            }));

            dialog.show();
        }));
    }

    fn setup_settings(&self) {
        let imp = imp::FurtheranceWindow::from_obj(self);
        self.reset_idle();

        // Enter starts timer
        let start = imp.start_button.clone();
        self.set_default_widget(Some(&start));
        imp.task_input.set_activates_default(true);
    }

    fn update_list_model(task_name: String, tag_list: Vec<String>) -> Result<gtk::ListStore, anyhow::Error> {
        let col_types: [glib::Type; 1] = [glib::Type::STRING];
        let mut task_list = database::get_list_by_name_and_tags(task_name, tag_list)?;
        task_list.dedup_by(|a, b| a.task_name == b.task_name && a.tags == b.tags);
        let store = gtk::ListStore::new(&col_types);

        for task in task_list {
            store.set(&store.append(), &[(0, &task.to_string())]);
        }
        Ok(store)
    }

    fn get_idle_time(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let c = Connection::new_session()?;

        let p = c.with_proxy(
            "org.gnome.Mutter.IdleMonitor",
            "/org/gnome/Mutter/IdleMonitor/Core",
            Duration::from_millis(5000),
        );
        let (idle_time,): (u64,) =
            p.method_call("org.gnome.Mutter.IdleMonitor", "GetIdletime", ())?;

        Ok(idle_time / 1000)
    }

    fn check_user_idle(&self) {
        let imp = imp::FurtheranceWindow::from_obj(self);
        // Check for user idle
        let idle_time = match self.get_idle_time() {
            Ok(val) => val,
            Err(_) => 1,
        };
        // If user was idle and has now returned...
        if idle_time < (settings_manager::get_int("idle-time") * 60) as u64
            && *imp.idle_time_reached.lock().unwrap()
            && !*imp.idle_notified.lock().unwrap()
        {
            *imp.idle_notified.lock().unwrap() = true;
            self.resume_from_idle();
        }
        *imp.stored_idle.lock().unwrap() = idle_time;

        // If user is idle but has not returned...
        if *imp.stored_idle.lock().unwrap() >= (settings_manager::get_int("idle-time") * 60) as u64
            && !*imp.idle_time_reached.lock().unwrap()
        {
            *imp.idle_time_reached.lock().unwrap() = true;
            let true_idle_start_time = Local::now()
                - ChronDur::seconds((settings_manager::get_int("idle-time") * 60) as i64);
            *imp.idle_start_time.lock().unwrap() = true_idle_start_time.to_rfc3339();
        }
    }

    fn resume_from_idle(&self) {
        let imp = imp::FurtheranceWindow::from_obj(self);

        let resume_time = Local::now();
        let idle_start =
            DateTime::parse_from_rfc3339(&imp.idle_start_time.lock().unwrap()).unwrap();
        let idle_start = idle_start.with_timezone(&Local);
        let idle_time = resume_time - idle_start;
        let idle_time = idle_time.num_seconds();
        let h = idle_time / 60 / 60;
        let m = (idle_time / 60) - (h * 60);
        let s = idle_time - (m * 60);
        let idle_time_str = format!(
            "{}{:02}:{:02}:{:02}",
            gettext("You have been idle for "),
            h,
            m,
            s
        );
        let question_str = gettext("\nWould you like to discard that time, or continue the clock?");
        let idle_time_msg = format!("{}{}", idle_time_str, question_str);

        let dialog = gtk::MessageDialog::with_markup(
            Some(self),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Warning,
            gtk::ButtonsType::None,
            Some(&format!(
                "<span size='x-large' weight='bold'>{}</span>",
                &gettext("Idle")
            )),
        );
        dialog.add_buttons(&[
            (&gettext("Discard"), gtk::ResponseType::Reject),
            (&gettext("Continue"), gtk::ResponseType::Accept),
        ]);
        dialog.set_secondary_text(Some(&idle_time_msg));

        dialog.connect_response(clone!(
            @weak self as this,
            @strong dialog,
            @strong imp.start_button as start_button => move |_, resp| {
            if resp == gtk::ResponseType::Reject {
                this.set_subtract_idle(true);
                start_button.emit_clicked();
                dialog.close();
            } else if resp == gtk::ResponseType::Accept {
                this.reset_idle();
                dialog.close();
            }
        }));

        *imp.idle_dialog.lock().unwrap() = dialog.clone();
        let app = FurtheranceApplication::default();
        app.system_idle_notification(&idle_time_str, &question_str);

        dialog.show();
    }

    fn pomodoro_over(&self, timer_start: DateTime<Local>, timer_stop: DateTime<Local>) {
        let dialog = gtk::MessageDialog::with_markup(
            Some(self),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Warning,
            gtk::ButtonsType::None,
            Some(&format!(
                "<span size='x-large' weight='bold'>{}</span>",
                &gettext("Time's up!")
            )),
        );
        dialog.add_buttons(&[
            (&gettext("Continue"), gtk::ResponseType::Accept),
            (&gettext("Stop"), gtk::ResponseType::Reject),
        ]);

        let app = FurtheranceApplication::default();
        app.system_pomodoro_notification(dialog.clone());
        dialog.connect_response(clone!(
            @weak self as this,
            @strong dialog => move |_, resp| {
            let imp = imp::FurtheranceWindow::from_obj(&this);
            if resp == gtk::ResponseType::Reject {
                imp.start_button.set_icon_name("media-playback-start-symbolic");
                this.refresh_timer();
                imp.task_input.set_sensitive(true);
                this.save_task(timer_start, timer_stop);
                this.reset_idle();
                dialog.close();
            } else if resp == gtk::ResponseType::Accept {
                *imp.pomodoro_continue.lock().unwrap() = true;
                this.reset_idle();
                imp.start_button.emit_clicked();
                dialog.close();
            }
        }));

        let imp2 = imp::FurtheranceWindow::from_obj(self);
        imp2.idle_dialog.lock().unwrap().close();

        dialog.show();
    }

    fn write_autosave(&self, auto_start_time: DateTime<Local>) {
        let auto_stop_time = Local::now().to_rfc3339();
        let auto_start_time = auto_start_time.to_rfc3339();
        let (task_name, tag_list) = self.split_tags_and_task();

        let path = FurtheranceWindow::get_autosave_path();
        let file = File::create(path).expect("Couldn't create autosave file");
        let mut file = BufWriter::new(file);

        writeln!(file, "{}", task_name).expect("Unable to write autosave");
        writeln!(file, "{}", auto_start_time).expect("Unable to write autosave");
        writeln!(file, "{}", auto_stop_time).expect("Unable to write autosave");
        write!(file, "{}", tag_list).expect("Unable to write autosave");
    }

    fn delete_autosave() {
        let path = FurtheranceWindow::get_autosave_path();
        if path.exists() {
            remove_file(path).expect("Could not delete autosave");
        }
    }

    fn get_autosave_path() -> PathBuf {
        let mut path = PathBuf::new();
        if let Some(proj_dirs) = ProjectDirs::from("com", "lakoliu", "Furtherance") {
            path = PathBuf::from(proj_dirs.data_dir());
            create_dir_all(path.clone()).expect("Unable to create autosave directory");
            path.extend(&["furtherance_autosave.txt"]);
        }
        path
    }

    fn split_tags_and_task(&self) -> (String, String) {
        let imp = imp::FurtheranceWindow::from_obj(self);
        let task_input_text = imp.task_input.text();
        let mut split_tags: Vec<&str> = task_input_text.trim().split("#").collect();
        // Remove task name from tags list
        let task_name = *split_tags.first().unwrap();
        split_tags.remove(0);
        // Trim whitespace around each tag
        split_tags = split_tags.iter().map(|x| x.trim()).collect();
        // Don't allow empty tags
        split_tags.retain(|&x| !x.trim().is_empty());
        // Handle duplicate tags before they are ever saved
        split_tags = split_tags.into_iter().unique().collect();
        // Lowercase tags
        let lower_tags: Vec<String> = split_tags.iter().map(|x| x.to_lowercase()).collect();
        let tag_list = lower_tags.join(" #");
        (task_name.trim().to_string(), tag_list)
    }

    fn check_for_autosave(&self) {
        let path = FurtheranceWindow::get_autosave_path();
        if path.exists() {
            let autosave = FurtheranceWindow::read_autosave().unwrap();

            database::write_autosave(&autosave[0], &autosave[1], &autosave[2], &autosave[3])
                .expect("Could not write autosave");

            let dialog = gtk::MessageDialog::new(
                Some(self),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Info,
                gtk::ButtonsType::Ok,
                &gettext("Autosave Restored"),
            );
            dialog.set_secondary_text(Some(&gettext(
                "Furtherance shut down improperly. An autosave was restored.",
            )));

            dialog.connect_response(clone!(
                @weak self as this,
                @strong dialog => move |_, resp| {
                if resp == gtk::ResponseType::Ok {
                    this.reset_history_box();
                    dialog.close();
                }
            }));

            dialog.show();
            FurtheranceWindow::delete_autosave();
        }
    }

    fn read_autosave() -> io::Result<Vec<String>> {
        let path = FurtheranceWindow::get_autosave_path();
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut vars = Vec::new();

        for line in reader.lines() {
            vars.push(line?);
        }
        // Add empty string if there are no tags
        if vars.len() == 3 {
            vars.push("".to_string());
        }

        Ok(vars)
    }

    fn create_autocomplete() -> gtk::EntryCompletion {
        let task_autocomplete = gtk::EntryCompletion::new();
        task_autocomplete.set_text_column(0);
        task_autocomplete.set_minimum_key_length(FurtheranceWindow::MIN_PREFIX_LENGTH);
        task_autocomplete.set_match_func(|_ac, _s, _it| { true });
        task_autocomplete
    }

    pub fn reset_idle(&self) {
        let imp = imp::FurtheranceWindow::from_obj(self);
        *imp.stored_idle.lock().unwrap() = 0;
        *imp.idle_notified.lock().unwrap() = false;
        *imp.idle_time_reached.lock().unwrap() = false;
        *imp.subtract_idle.lock().unwrap() = false;
    }

    pub fn set_subtract_idle(&self, val: bool) {
        let imp = imp::FurtheranceWindow::from_obj(self);
        *imp.subtract_idle.lock().unwrap() = val;
    }

    pub fn duplicate_task(&self, task: database::Task) {
        let imp = imp::FurtheranceWindow::from_obj(self);
        if !*imp.running.lock().unwrap() {
            let task_text: String;
            if task.tags.trim().is_empty() {
                task_text = task.task_name;
            } else {
                task_text = format!("{} #{}", task.task_name, task.tags);
            }
            imp.task_input.set_text(&task_text);
            imp.start_button.emit_clicked();
        } else {
            self.display_toast(&gettext("Stop the timer to duplicate a task."));
        }
    }

    pub fn refresh_timer(&self) {
        let imp = imp::FurtheranceWindow::from_obj(self);
        if settings_manager::get_bool("pomodoro") {
            let mut mins = settings_manager::get_int("pomodoro-time");
            let mut hrs: i32 = 0;
            if mins > 59 {
                hrs = mins / 60;
                mins = mins % 60;
            }
            let watch_text: &str = &format!("{:02}:{:02}:00", hrs, mins);
            imp.watch.set_text(watch_text);
        } else {
            imp.watch.set_text("00:00:00");
        }
    }

    pub async fn export_csv_to_file(
        sort: TaskSort,
        order: SortOrder,
        file: &gio::File,
    ) -> anyhow::Result<()> {
        async fn overwrite_file_future(file: &gio::File, bytes: Vec<u8>) -> anyhow::Result<()> {
            let output_stream = file
                .replace_future(
                    None,
                    false,
                    gio::FileCreateFlags::REPLACE_DESTINATION,
                    glib::source::Priority::DEFAULT,
                )
                .await?;

            output_stream
                .write_all_future(bytes, glib::source::Priority::DEFAULT)
                .await
                .map_err(|e| anyhow::anyhow!(e.1))?;
            output_stream.close_future(glib::source::Priority::DEFAULT).await?;

            Ok(())
        }

        let csv = database::export_as_csv(sort, order, b',')?;
        overwrite_file_future(file, csv.into_bytes()).await
    }

    pub fn open_csv_export_dialog(&self) {
        let builder = gtk::Builder::from_resource("/com/lakoliu/Furtherance/gtk/dialogs.ui");
        let dialog = builder.object::<gtk::Dialog>("dialog_csv_export").unwrap();
        let tasksort_row = builder
            .object::<adw::ComboRow>("csv_export_tasksort_row")
            .unwrap();
        let sortorder_row = builder
            .object::<adw::ComboRow>("csv_export_sortorder_row")
            .unwrap();
        let filechooser_button = builder
            .object::<gtk::Button>("csv_export_filechooser_button")
            .unwrap();
        let chosenfile_label = builder
            .object::<gtk::Label>("csv_export_chosenfile_label")
            .unwrap();

        dialog.set_transient_for(Some(self));

        let filefilter = gtk::FileFilter::new();
        filefilter.add_mime_type("text/csv");
        filefilter.add_pattern("*.csv");

        let filechooser = gtk::FileChooserNative::builder()
            .title(&gettext("Create or choose a CSV file"))
            .modal(true)
            .transient_for(self)
            .action(gtk::FileChooserAction::Save)
            .accept_label(&gettext("Accept"))
            .cancel_label(&gettext("Cancel"))
            .select_multiple(false)
            .filter(&filefilter)
            .build();

        filechooser.set_current_name("data.csv");

        filechooser_button.connect_clicked(
            clone!(@weak self as window, @weak filechooser, @weak dialog => move |_| {
                dialog.hide();
                filechooser.show();
            }),
        );

        filechooser.connect_response(
            clone!(@weak dialog, @weak chosenfile_label => move |filechooser, response| {
                if response == gtk::ResponseType::Accept {
                    if let Some(path) = filechooser.file().and_then(|file| file.path()) {
                        chosenfile_label.set_label(&path.to_string_lossy());
                    } else {
                        chosenfile_label.set_label(&gettext(" - no file selected - "));
                    }
                }

                dialog.show();
            }),
        );

        dialog.connect_response(clone!(@weak self as window, @weak filechooser, @weak tasksort_row, @weak sortorder_row => move |dialog, response| {
            match response {
                gtk::ResponseType::Apply => {
                    let sort = TaskSort::try_from(tasksort_row.selected()).unwrap_or_default();
                    let order = SortOrder::try_from(sortorder_row.selected()).unwrap_or_default();

                    if let Some(file) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong window, @strong file => async move {
                            if let Err(e) = FurtheranceWindow::export_csv_to_file(sort, order, &file).await {
                                log::error!("replace file {:?} failed, Err {}", file, e);
                                window.display_toast(&gettext("Exporting as CSV failed."));
                            } else {
                                window.display_toast(&gettext("Exported as CSV successfully."));
                            };
                        }));
                    }
                }
                _ => {}
            }

            dialog.close();
        }));

        *self.imp().filechooser.borrow_mut() = filechooser;

        dialog.show()
    }

    pub fn vertical_align(&self, align: gtk::Align) {
        let imp = imp::FurtheranceWindow::from_obj(self);
        imp.win_box.set_valign(align);
    }
}

impl Default for FurtheranceWindow {
    fn default() -> Self {
        FurtheranceApplication::default()
            .active_window()
            .unwrap()
            .downcast()
            .unwrap()
    }
}

