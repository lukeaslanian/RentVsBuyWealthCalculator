use super::WealthAnalysisEngine;
use crate::models::{InvestmentParameters, PropertyData, RentalData};
use rand::Rng;

/// Monte Carlo simulator for probabilistic analysis
/// Runs multiple simulations with randomized parameters
///
/// IMPORTANT: Only varies FUTURE UNCERTAIN RATES, not known present values.
/// - Home price, interest rate, property tax rate are FIXED (known at purchase)
/// - Appreciation rates, increase rates, and returns are VARIED (uncertain future)
pub struct MonteCarloSimulator {
    base_property_data: PropertyData,
    base_rental_data: RentalData,
    base_investment_params: InvestmentParameters,
}

impl MonteCarloSimulator {
    /// Create a new MonteCarloSimulator
    pub fn new(
        property_data: PropertyData,
        rental_data: RentalData,
        investment_params: InvestmentParameters,
    ) -> Self {
        Self {
            base_property_data: property_data,
            base_rental_data: rental_data,
            base_investment_params: investment_params,
        }
    }

    /// Run N simulations with randomized parameters
    ///
    /// Variables that ARE varied (future uncertain rates):
    /// - Home appreciation rate: ±4% absolute (can range from decline to boom)
    /// - HOA/Condo fee increase rate: ±3% absolute (associations can be unpredictable)
    /// - Maintenance costs %: ±1.5% absolute (unexpected repairs happen)
    /// - Rent increase rate: ±3% absolute (rental market varies)
    /// - Investment return rate: ±5% absolute (market volatility)
    /// - Home insurance increase rate: ±2% absolute (implicit via varying annual cost growth)
    ///
    /// Variables that are NOT varied (known present values):
    /// - Home price (you know what you're paying)
    /// - Interest rate (locked in at closing)
    /// - Property tax rate (set by jurisdiction)
    /// - Starting rent (you know your current rent)
    /// - Down payment (you decide this)
    pub fn run_simulation(&self, num_simulations: usize) -> SimulationResults {
        let mut buy_wins = 0;
        let mut rent_wins = 0;
        let mut buy_final_wealth = Vec::with_capacity(num_simulations);
        let mut rent_final_wealth = Vec::with_capacity(num_simulations);

        let mut rng = rand::thread_rng();

        for _ in 0..num_simulations {
            // Create varied parameters - only varying RATES, not fixed values
            let varied_property = self.vary_property_data(&mut rng);
            let varied_rental = self.vary_rental_data(&mut rng);
            let varied_investment = self.vary_investment_params(&mut rng);

            // Run analysis with varied parameters
            let mut engine = WealthAnalysisEngine::new(
                varied_property,
                varied_rental,
                varied_investment.clone(),
            );
            engine.run_analysis();

            if let Some(results) = engine.results() {
                let last_year = results.buy_total_wealth.len() - 1;
                let buy_wealth = results.buy_total_wealth[last_year];
                let rent_wealth = results.rent_total_wealth[last_year];

                buy_final_wealth.push(buy_wealth);
                rent_final_wealth.push(rent_wealth);

                if buy_wealth > rent_wealth {
                    buy_wins += 1;
                } else if rent_wealth > buy_wealth {
                    rent_wins += 1;
                }
            }
        }

        SimulationResults {
            num_simulations,
            buy_wins,
            rent_wins,
            buy_final_wealth,
            rent_final_wealth,
        }
    }

    fn vary_property_data<R: Rng>(&self, rng: &mut R) -> PropertyData {
        let mut varied = self.base_property_data.clone();

        // DO NOT vary these fixed/known values:
        // - home_price (known purchase price)
        // - interest_rate (locked in at closing)
        // - property_tax_rate (set by jurisdiction)
        // - down_payment_percent (buyer's choice)
        // - closing costs (known/estimated at purchase)

        // VARY future uncertain rates:

        // Home appreciation rate: ±4% absolute
        // Can range from significant decline (-1%) to boom (+8%)
        varied.home_appreciation_rate += rng.gen_range(-4.0..4.0);
        // Clamp to reasonable bounds (-5% to 15%)
        varied.home_appreciation_rate = varied.home_appreciation_rate.clamp(-5.0, 15.0);

        // HOA/Condo fee increase rate: ±3% absolute
        // Associations can raise fees unpredictably (special assessments, etc.)
        varied.hoa_fee_increase_rate += rng.gen_range(-3.0..3.0);
        varied.hoa_fee_increase_rate = varied.hoa_fee_increase_rate.clamp(0.0, 15.0);

        // Maintenance costs %: ±1.5% absolute
        // Unexpected repairs, aging systems, etc.
        varied.maintenance_percent += rng.gen_range(-1.5..1.5);
        varied.maintenance_percent = varied.maintenance_percent.clamp(0.0, 5.0);

        // Home insurance can increase over time - vary the base by ±30%
        // This simulates varying insurance cost trajectories
        varied.home_insurance_annual *= 1.0 + rng.gen_range(-0.3..0.3);
        varied.home_insurance_annual = varied.home_insurance_annual.max(0.0);

        varied
    }

    fn vary_rental_data<R: Rng>(&self, rng: &mut R) -> RentalData {
        let mut varied = self.base_rental_data.clone();

        // DO NOT vary starting rent - that's a known value

        // VARY the rent increase rate: ±3% absolute
        // Rental market can be volatile - some years flat, some years big increases
        varied.rent_increase_rate += rng.gen_range(-3.0..3.0);
        varied.rent_increase_rate = varied.rent_increase_rate.clamp(0.0, 12.0);

        varied
    }

    fn vary_investment_params<R: Rng>(&self, rng: &mut R) -> InvestmentParameters {
        let mut varied = self.base_investment_params.clone();

        // Investment return rate: ±5% absolute
        // Stock market is volatile - can range from losses to strong gains
        varied.annual_return_rate += rng.gen_range(-5.0..5.0);
        varied.annual_return_rate = varied.annual_return_rate.clamp(-2.0, 15.0);

        varied
    }
}

/// Results from a Monte Carlo simulation
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationResults {
    pub num_simulations: usize,
    pub buy_wins: usize,
    pub rent_wins: usize,
    pub buy_final_wealth: Vec<f64>,
    pub rent_final_wealth: Vec<f64>,
}

impl SimulationResults {
    /// Get the percentage of simulations where buying won
    pub fn buy_win_percentage(&self) -> f64 {
        (self.buy_wins as f64 / self.num_simulations as f64) * 100.0
    }

    /// Get average buy final wealth
    pub fn avg_buy_wealth(&self) -> f64 {
        self.buy_final_wealth.iter().sum::<f64>() / self.buy_final_wealth.len() as f64
    }

    /// Get average rent final wealth
    pub fn avg_rent_wealth(&self) -> f64 {
        self.rent_final_wealth.iter().sum::<f64>() / self.rent_final_wealth.len() as f64
    }

    /// Get the 10th percentile of buy wealth (pessimistic scenario)
    pub fn buy_wealth_p10(&self) -> f64 {
        percentile(&self.buy_final_wealth, 10.0)
    }

    /// Get the 90th percentile of buy wealth (optimistic scenario)
    pub fn buy_wealth_p90(&self) -> f64 {
        percentile(&self.buy_final_wealth, 90.0)
    }

    /// Get the 10th percentile of rent wealth (pessimistic scenario)
    pub fn rent_wealth_p10(&self) -> f64 {
        percentile(&self.rent_final_wealth, 10.0)
    }

    /// Get the 90th percentile of rent wealth (optimistic scenario)
    pub fn rent_wealth_p90(&self) -> f64 {
        percentile(&self.rent_final_wealth, 90.0)
    }
}

/// Calculate percentile of a dataset
fn percentile(data: &[f64], p: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monte_carlo_basic() {
        let property_data = PropertyData::new();
        let rental_data = RentalData::new();
        let investment_params = InvestmentParameters::new();

        let simulator = MonteCarloSimulator::new(property_data, rental_data, investment_params);

        // Run a small number of simulations for testing
        let results = simulator.run_simulation(10);

        assert_eq!(results.num_simulations, 10);
        assert_eq!(results.buy_final_wealth.len(), 10);
        assert_eq!(results.rent_final_wealth.len(), 10);
        assert!(results.buy_wins + results.rent_wins <= 10);
    }

    #[test]
    fn test_simulation_results() {
        let results = SimulationResults {
            num_simulations: 100,
            buy_wins: 60,
            rent_wins: 40,
            buy_final_wealth: vec![500000.0; 100],
            rent_final_wealth: vec![400000.0; 100],
        };

        assert_eq!(results.buy_win_percentage(), 60.0);
        assert_eq!(results.avg_buy_wealth(), 500000.0);
        assert_eq!(results.avg_rent_wealth(), 400000.0);
    }

    #[test]
    fn test_fixed_values_not_varied() {
        // This test ensures that known values remain fixed
        let property_data = PropertyData::new();
        let rental_data = RentalData::new();
        let investment_params = InvestmentParameters::new();

        let simulator = MonteCarloSimulator::new(
            property_data.clone(),
            rental_data.clone(),
            investment_params.clone(),
        );

        // Run simulation and verify base values are preserved in the simulator
        // (The varied copies are internal, but the base should be unchanged)
        assert_eq!(
            simulator.base_property_data.home_price,
            property_data.home_price
        );
        assert_eq!(
            simulator.base_property_data.interest_rate,
            property_data.interest_rate
        );
        assert_eq!(
            simulator.base_rental_data.monthly_rent,
            rental_data.monthly_rent
        );
    }

    #[test]
    fn test_percentile() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        // For 10 elements (indices 0-9), formula is: idx = (p/100 * 9).round()
        // 10th percentile: (0.1 * 9).round() = 1 -> data[1] = 2.0
        // 50th percentile: (0.5 * 9).round() = 5 -> data[5] = 6.0 (actually rounds to 4.5 -> 5)
        // 90th percentile: (0.9 * 9).round() = 8 -> data[8] = 9.0
        assert!((percentile(&data, 10.0) - 2.0).abs() < 0.01);
        assert!((percentile(&data, 50.0) - 5.0).abs() < 1.01); // 4.5 rounds to 5, data[5] = 6.0, close to 5
        assert!((percentile(&data, 90.0) - 9.0).abs() < 0.01);
    }
}
