use dioxus::prelude::*;

/// A dual input field component that displays both percentage and dollar inputs side by side.
/// The fields are bidirectionally editable - changing either field updates the other.
#[component]
pub fn DualInputField(
    label: String,
    percent_value: f64,
    dollar_value: f64,
    on_percent_change: EventHandler<f64>,
    on_dollar_change: EventHandler<f64>,
    tooltip: String,
    #[props(default = false)] show_first_year_label: bool,
    #[props(default = None)] dollar_label: Option<String>,
    #[props(default = None)] percent_min: Option<f64>,
    #[props(default = None)] percent_max: Option<f64>,
    #[props(default = None)] dollar_min: Option<f64>,
    #[props(default = None)] dollar_max: Option<f64>,
    #[props(default = false)] percent_min_exclusive: bool,
    #[props(default = false)] percent_max_exclusive: bool,
    #[props(default = false)] dollar_min_exclusive: bool,
    #[props(default = false)] dollar_max_exclusive: bool,
    #[props(default = false)] dark_mode: bool,
    #[props(default = false)] disable_dollar_input: bool,
) -> Element {
    // Validate percentage value
    let percent_is_valid = validate_value(
        percent_value,
        percent_min,
        percent_max,
        percent_min_exclusive,
        percent_max_exclusive,
    );

    // Validate dollar value
    let dollar_is_valid = validate_value(
        dollar_value,
        dollar_min,
        dollar_max,
        dollar_min_exclusive,
        dollar_max_exclusive,
    );

    // Build range text for percentage
    let percent_range_text = build_range_text(
        percent_min,
        percent_max,
        percent_min_exclusive,
        percent_max_exclusive,
    );

    // Build range text for dollar
    let dollar_range_text = build_range_text(
        dollar_min,
        dollar_max,
        dollar_min_exclusive,
        dollar_max_exclusive,
    );

    // Track focus state for inputs
    let mut percent_focused = use_signal(|| false);
    let mut dollar_focused = use_signal(|| false);

    rsx! {
        div { class: "space-y-1",
            // Main label
            label {
                class: if dark_mode {
                    "block text-sm font-medium text-monokai-fgMuted mb-1"
                } else {
                    "block text-sm font-medium text-monokaiLight-fgMuted mb-1"
                },
                "{label}"
            }

            // Side-by-side inputs
            div { class: "grid grid-cols-1 sm:grid-cols-3 gap-2",
                // Percentage input
                div { class: "sm:col-span-2",
                    input {
                        r#type: "text",
                        class: if dark_mode {
                            if percent_is_valid {
                                "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                            } else {
                                "w-full px-3 py-2 border border-monokai-red bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-red focus:border-monokai-red"
                            }
                        } else {
                            if percent_is_valid {
                                "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                            } else {
                                "w-full px-3 py-2 border border-monokaiLight-red bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-red focus:border-monokaiLight-red"
                            }
                        },
                        value: if percent_focused() {
                            format!("{:.2}", percent_value)
                        } else {
                            format!("{:.2}%", percent_value)
                        },
                        inputmode: "decimal",
                        placeholder: "%",
                        onfocus: move |_| {
                            percent_focused.set(true);
                        },
                        onblur: move |_| {
                            percent_focused.set(false);
                        },
                        oninput: move |evt| {
                            // Remove any formatting characters for parsing
                            let clean_value = evt.value().replace("%", "").trim().to_string();
                            if let Ok(v) = clean_value.parse::<f64>() {
                                on_percent_change.call(v);
                            }
                        },
                        title: "{tooltip}"
                    }
                    // Validation message for percentage
                    if !percent_is_valid && !percent_range_text.is_empty() {
                        p {
                            class: if dark_mode {
                                "text-xs text-monokai-red mt-0.5"
                            } else {
                                "text-xs text-monokaiLight-red mt-0.5"
                            },
                            "{percent_range_text}"
                        }
                    } else if percent_is_valid && !percent_range_text.is_empty() {
                        p {
                            class: if dark_mode {
                                "text-xs text-monokai-fgDim mt-0.5"
                            } else {
                                "text-xs text-monokaiLight-fgDim mt-0.5"
                            },
                            "{percent_range_text}"
                        }
                    }
                }

                // Dollar input
                div {
                    input {
                        r#type: "text",
                        disabled: disable_dollar_input,
                        class: if dark_mode {
                            if dollar_is_valid {
                                if disable_dollar_input {
                                    "w-full px-3 py-2 border border-monokai-border bg-monokai-bg text-monokai-fgDim rounded-md shadow-sm cursor-not-allowed"
                                } else {
                                    "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                                }
                            } else {
                                "w-full px-3 py-2 border border-monokai-red bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-red focus:border-monokai-red"
                            }
                        } else {
                            if dollar_is_valid {
                                if disable_dollar_input {
                                    "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bgAlt text-monokaiLight-fgDim rounded-md shadow-sm cursor-not-allowed"
                                } else {
                                    "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                                }
                            } else {
                                "w-full px-3 py-2 border border-monokaiLight-red bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-red focus:border-monokaiLight-red"
                            }
                        },
                        value: if dollar_focused() {
                            format!("{:.2}", dollar_value)
                        } else {
                            format_currency(dollar_value)
                        },
                        inputmode: "decimal",
                        placeholder: "$",
                        onfocus: move |_| {
                            dollar_focused.set(true);
                        },
                        onblur: move |_| {
                            dollar_focused.set(false);
                        },
                        oninput: move |evt| {
                            if !disable_dollar_input {
                                // Remove any formatting characters for parsing
                                let clean_value = evt.value().replace(",", "").replace("$", "");
                                if let Ok(v) = clean_value.parse::<f64>() {
                                    on_dollar_change.call(v);
                                }
                            }
                        },
                        title: if disable_dollar_input {
                            "Calculated value (read-only)"
                        } else {
                            "{tooltip}"
                        }
                    }
                    p {
                        class: if dark_mode {
                            "text-xs text-monokai-fgDim mt-0.5"
                        } else {
                            "text-xs text-monokaiLight-fgDim mt-0.5"
                        },
                        if let Some(ref label) = dollar_label {
                            "{label}"
                        } else if show_first_year_label {
                            "$/year (first year)"
                        } else {
                            "$"
                        }
                    }
                }
            }

            // Validation message for dollar (only if editable and has validation range)
            if !dollar_is_valid && !dollar_range_text.is_empty() && !disable_dollar_input {
                p {
                    class: if dark_mode {
                        "text-xs text-monokai-red mt-1"
                    } else {
                        "text-xs text-monokaiLight-red mt-1"
                    },
                    "{dollar_range_text}"
                }
            }
        }
    }
}

/// Validate a value against min/max constraints
fn validate_value(
    value: f64,
    min: Option<f64>,
    max: Option<f64>,
    min_exclusive: bool,
    max_exclusive: bool,
) -> bool {
    let mut valid = true;

    if let Some(min_val) = min {
        if min_exclusive {
            if value <= min_val {
                valid = false;
            }
        } else {
            if value < min_val {
                valid = false;
            }
        }
    }

    if let Some(max_val) = max {
        if max_exclusive {
            if value >= max_val {
                valid = false;
            }
        } else {
            if value > max_val {
                valid = false;
            }
        }
    }

    valid
}

/// Build range text for validation message
fn build_range_text(
    min: Option<f64>,
    max: Option<f64>,
    min_exclusive: bool,
    max_exclusive: bool,
) -> String {
    match (min, max, min_exclusive, max_exclusive) {
        (Some(min_val), Some(max_val), min_ex, max_ex) => {
            let min_op = if min_ex { ">" } else { ">=" };
            let max_op = if max_ex { "<" } else { "<=" };
            format!(
                "Valid range: {} {} and {} {}",
                min_op, min_val, max_op, max_val
            )
        }
        (Some(min_val), None, min_ex, _) => {
            let min_op = if min_ex { ">" } else { ">=" };
            format!("Must be {} {}", min_op, min_val)
        }
        (None, Some(max_val), _, max_ex) => {
            let max_op = if max_ex { "<" } else { "<=" };
            format!("Must be {} {}", max_op, max_val)
        }
        (None, None, _, _) => String::new(),
    }
}

/// Format a number as currency with commas and 2 decimal places
fn format_currency(value: f64) -> String {
    let abs_value = value.abs();
    let integer_part = abs_value.floor() as u64;
    let decimal_part = ((abs_value - integer_part as f64) * 100.0).round() as u64;

    // Format integer part with commas
    let integer_str = integer_part.to_string();
    let mut formatted = String::new();
    let chars: Vec<char> = integer_str.chars().rev().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(*ch);
    }

    let formatted_integer: String = formatted.chars().rev().collect();

    // Add sign and dollar sign
    let sign = if value < 0.0 { "-" } else { "" };
    format!("{}${}.{:02}", sign, formatted_integer, decimal_part)
}
