/// Logic for finding break-even points in the analysis.
/// Implements Algorithm 3 from the proposal (Section 4.1).
pub struct BreakEvenAnalyzer<'a> {
    series1: &'a [f64], // First data series (e.g., buy wealth)
    series2: &'a [f64], // Second data series (e.g., rent wealth)
}

impl<'a> BreakEvenAnalyzer<'a> {
    /// Create a new BreakEvenAnalyzer
    pub fn new(series1: &'a [f64], series2: &'a [f64]) -> Self {
        Self { series1, series2 }
    }

    /// Find the break-even year where series1 becomes greater than series2
    ///
    /// Algorithm from proposal Section 4.1, Algorithm 3:
    /// For year = 1 to 30:
    ///   If buyWealth[year] > rentWealth[year]:
    ///     Return year as breakEvenYear
    /// If no crossover found: Return None
    ///
    /// Returns the year (1-based) where break-even occurs, or None if never
    pub fn find_break_even_year(&self) -> Option<usize> {
        let length = self.series1.len().min(self.series2.len());

        for year in 1..=length {
            if self.series1[year - 1] > self.series2[year - 1] {
                return Some(year);
            }
        }

        // No crossover found (renting always better, or tie)
        None
    }

    /// Check if there's a crossover point
    pub fn has_crossover(&self) -> bool {
        self.find_break_even_year().is_some()
    }

    /// Get the value at the break-even point
    pub fn value_at_break_even(&self) -> f64 {
        match self.find_break_even_year() {
            Some(year) => self.series1[year - 1],
            None => 0.0,
        }
    }

    /// Get the difference between series at a specific year
    pub fn difference_at_year(&self, year: usize) -> f64 {
        if year < 1 || year > self.series1.len() || year > self.series2.len() {
            return 0.0;
        }
        self.series1[year - 1] - self.series2[year - 1]
    }

    /// Find all break-even years where the advantage switches between scenarios
    ///
    /// Returns a list of years where crossovers occur
    /// (series1 overtakes series2 OR series2 overtakes series1)
    pub fn find_all_break_even_years(&self) -> Vec<usize> {
        let mut break_even_years = Vec::new();
        let length = self.series1.len().min(self.series2.len());

        if length == 0 {
            return break_even_years;
        }

        // Track who was winning in the previous year
        let mut series1_was_ahead = self.series1[0] > self.series2[0];

        // Check each subsequent year for crossovers
        for year in 2..=length {
            let series1_is_ahead = self.series1[year - 1] > self.series2[year - 1];

            // If the leader changed, we have a crossover at this year
            if series1_is_ahead != series1_was_ahead {
                break_even_years.push(year);
                series1_was_ahead = series1_is_ahead;
            }
        }

        break_even_years
    }

    /// Get a descriptive string for all break-even points
    pub fn break_even_description(&self) -> String {
        let break_even_years = self.find_all_break_even_years();

        if break_even_years.is_empty() {
            // Check who wins overall
            if !self.series1.is_empty() && !self.series2.is_empty() {
                let last_idx = self.series1.len() - 1;
                if self.series1[last_idx] > self.series2[last_idx] {
                    return "Never (buying always better)".to_string();
                } else {
                    return "Never (renting always better)".to_string();
                }
            }
            return "Never".to_string();
        }

        if break_even_years.len() == 1 {
            return format!("Year {}", break_even_years[0]);
        }

        // Multiple break-even points
        let mut description = String::from("Years ");
        for (i, year) in break_even_years.iter().enumerate() {
            if i > 0 {
                if i == break_even_years.len() - 1 {
                    description.push_str(" and ");
                } else {
                    description.push_str(", ");
                }
            }
            description.push_str(&year.to_string());
        }
        description
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_break_even_simple() {
        let buy_wealth = vec![100.0, 200.0, 350.0, 500.0, 650.0];
        let rent_wealth = vec![150.0, 250.0, 300.0, 400.0, 600.0];

        let analyzer = BreakEvenAnalyzer::new(&buy_wealth, &rent_wealth);

        // Buying overtakes renting at year 3 (350 > 300)
        assert_eq!(analyzer.find_break_even_year(), Some(3));
        assert!(analyzer.has_crossover());
    }

    #[test]
    fn test_never_breaks_even() {
        let buy_wealth = vec![100.0, 200.0, 300.0, 400.0, 500.0];
        let rent_wealth = vec![150.0, 250.0, 350.0, 450.0, 550.0];

        let analyzer = BreakEvenAnalyzer::new(&buy_wealth, &rent_wealth);

        // Renting is always better
        assert_eq!(analyzer.find_break_even_year(), None);
        assert!(!analyzer.has_crossover());
    }

    #[test]
    fn test_always_better() {
        let buy_wealth = vec![200.0, 400.0, 600.0, 800.0, 1000.0];
        let rent_wealth = vec![100.0, 200.0, 300.0, 400.0, 500.0];

        let analyzer = BreakEvenAnalyzer::new(&buy_wealth, &rent_wealth);

        // Buying is always better (breaks even at year 1)
        assert_eq!(analyzer.find_break_even_year(), Some(1));
    }

    #[test]
    fn test_value_at_break_even() {
        let buy_wealth = vec![100.0, 200.0, 350.0, 500.0];
        let rent_wealth = vec![150.0, 250.0, 300.0, 400.0];

        let analyzer = BreakEvenAnalyzer::new(&buy_wealth, &rent_wealth);

        // Break-even occurs at year 3, where buy wealth is 350
        assert_eq!(analyzer.value_at_break_even(), 350.0);
    }

    #[test]
    fn test_difference_at_year() {
        let buy_wealth = vec![100.0, 200.0, 300.0];
        let rent_wealth = vec![150.0, 180.0, 250.0];

        let analyzer = BreakEvenAnalyzer::new(&buy_wealth, &rent_wealth);

        assert_eq!(analyzer.difference_at_year(1), -50.0); // 100 - 150
        assert_eq!(analyzer.difference_at_year(2), 20.0);  // 200 - 180
        assert_eq!(analyzer.difference_at_year(3), 50.0);  // 300 - 250
    }

    #[test]
    fn test_multiple_crossovers() {
        // Scenario where advantage switches multiple times
        let buy_wealth = vec![100.0, 250.0, 300.0, 350.0, 300.0, 500.0];
        let rent_wealth = vec![200.0, 200.0, 350.0, 300.0, 400.0, 450.0];

        let analyzer = BreakEvenAnalyzer::new(&buy_wealth, &rent_wealth);

        let crossovers = analyzer.find_all_break_even_years();

        // Year 2: buy overtakes rent (250 > 200)
        // Year 3: rent overtakes buy (350 > 300)
        // Year 4: buy overtakes rent (350 > 300)
        // Year 5: rent overtakes buy (400 > 300)
        // Year 6: buy overtakes rent (500 > 450)
        assert_eq!(crossovers.len(), 5);
    }

    #[test]
    fn test_break_even_description() {
        // Single crossover
        let buy_wealth1 = vec![100.0, 200.0, 350.0];
        let rent_wealth1 = vec![150.0, 250.0, 300.0];
        let analyzer1 = BreakEvenAnalyzer::new(&buy_wealth1, &rent_wealth1);
        assert_eq!(analyzer1.break_even_description(), "Year 3");

        // No crossover (renting always better)
        let buy_wealth2 = vec![100.0, 200.0, 300.0];
        let rent_wealth2 = vec![150.0, 250.0, 350.0];
        let analyzer2 = BreakEvenAnalyzer::new(&buy_wealth2, &rent_wealth2);
        assert_eq!(analyzer2.break_even_description(), "Never (renting always better)");

        // No crossover (buying always better)
        let buy_wealth3 = vec![200.0, 300.0, 400.0];
        let rent_wealth3 = vec![100.0, 200.0, 300.0];
        let analyzer3 = BreakEvenAnalyzer::new(&buy_wealth3, &rent_wealth3);
        assert_eq!(analyzer3.break_even_description(), "Never (buying always better)");
    }
}
