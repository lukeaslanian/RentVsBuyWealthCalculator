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

    /// Parse what a user typed into a number input. Ignores "$", ",", "%" and spaces,
    /// so "$8,000", "10%" and " 1,234.5 " all work. Returns None for anything that isn't
    /// a finite number, including "inf" and "NaN", which Rust's parser would otherwise
    /// accept and which would poison every calculation.
    pub fn parse_input(text: &str) -> Option<f64> {
        let cleaned: String = text
            .chars()
            .filter(|c| !matches!(c, '$' | ',' | '%') && !c.is_whitespace())
            .collect();
        let value = cleaned.parse::<f64>().ok()?;
        value.is_finite().then_some(value)
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
    fn test_parse_input() {
        assert_eq!(CurrencyFormatter::parse_input("8000"), Some(8000.0));
        assert_eq!(CurrencyFormatter::parse_input("$8,000"), Some(8000.0));
        assert_eq!(CurrencyFormatter::parse_input(" $1,234.50 "), Some(1234.5));
        assert_eq!(CurrencyFormatter::parse_input("10%"), Some(10.0));
        assert_eq!(CurrencyFormatter::parse_input("5.99 %"), Some(5.99));
        assert_eq!(CurrencyFormatter::parse_input("-5"), Some(-5.0));
        assert_eq!(CurrencyFormatter::parse_input("1,000,000"), Some(1_000_000.0));
        assert_eq!(CurrencyFormatter::parse_input(".5"), Some(0.5));
        // Not numbers: leave the field unchanged
        assert_eq!(CurrencyFormatter::parse_input(""), None);
        assert_eq!(CurrencyFormatter::parse_input("abc"), None);
        assert_eq!(CurrencyFormatter::parse_input("$"), None);
        assert_eq!(CurrencyFormatter::parse_input("1.2.3"), None);
        // Rust's f64 parser accepts these; they must be rejected
        assert_eq!(CurrencyFormatter::parse_input("inf"), None);
        assert_eq!(CurrencyFormatter::parse_input("-infinity"), None);
        assert_eq!(CurrencyFormatter::parse_input("NaN"), None);
    }

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
        // Values just under a whole dollar round up and carry (no "$7,999.100")
        assert_eq!(CurrencyFormatter::format_currency(7999.999999999), "$8,000.00");
        assert_eq!(CurrencyFormatter::format_currency(0.999), "$1.00");
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
