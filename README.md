# Furtherance
Furtherance is a time tracking app, with a native interface on Gnome, Windows, and Mac.
It allows you to track time spent on different activities without worrying about your data being captured and sold.

<p align="center">
    <img width="500px" src="https://github.com/lakoliu/Furtherance/raw/main/data/screenshots/furtherance-2023-06-06.png" alt="Furtherance screenshot"/>
</p>

## Features
* Tasks are saved in a database with an unlimited capacity. It can be backed up for security and portability.
* Tasks are grouped by name and date.
* Tasks can be edited after they are created (name, time, and date).
* Settings to customize the view and defaults to your liking.
* Features can be added! Just open an issue.

## Getting Started

### Install
**Linux**

<a href="https://flathub.org/apps/details/com.lakoliu.Furtherance"><img width='240' alt="Download on Flathub" src="https://flathub.org/assets/badges/flathub-badge-en.png"/></a>
* The easiest way to install Furtherance is from [Flathub](https://flathub.org/apps/details/com.lakoliu.Furtherance). Make sure you have Flatpak [setup](https://flatpak.org/setup/).
* Furtherance is also available on Arch in the AUR, btw ([stable](https://aur.archlinux.org/packages/furtherance) or [nightly](https://aur.archlinux.org/packages/furtherance-git))

**Mac**

<a href="https://apps.apple.com/app/furtherance/id1659277200"><img width='240' alt="Download on the Mac App Store" src="https://furtherance.app/images/app-store-dark.svg"/></a>
* Furtherance for Mac is availaible from the [Mac App Store](https://apps.apple.com/app/furtherance/id1659277200). It uses a different codebase (Swift & SwiftUI) to provide a native experience on Mac, and it is not open-source.

**Windows**

<a href="https://apps.microsoft.com/store/detail/furtherance/9NHG98S3VR3W"><img width='240' alt="Download from Microsoft Store" src="https://furtherance.app/images/microsoft-store-dark.svg"/></a>
* Furtherance for Windows is availaible from the [Microsoft Store](https://www.microsoft.com/store/apps/9NHG98S3VR3W). It uses a different codebase to provide a better experience on Windows, and it is not open-source.

**Android**

<a href='https://play.google.com/store/apps/details?id=com.livaliva.furtherance&pcampaignid=pcampaignidMKT-Other-global-all-co-prtnr-py-PartBadge-Mar2515-1'><img  width='280' alt='Get it on Google Play' src='https://play.google.com/intl/en_us/badges/static/images/badges/en_badge_web_generic.png'/></a>
* Furtherance for Android is available on [Google Play](https://play.google.com/store/apps/details?id=com.livaliva.furtherance). It uses a different codebase to provide a better experience on mobile, and it is not open-source.

### Build
The easiest way to build Furtherance is with [GNOME Builder](https://flathub.org/apps/details/org.gnome.Builder).

To build Furtherance on your own, make sure you have all the dependencies: *rust, cargo, meson, ninja-build, sqlite3, dbus-1, glib-2.0, gtk4, libadwaita-1*

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

### Use
Type in the name of the task you are working on, add some #tags, and press start. That's really all there is to it.

## Contribute

### Translations
If you speak another language, it would be greatly appreciated if you could help translate Furtherance to make it available to more people!
You can get started easily using [Weblate](https://hosted.weblate.org/projects/furtherance/translations/).

### Tips
Besides helping to pay the bills, tips make me feel all warm and fuzzy inside. If you've gotten value from Furtherance, you can tip me via: 
* [Ko-fi](https://ko-fi.com/unobserved) 
* [PayPal](https://www.paypal.com/donate/?hosted_button_id=TLYY8YZ424VRL)
* **Bitcoin**: bc1q70czd5evhsxnjcd45cj2n4s3dr6qmhvrlljjlk

Thank you so much!

## Project Details

### Built With
* Linux: Written in Rust using the Gtk-rs bindings for GTK 4.
* Windows: Written in C# using WinUI 3
* Mac: Written in Swift using SwifUI
* Android: Written in Dart using Flutter

### License
This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details. (This only applies to the Linux version, the source code you see in this repository)

### Author
This project is created and maintained by [Ricky Kresslein](https://kressle.in) under [Unobserved](https://unobserved.io). More information at [Furtherance.app](https://furtherance.app).
