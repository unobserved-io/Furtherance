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
use glib::clone;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*, CompositeTemplate};
use chrono::{DateTime, NaiveDateTime, Local, offset::TimeZone};

use crate::FurtheranceApplication;
use crate::ui::FurtheranceWindow;
use crate::database;

mod imp {
    use super::*;
    use glib::subclass;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/lakoliu/Furtherance/gtk/task_details.ui")]
    pub struct FurTaskDetails {
        #[template_child]
        pub headerbar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub dialog_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub task_name_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub main_box: TemplateChild<gtk::Box>,

        #[template_child]
        pub delete_all_btn: TemplateChild<gtk::Button>,

        pub all_boxes: RefCell<Vec<gtk::Box>>,
        pub all_task_ids: RefCell<Vec<i32>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FurTaskDetails {
        const NAME: &'static str = "FurTaskDetails";
        type ParentType = adw::Window;
        type Type = super::FurTaskDetails;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FurTaskDetails {

        fn constructed(&self, obj: &Self::Type) {
            obj.setup_signals();
            obj.setup_delete_all();
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for FurTaskDetails {}

    impl WindowImpl for FurTaskDetails {}

    impl AdwWindowImpl for FurTaskDetails {}
}

glib::wrapper! {
    pub struct FurTaskDetails(ObjectSubclass<imp::FurTaskDetails>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl FurTaskDetails {
    pub fn new() -> Self {
        let dialog: Self = glib::Object::new(&[]).unwrap();

        let window = FurtheranceWindow::default();
        dialog.set_transient_for(Some(&window));

        let app = FurtheranceApplication::default();
        app.add_window(&window);

        dialog
    }

    pub fn setup_widgets(&self, mut task_group: Vec<database::Task>) {
        let imp = imp::FurTaskDetails::from_instance(self);

        imp.task_name_label.set_text(&task_group[0].task_name);

        for task in task_group.clone() {
            imp.all_task_ids.borrow_mut().push(task.id);
        }

        let task_group_len = task_group.len();
        task_group.reverse();
        for task in task_group {
            let task_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
            task_box.set_homogeneous(true);

            let start_time = DateTime::parse_from_rfc3339(&task.start_time).unwrap();
            let start_time_str = start_time.format("%H:%M:%S").to_string();
            let start = gtk::Button::new();
            start.set_label(&start_time_str);
            task_box.append(&start);

            let stop_time = DateTime::parse_from_rfc3339(&task.stop_time).unwrap();
            let stop_time_str = stop_time.format("%H:%M:%S").to_string();
            let stop = gtk::Button::new();
            stop.set_label(&stop_time_str);
            task_box.append(&stop);

            let total_time = stop_time - start_time;
            let total_time = total_time.num_seconds();
            let h = total_time / 60 / 60;
            let m = (total_time / 60) - (h * 60);
            let s = total_time - (m * 60);
            let total_time_str = format!("{:02}:{:02}:{:02}", h, m, s);
            let total = gtk::Button::new();
            total.set_label(&total_time_str);
            total.add_css_class("inactive-button");
            total.set_hexpand(false);
            task_box.append(&total);

            imp.main_box.append(&task_box);
            imp.all_boxes.borrow_mut().push(task_box);

            start.connect_clicked(clone!(@weak self as this => move |_|{
                let window = FurtheranceWindow::default();
                let dialog = gtk::MessageDialog::new(
                    Some(&window),
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Question,
                    gtk::ButtonsType::OkCancel,
                    "<span size='x-large' weight='bold'>Edit Task</span>",
                );
                dialog.set_use_markup(true);

                let message_area = dialog.message_area().downcast::<gtk::Box>().unwrap();
                let vert_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
                let task_name_edit = gtk::Entry::new();
                task_name_edit.set_text(&task.task_name);
                let labels_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                labels_box.set_homogeneous(true);
                let start_label = gtk::Label::new(Some("Start"));
                start_label.add_css_class("title-4");
                let stop_label = gtk::Label::new(Some("Stop"));
                stop_label.add_css_class("title-4");
                let times_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                times_box.set_homogeneous(true);

                let start_time_w_year = start_time.format("%h %e %Y %H:%M:%S").to_string();
                let stop_time_w_year = stop_time.format("%h %e %Y %H:%M:%S").to_string();
                let start_time_edit = gtk::Entry::new();
                start_time_edit.set_text(&start_time_w_year);
                let stop_time_edit = gtk::Entry::new();
                stop_time_edit.set_text(&stop_time_w_year);

                let instructions = gtk::Label::new(Some(
                    "*Use the format MMM DD YYYY HH:MM:SS"));
                instructions.set_visible(false);
                instructions.add_css_class("error_message");

                let time_error = gtk::Label::new(Some(
                    "*Start time cannot be later than stop time."));
                time_error.set_visible(false);
                time_error.add_css_class("error_message");

                let future_error = gtk::Label::new(Some(
                    "*Time cannot be in the future."));
                future_error.set_visible(false);
                future_error.add_css_class("error_message");

                let delete_task_btn = gtk::Button::new();
                delete_task_btn.set_icon_name("user-trash-symbolic");
                delete_task_btn.set_tooltip_text(Some("Delete task"));
                delete_task_btn.set_hexpand(false);
                delete_task_btn.set_vexpand(false);
                delete_task_btn.set_halign(gtk::Align::End);

                vert_box.append(&task_name_edit);
                labels_box.append(&start_label);
                labels_box.append(&stop_label);
                times_box.append(&start_time_edit);
                times_box.append(&stop_time_edit);
                vert_box.append(&labels_box);
                vert_box.append(&times_box);
                vert_box.append(&instructions);
                vert_box.append(&time_error);
                vert_box.append(&future_error);
                message_area.append(&delete_task_btn);
                message_area.append(&vert_box);

                delete_task_btn.connect_clicked(clone!(
                    @strong task, @strong dialog, @weak this => move |_| {

                    let delete_confirmation = gtk::MessageDialog::with_markup(
                        Some(&window),
                        gtk::DialogFlags::MODAL,
                        gtk::MessageType::Question,
                        gtk::ButtonsType::OkCancel,
                        Some("<span size='x-large' weight='bold'>Delete task?</span>"),
                    );

                    delete_confirmation.connect_response(clone!(
                        @strong dialog,
                        @strong delete_confirmation => move |_, resp| {
                        if resp == gtk::ResponseType::Ok {
                            let _ = database::delete_by_id(task.id);
                            if task_group_len == 1 {
                                delete_confirmation.close();
                                dialog.close();
                                this.close();
                                let window = FurtheranceWindow::default();
                                window.reset_history_box();
                            } else {
                                delete_confirmation.close();
                                this.clear_task_list();
                                dialog.close();
                            }
                        } else {
                            delete_confirmation.close();
                        }
                    }));

                    delete_confirmation.show();
                }));


                dialog.connect_response(
                    clone!(@strong dialog,
                        @strong task.task_name as name
                        @strong task.start_time as start_time
                        @strong task.stop_time as stop_time => move |_ , resp| {
                        if resp == gtk::ResponseType::Ok {
                            instructions.set_visible(false);
                            time_error.set_visible(false);
                            future_error.set_visible(false);
                            let mut start_successful = false;
                            let mut stop_successful = false;
                            let mut do_not_close = false;
                            let mut new_start_time_edited: String = "".to_string();
                            let mut new_start_time_local = Local::now();
                            let new_stop_time_edited: String;
                            if start_time_edit.text() != start_time_w_year {
                                let new_start_time_str = start_time_edit.text();
                                let new_start_time = NaiveDateTime::parse_from_str(
                                                        &new_start_time_str,
                                                        "%h %e %Y %H:%M:%S");
                                if let Err(_) = new_start_time {
                                    instructions.set_visible(true);
                                    do_not_close = true;
                                } else {
                                    new_start_time_local = Local.from_local_datetime(&new_start_time.unwrap()).unwrap();
                                    new_start_time_edited = new_start_time_local.to_rfc3339();
                                    start_successful = true;
                                }
                            }
                            if stop_time_edit.text() != stop_time_w_year {
                                let new_stop_time_str = stop_time_edit.text();
                                let new_stop_time = NaiveDateTime::parse_from_str(
                                                        &new_stop_time_str,
                                                        "%h %e %Y %H:%M:%S");
                                if let Err(_) = new_stop_time {
                                    instructions.set_visible(true);
                                    do_not_close = true;
                                } else {
                                    let new_stop_time = Local.from_local_datetime(&new_stop_time.unwrap()).unwrap();
                                    new_stop_time_edited = new_stop_time.to_rfc3339();
                                    if start_successful {
                                        if (new_stop_time - new_start_time_local).num_seconds() >= 0 {
                                            database::update_stop_time(task.id, new_stop_time_edited.clone())
                                                .expect("Failed to update stop time.");
                                            database::update_start_time(task.id, new_start_time_edited.clone())
                                                .expect("Failed to update stop time.");
                                        }
                                    } else {
                                        let old_start_time = DateTime::parse_from_rfc3339(&start_time);
                                        let old_start_time = old_start_time.unwrap().with_timezone(&Local);
                                        if (Local::now() - new_stop_time).num_seconds() < 0 {
                                            future_error.set_visible(true);
                                            do_not_close = true;
                                        } else if (new_stop_time - old_start_time).num_seconds() >= 0 {
                                            database::update_stop_time(task.id, new_stop_time_edited)
                                                .expect("Failed to update stop time.");
                                        } else {
                                            time_error.set_visible(true);
                                            do_not_close = true;
                                        }
                                    }
                                    stop_successful = true;
                                }
                            }
                            if task_name_edit.text() != name {
                                database::update_task_name(task.id, task_name_edit.text().to_string())
                                    .expect("Failed to update start time.");
                            }

                            if start_successful && !stop_successful {
                                let old_stop_time = DateTime::parse_from_rfc3339(&stop_time);
                                let old_stop_time = old_stop_time.unwrap().with_timezone(&Local);
                                if (old_stop_time - new_start_time_local).num_seconds() >= 0 {
                                    database::update_start_time(task.id, new_start_time_edited)
                                        .expect("Failed to update start time.");
                                } else {
                                    time_error.set_visible(true);
                                    do_not_close = true;
                                }

                            }

                            if !do_not_close {
                                this.clear_task_list();
                                dialog.close();
                            }


                        } else {
                            // If Cancel, close dialog and do nothing.
                            dialog.close();
                        }
                    }),
                );


                dialog.show();
            }));

            stop.connect_clicked(move |_|{
                start.emit_clicked();
            });
        }
    }

    fn clear_task_list(&self) {
        let imp = imp::FurTaskDetails::from_instance(&self);

        for task_box in &*imp.all_boxes.borrow() {
            imp.main_box.remove(task_box);
        }

        imp.all_boxes.borrow_mut().clear();
        // Get list from database by a vec of IDs
        let updated_list = database::get_list_by_id(imp.all_task_ids.clone().borrow().to_vec());
        imp.all_task_ids.borrow_mut().clear();
        let window = FurtheranceWindow::default();
        window.reset_history_box();
        self.setup_widgets(updated_list.unwrap());
    }

    fn setup_signals(&self) {
        let imp = imp::FurTaskDetails::from_instance(self);

        // Add headerbar to dialog when scrolled far
        imp.scrolled_window.vadjustment().connect_value_notify(
            clone!(@weak self as this => move |adj|{
                let imp = imp::FurTaskDetails::from_instance(&this);
                if adj.value() < 120.0 {
                    imp.headerbar.add_css_class("hidden");
                    imp.dialog_title.set_visible(false);
                }else {
                    imp.headerbar.remove_css_class("hidden");
                    imp.dialog_title.set_visible(true);
                }
            }),
        );

        // Make dialog header smaller if the name is long
        imp.task_name_label.connect_label_notify(|label| {
            let large_title = !(label.text().len() > 25);

            if large_title {
                label.remove_css_class("title-2");
                label.add_css_class("title-1");
            } else {
                label.remove_css_class("title-1");
                label.add_css_class("title-2");
            }
        });
    }

    fn setup_delete_all(&self) {
        let imp = imp::FurTaskDetails::from_instance(self);
        let window = FurtheranceWindow::default();

        imp.delete_all_btn.connect_clicked(clone!(@weak self as this => move |_|{
            let dialog = gtk::MessageDialog::with_markup(
                Some(&window),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Warning,
                gtk::ButtonsType::None,
                Some("<span size='large'>Delete All?</span>"),
            );
            dialog.set_secondary_text(Some("This will delete all occurrences of this task on this day."));
            dialog.add_buttons(&[
                ("Delete", gtk::ResponseType::Accept),
                ("Cancel", gtk::ResponseType::Reject)
            ]);
            dialog.show();

            dialog.connect_response(clone!(@strong dialog => move |_,resp|{
                if resp == gtk::ResponseType::Accept {
                    this.delete_all();
                    dialog.close();
                    this.close();
                    let window = FurtheranceWindow::default();
                    window.reset_history_box();
                } else {
                    dialog.close();
                }
            }));

        }));

    }

    fn delete_all(&self) {
        let imp = imp::FurTaskDetails::from_instance(self);
        let _ = database::delete_by_ids(imp.all_task_ids.borrow().to_vec());
    }

}

