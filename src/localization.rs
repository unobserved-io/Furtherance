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

use std::{borrow::Cow, collections::HashMap, fs};

use fluent::{FluentArgs, FluentBundle, FluentResource};

fn load_fluent_resource(path: &str) -> FluentResource {
    let source = fs::read_to_string(path).expect("Failed to read the file");
    FluentResource::try_new(source).expect("Failed to parse an FTL string")
}

fn create_bundle(lang: &str) -> FluentBundle<FluentResource> {
    let mut bundle = FluentBundle::new(vec![lang.parse().expect("Failed to parse language tag")]);
    let resource = load_fluent_resource(&format!("src/locales/{}/main.ftl", lang));
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
        bundles.insert("en-US".to_string(), create_bundle("en-US"));
        // bundles.insert("es".to_string(), create_bundle("es"));

        Localization {
            bundles,
            current_lang: "en-US".to_string(),
        }
    }

    pub fn get_message(&self, key: &str, args: Option<&HashMap<&str, &str>>) -> String {
        let bundle = self.bundles.get(&self.current_lang).unwrap();
        let msg = bundle.get_message(key).expect("Message doesn't exist");
        let pattern = msg.value().expect("Message has no value");

        let mut errors = vec![];
        let formatted = if let Some(arg_map) = args {
            let mut fluent_args = FluentArgs::new();
            for (k, v) in arg_map {
                fluent_args.set(Cow::Borrowed(*k), Cow::Borrowed(*v));
            }
            bundle.format_pattern(pattern, Some(&fluent_args), &mut errors)
        } else {
            bundle.format_pattern(pattern, None, &mut errors)
        };

        if !errors.is_empty() {
            println!("Errors occurred during formatting: {:?}", errors);
        }

        formatted.to_string()
    }

    fn set_language(&mut self, lang: &str) {
        if self.bundles.contains_key(lang) {
            self.current_lang = lang.to_string();
        } else {
            println!("Language not available: {}", lang);
        }
    }
}
