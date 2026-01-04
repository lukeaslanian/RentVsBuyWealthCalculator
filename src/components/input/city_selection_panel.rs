use crate::models::{CityPreset, InvestmentParameters, PropertyData, RentalData};
use dioxus::prelude::*;

#[component]
pub fn CitySelectionPanel(
    property_data: Signal<PropertyData>,
    rental_data: Signal<RentalData>,
    investment_params: Signal<InvestmentParameters>,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let mut selected_city = use_signal(|| "Washington DC".to_string());
    let mut selected_bedrooms = use_signal(|| "2BR".to_string());

    let cities = vec!["Washington DC", "Boston", "New York City", "San Francisco"];
    let bedroom_options = vec!["Studio", "1BR", "2BR", "3BR"];

    let load_preset = move |_| {
        // Find the matching preset based on city and bedroom selection
        let presets = CityPreset::all_presets();
        let city = selected_city();
        let bedrooms = selected_bedrooms();

        // Map display bedroom values to actual bedroom_type values
        let bedroom_type = match bedrooms.as_str() {
            "Studio" => "Studio",
            "1BR" => "1",
            "2BR" => "2",
            "3BR" => "3",
            _ => "Studio",
        };

        if let Some(preset) = presets
            .iter()
            .find(|p| p.city_name == city && p.bedroom_type == bedroom_type)
        {
            property_data.set(preset.to_property_data());
            rental_data.set(preset.to_rental_data());
            investment_params.set(preset.to_investment_parameters());
        }
    };

    rsx! {
        div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow p-6" } else { "bg-monokaiLight-bgLight rounded-lg shadow p-6" },
            h2 { class: if dark_mode { "text-xl font-semibold text-monokai-fg mb-4" } else { "text-xl font-semibold text-monokaiLight-fg mb-4" },
                "City Presets"
            }
            p { class: if dark_mode { "text-sm text-monokai-fgMuted mb-4" } else { "text-sm text-monokaiLight-fgMuted mb-4" },
                "Load pre-configured values for major cities"
            }

            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 items-end",
                // City dropdown
                div { class: "flex-1",
                    label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-2" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-2" },
                        "City"
                    }
                    select {
                        class: if dark_mode {
                            "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                        } else {
                            "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                        },
                        value: "{selected_city()}",
                        onchange: move |evt| {
                            selected_city.set(evt.value());
                        },
                        for city in cities.iter() {
                            option {
                                value: "{city}",
                                selected: *city == selected_city(),
                                "{city}"
                            }
                        }
                    }
                }

                // Bedroom dropdown
                div { class: "flex-1",
                    label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-2" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-2" },
                        "Size"
                    }
                    select {
                        class: if dark_mode {
                            "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                        } else {
                            "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                        },
                        value: "{selected_bedrooms()}",
                        onchange: move |evt| {
                            selected_bedrooms.set(evt.value());
                        },
                        for bedroom in bedroom_options.iter() {
                            option {
                                value: "{bedroom}",
                                selected: *bedroom == selected_bedrooms(),
                                "{bedroom}"
                            }
                        }
                    }
                }

                // Load button
                button {
                    class: if dark_mode {
                        "px-6 py-2 bg-monokai-green text-monokai-bg font-medium rounded-md hover:opacity-90 shadow-sm transition"
                    } else {
                        "px-6 py-2 bg-monokaiLight-green text-white font-medium rounded-md hover:opacity-90 shadow-sm transition"
                    },
                    onclick: load_preset,
                    "Load Preset"
                }
            }

            // Preview of what will be loaded
            div { class: if dark_mode { "mt-4 p-3 bg-monokai-bgLighter border border-monokai-border rounded-md" } else { "mt-4 p-3 bg-monokaiLight-bgDark border border-monokaiLight-border rounded-md" },
                p { class: if dark_mode { "text-sm text-monokai-fgMuted" } else { "text-sm text-monokaiLight-fgMuted" },
                    span { class: "font-semibold", "Selected: " }
                    "{selected_city()} - {selected_bedrooms()}"
                }
            }
        }
    }
}
