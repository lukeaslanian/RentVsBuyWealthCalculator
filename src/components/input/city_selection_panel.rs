use crate::models::{CityPreset, InvestmentParameters, PropertyData, RentalData};
use dioxus::prelude::*;

/// Helper function to compare floating point values with epsilon
fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

/// Check if current data matches the selected preset
fn matches_preset(
    property_data: &PropertyData,
    rental_data: &RentalData,
    investment_params: &InvestmentParameters,
    preset: &CityPreset,
) -> bool {
    const EPSILON: f64 = 0.01;

    // Compare PropertyData fields that come from the preset
    approx_eq(property_data.home_price, preset.home_price, EPSILON)
        && approx_eq(property_data.down_payment_percent, preset.down_payment_percent, EPSILON)
        && approx_eq(property_data.interest_rate, preset.interest_rate, EPSILON)
        && approx_eq(property_data.property_tax_rate, preset.property_tax_rate, EPSILON)
        && approx_eq(property_data.home_insurance_annual, preset.home_insurance_annual, EPSILON)
        && approx_eq(property_data.maintenance_percent, preset.maintenance_percent, EPSILON)
        && approx_eq(property_data.hoa_fee, preset.hoa_fee, EPSILON)
        && approx_eq(property_data.non_included_utilities, preset.non_included_utilities, EPSILON)
        && approx_eq(property_data.lender_grant, preset.lender_grant, EPSILON)
        && approx_eq(property_data.closing_costs_percent_purchase, preset.closing_costs_percent_purchase, EPSILON)
        && approx_eq(property_data.seller_closing_assistance, preset.seller_closing_assistance, EPSILON)
        && approx_eq(property_data.closing_costs_percent_sale, preset.closing_costs_percent_sale, EPSILON)
        && approx_eq(property_data.home_appreciation_rate, preset.home_appreciation_rate, EPSILON)
        // Compare RentalData fields that come from the preset
        && approx_eq(rental_data.monthly_rent, preset.monthly_rent, EPSILON)
        && approx_eq(rental_data.amenity_fees, preset.amenity_fees, EPSILON)
        && approx_eq(rental_data.rent_included_utilities, preset.rent_included_utilities, EPSILON)
        && approx_eq(rental_data.rent_non_included_utilities, preset.rent_non_included_utilities, EPSILON)
        && approx_eq(rental_data.renters_insurance, preset.renters_insurance, EPSILON)
        && approx_eq(rental_data.rent_increase_rate, preset.rent_increase_rate, EPSILON)
        // Compare InvestmentParameters fields
        && approx_eq(investment_params.annual_return_rate, preset.annual_return_rate, EPSILON)
        && investment_params.analysis_years == 30  // Presets hardcode this to 30
        && approx_eq(investment_params.inflation_rate, 3.0, EPSILON) // Presets hardcode this to 3.0
}

#[component]
pub fn CitySelectionPanel(
    property_data: Signal<PropertyData>,
    rental_data: Signal<RentalData>,
    investment_params: Signal<InvestmentParameters>,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let mut selected_city = use_signal(|| "-- Select --".to_string());
    let mut selected_bedrooms = use_signal(|| "-- Select --".to_string());

    let cities = vec![
        "-- Select --",
        "Washington DC",
        "Boston",
        "New York City",
        "San Francisco",
    ];
    let bedroom_options = vec!["-- Select --", "Studio", "1BR", "2BR", "3BR"];

    // Check if current data matches the selected preset
    let current_matches_preset = use_memo(move || {
        let city = selected_city();
        let bedrooms = selected_bedrooms();

        // If either dropdown is on "-- Select --", no preset is selected
        if city == "-- Select --" || bedrooms == "-- Select --" {
            return false;
        }

        let presets = CityPreset::all_presets();

        // Map display bedroom values to actual bedroom_type values
        let bedroom_type = match bedrooms.as_str() {
            "Studio" => "Studio",
            "1BR" => "1",
            "2BR" => "2",
            "3BR" => "3",
            _ => return false,
        };

        if let Some(preset) = presets
            .iter()
            .find(|p| p.city_name == city && p.bedroom_type == bedroom_type)
        {
            matches_preset(
                &property_data.read(),
                &rental_data.read(),
                &investment_params.read(),
                preset,
            )
        } else {
            false
        }
    });

    let load_preset = move |_| {
        let city = selected_city();
        let bedrooms = selected_bedrooms();

        // Don't load if either dropdown is on "-- Select --"
        if city == "-- Select --" || bedrooms == "-- Select --" {
            return;
        }

        // Find the matching preset based on city and bedroom selection
        let presets = CityPreset::all_presets();

        // Map display bedroom values to actual bedroom_type values
        let bedroom_type = match bedrooms.as_str() {
            "Studio" => "Studio",
            "1BR" => "1",
            "2BR" => "2",
            "3BR" => "3",
            _ => return,
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
                    span { class: "font-semibold", "Current: " }
                    if selected_city() == "-- Select --" || selected_bedrooms() == "-- Select --" {
                        span { class: if dark_mode { "italic text-monokai-fgDim" } else { "italic text-monokaiLight-fgDim" },
                            "None selected"
                        }
                    } else if current_matches_preset() {
                        "{selected_city()} - {selected_bedrooms()}"
                    } else {
                        span { class: if dark_mode { "italic text-monokai-fgDim" } else { "italic text-monokaiLight-fgDim" },
                            "Custom (no preset)"
                        }
                    }
                }
            }
        }
    }
}
