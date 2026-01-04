/// Utility for formatting currency and parsing numeric values
pub struct CurrencyFormatter;

impl CurrencyFormatter {
    /// Format a value as currency (e.g., "$1,234.56")
    pub fn format_currency(value: f64) -> String {
        let is_negative = value < 0.0;
        let abs_value = value.abs();

        // Format with 2 decimal places
        let formatted = format!("{:.2}", abs_value);

        // Split into integer and decimal parts
        let parts: Vec<&str> = formatted.split('.').collect();
        let integer_part = parts[0];
        let decimal_part = parts.get(1).unwrap_or(&"00");

        // Add commas to integer part (from right to left)
        let mut result = String::new();
        for (i, c) in integer_part.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        let integer_with_commas: String = result.chars().rev().collect();

        // Combine with dollar sign and decimal
        if is_negative {
            format!("-${}.{}", integer_with_commas, decimal_part)
        } else {
            format!("${}.{}", integer_with_commas, decimal_part)
        }
    }

    /// Parse a string value to f64, handling currency symbols and commas
    pub fn parse_value(text: &str) -> f64 {
        text.replace('$', "")
            .replace(',', "")
            .trim()
            .parse()
            .unwrap_or(0.0)
    }

    /// Format a percentage value (e.g., "5.25%")
    pub fn format_percent(value: f64) -> String {
        format!("{:.2}%", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_currency() {
        assert_eq!(CurrencyFormatter::format_currency(1234.56), "$1,234.56");
        assert_eq!(
            CurrencyFormatter::format_currency(1000000.0),
            "$1,000,000.00"
        );
        assert_eq!(CurrencyFormatter::format_currency(0.0), "$0.00");
        assert_eq!(CurrencyFormatter::format_currency(999.99), "$999.99");
        assert_eq!(CurrencyFormatter::format_currency(1000.0), "$1,000.00");
        assert_eq!(CurrencyFormatter::format_currency(-1234.56), "-$1,234.56");
        assert_eq!(CurrencyFormatter::format_currency(368487.38), "$368,487.38");
    }

    #[test]
    fn test_parse_value() {
        assert_eq!(CurrencyFormatter::parse_value("$1,234.56"), 1234.56);
        assert_eq!(CurrencyFormatter::parse_value("1234.56"), 1234.56);
        assert_eq!(CurrencyFormatter::parse_value("1,234"), 1234.0);
    }

    #[test]
    fn test_format_percent() {
        assert_eq!(CurrencyFormatter::format_percent(5.25), "5.25%");
        assert_eq!(CurrencyFormatter::format_percent(100.0), "100.00%");
    }
}
