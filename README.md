# Furtherance
Furtherance is a time tracking app.
It allows you to track time spent on different activities without worrying about your data being captured and sold.

**UPDATE: Furtherance has undergone a complete rewrite using [Iced](https://github.com/iced-rs/iced). It is better in many ways and contains more features. Most importantly, there is now a single, cross-platform version,
so updates will be far more frequent and everyone will have the same features, regardless of OS. You can read more about the change [here](https://unobserved.io/blog/twau-1).**

<p align="center">
    <img width="750px" src="https://unobserved.io/assets/screenshots/furtherance/mac/Timer.png" alt="Furtherance timer"/>
</p>

## Features
* Track your time spent on tasks with an associated project, rate, and tags.
* Pomodoro timer with breaks and periodic larger breaks.
* Cross-platform! Use it on Linux, Mac, and Windows.
* Tasks can be edited after they are created.
* Settings to customize the view and defaults to your liking.
* Features can be added! Just open an issue.

## Getting Started

### Install
_**Furtherance has been re-written and uses a new database structure. Back up your old database (to either .db or .csv) before converting the database with the new app**_

**Linux**

* Install from [Flathub](https://flathub.org/apps/io.unobserved.furtherance): `flatpak install flathub io.unobserved.furtherance`
* Download the .deb from the latest release

**Mac**

* Use [Homebrew](https://formulae.brew.sh/cask/furtherance): `brew install --cask furtherance`
* Download the .dmg from the lastest release

**Windows**

* Install from the [Microsoft Store](https://apps.microsoft.com/detail/9nhg98s3vr3w)
* Use the .msi installer provided in the latest release.

### Use
Type in the `name` of the task you are working on, add a `@Project`, some `#tags`, and a `$rate`, and press start. That's really all there is to it.

## Contribute

### Translations
If you speak another language, it would be greatly appreciated if you could help translate Furtherance to make it available to more people! You can edit the current translations in the `src/locales` directory, or create a new translation there and submit a pull request.

### Tips
Besides helping to pay the bills, tips show me people want me to continue spending time on Furtherance. I truly appreciate these, and I am humbled by what I've received so far. If you've gotten value from Furtherance, you can tip me via:
* [PayPal](https://www.paypal.com/donate/?hosted_button_id=TLYY8YZ424VRL)
* [GitHub](https://github.com/sponsors/rickykresslein)
* [Patreon](https://www.patreon.com/unobserved)
* [Ko-fi](https://ko-fi.com/unobserved)
* Lightning: unobserved [at] getalby [dot com]

Thank you so much!

## Project Details

### Built With
Rust & [Iced](https://github.com/iced-rs/iced)

### License
This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.

### Author
This project is created and maintained by [Ricky Kresslein](https://kressle.in) under [Unobserved](https://unobserved.io). More information at [Furtherance.app](https://furtherance.app).
