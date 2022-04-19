# Furtherance
Furtherance is a time tracking app written in Rust with GTK 4.
It allows you to track time spent on different activities without worrying about your data being captured and sold.

<p float="left">
    <img src="https://github.com/lakoliu/Furtherance/raw/main/data/screenshots/furtherance-screenshot-main.png" alt="Furtherance main window" width="400"/>
    <img src="https://github.com/lakoliu/Furtherance/raw/main/data/screenshots/furtherance-screenshot-task-details.png" alt="Furtherance task details" width="400"/>
    <img src="https://github.com/lakoliu/Furtherance/raw/main/data/screenshots/furtherance-screenshot-edit-task.png" alt="Furtherance edit task" width="400"/>
    <img src="https://github.com/lakoliu/Furtherance/raw/main/data/screenshots/furtherance-screenshot-settings.png" alt="Furtherance settings" width="400"/>
</p>

## Features
* Tasks are saved in a database with an unlimited capacity. It can be backed up for security and portability.
* Tasks are grouped by name and date.
* Tasks can be edited after they are created (name, time, and date).
* Settings to customize the view and defaults to your liking.
* Features can be added! Just open an issue.

## Getting Started
<a href="https://flathub.org/apps/details/com.lakoliu.Furtherance"><img width='240' alt='Download on Flathub' src="https://flathub.org/assets/badges/flathub-badge-en.png"/></a>
* The easiest way to install Furtherance is from [Flathub](https://flathub.org/apps/details/com.lakoliu.Furtherance). Make sure you have Flatpak [setup](https://flatpak.org/setup/).
* Furtherance is also available on Arch in the AUR, btw ([stable](https://aur.archlinux.org/packages/furtherance) or [nightly](https://aur.archlinux.org/packages/furtherance-git))

## Build
The easiest way to build Furtherance is with [GNOME Builder](https://flathub.org/apps/details/org.gnome.Builder).

To build Furtherance on your own, make sure you have all the dependencies: rust, cargo, meson, ninja-build, sqlite3, dbus-1, glib-2.0, gtk4, libadwaita-1
Then do:
```
git clone https://github.com/lakoliu/Furtherance.git
cd Furtherance
mkdir build
cd build
meson ..
ninja
sudo ninja install
```
To uninstall, run `sudo ninja uninstall` in the same directory.

## Use
Type in the name of the task you are working on and press start. That's really all there is to it.

## Project Details

### Built With
Furtherance is written in Rust using the Gtk-rs bindings for GTK 4.

### License
This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.

### Author
This project is created and maintained by [Ricky Kresslein](https://kressle.in) under [lakoliu](https://lakoliu.com). More information at [Furtherance.app](https://furtherance.app).

### Give
Besides helping to pay the bills, donations make me feel all warm and fuzzy inside. If you want to help out, you can use the links to the right under "Sponsor this project". Thank you so much!
