// Furtherance - Track your time without being tracked
// Copyright (C) 2024  Ricky Kresslein <rk@unobserved.io>
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

use std::{borrow::Cow, collections::HashMap};

use fluent::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use rust_embed::RustEmbed;
use sys_locale::get_locale;

// Embed the files in the app package
#[derive(RustEmbed)]
#[folder = "src/locales/"]
struct Locales;

fn load_fluent_resource(lang: &str) -> FluentResource {
    let file_path = format!("{}/main.ftl", lang);
    let source = Locales::get(&file_path)
        .expect("Failed to load embedded file")
        .data;
    let source_str = std::str::from_utf8(&source).expect("Failed to convert to UTF-8");
    FluentResource::try_new(source_str.to_string()).expect("Failed to parse an FTL string")
}

fn create_bundle(lang: &str) -> FluentBundle<FluentResource> {
    let mut bundle = FluentBundle::new(vec![lang.parse().expect("Failed to parse language tag")]);
    let resource = load_fluent_resource(lang);
    bundle
        .add_resource(resource)
        .expect("Failed to add FTL resources to the bundle");
    bundle
}

pub struct Localization {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    current_lang: String,
}

impl Localization {
    pub fn new() -> Self {
        let mut bundles = HashMap::new();
        bundles.insert("de".to_string(), create_bundle("de"));
        bundles.insert("en-US".to_string(), create_bundle("en-US"));
        bundles.insert("es".to_string(), create_bundle("es"));
        bundles.insert("fi".to_string(), create_bundle("fi"));
        bundles.insert("fr".to_string(), create_bundle("fr"));
        bundles.insert("it".to_string(), create_bundle("it"));
        bundles.insert("nl".to_string(), create_bundle("nl"));
        bundles.insert("pt-BR".to_string(), create_bundle("pt-BR"));
        bundles.insert("pt-PT".to_string(), create_bundle("pt-PT"));
        bundles.insert("ru".to_string(), create_bundle("ru"));
        bundles.insert("sk".to_string(), create_bundle("sk"));
        bundles.insert("tr".to_string(), create_bundle("tr"));

        let mut current_lang = get_locale().unwrap_or_else(|| String::from("en-US"));
        if !bundles.contains_key(&current_lang) {
            let truncated_lang = current_lang.chars().take(2).collect::<String>();
            if !bundles.contains_key(&truncated_lang) {
                current_lang = "en-US".to_string();
            } else {
                current_lang = truncated_lang;
            }
        }

        Localization {
            bundles,
            current_lang,
        }
    }

    pub fn get_message(&self, key: &str, args: Option<&HashMap<&str, String>>) -> String {
        let bundle = self.bundles.get(&self.current_lang).unwrap();
        let msg = bundle.get_message(key).expect("Message doesn't exist");
        let pattern = msg.value().expect("Message has no value");

        let mut errors = vec![];
        let formatted = if let Some(arg_map) = args {
            let mut fluent_args = FluentArgs::new();
            for (k, v) in arg_map {
                let cow_str: Cow<str> = Cow::Borrowed(v.as_str());
                fluent_args.set(Cow::Borrowed(*k), FluentValue::from(cow_str));
            }

            bundle.format_pattern(pattern, Some(&fluent_args), &mut errors)
        } else {
            bundle.format_pattern(pattern, None, &mut errors)
        };

        if !errors.is_empty() {
            eprintln!("Errors occurred during formatting: {:?}", errors);
        }

        // Prevent odd symbols in iced
        // TODO: Try to remove when iced has rtl text formatting
        formatted
            .to_string()
            .replace('\u{2068}', "")
            .replace('\u{2069}', "")
    }

    // fn set_language(&mut self, lang: &str) {
    //     if self.bundles.contains_key(lang) {
    //         self.current_lang = lang.to_string();
    //     } else {
    //         eprintln!("Language not available: {}", lang);
    //     }
    // }
}
