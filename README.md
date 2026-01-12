## To Buy Or Not To Buy: A Comprehensive Rent vs Buy Calculator with Ample Analytics

An interactive financial calculator comparing the long-term wealth outcomes of buying vs. renting a home over 30 years (or a time period of your choice), built with Rust and Dioxus for WebAssembly deployment. 

Unlike most Rent vs. Buy Calculators that I am aware of, this app assumes that the renter invests what they _would have spent_ on a down payment and closing costs into index funds or other investments. It also differs from most Rent vs. Buy calculators in that it assumes that each month, the hypothetical person who has lower monthly costs (than they would have had they chosen the other modality) invests the amount they 'saved' that month. These changes make the calculator fairer to (at least financially savvy) renters and make it more of a hypothetical "wealth" calculator than just a cost comparison calculator. 

I also added a [Monte Carlo](https://en.wikipedia.org/wiki/Monte_Carlo_method) feature I haven't ever seen in a Rent vs. Buy calculator before that accounts for some of the randomness that could exist in real life variations of our hypothetical renter and buyer. The result is a % and a "X out of Y times" result of which modality was better in our simulation.

I built this app after completing a Java & Swing version for one of my Harvard Extension School Courses, as I wanted to make a much more modern and web-friendly (or in this case due to Dioxus' strengths, an almost platform-agnostic) version of my app!

## Screenshots

Input Panel:
<img width="1904" height="926" alt="image" src="https://github.com/user-attachments/assets/0adbe7c1-6112-4a3f-91a7-87365a9ca366" />

Results Panel:
<img width="1340" height="929" alt="image" src="https://github.com/user-attachments/assets/1f18695c-1461-410c-bb47-d7f64c0e6cc1" />

Monte Carlo Panel:
<img width="1047" height="932" alt="image" src="https://github.com/user-attachments/assets/3c996702-413c-4d24-9409-3dd8c42435c0" />

## Features

- Complete buy vs. rent financial analysis over whatever time period you choose (defaults to 30 years)
- **Live mortgage rate fetching** from FRED API (optional, with fallback to defaults)
- Real-time input validation with visual feedback and smart formatting
  - Dollar amounts display with commas and $ symbol (e.g., $575,000.00)
  - Percentages auto-format with % symbol (e.g., 5.99%)
- 16 city presets (Washington DC, Boston, NYC, San Francisco x 4 bedroom sizes)
- US Tax Benefits calculation (mortgage interest and property tax deductions, **enabled by default**)
- Monte Carlo simulations for probabilistic analysis over the time period
- Interactive charts showing wealth accumulation with tooltips
- Year-by-year data breakdown table
- Responsive design with Tailwind CSS
- Dark mode support with a nice ristretto-monokai-esque colorway

## Tax Benefits

The calculator includes US tax benefit calculations following current tax law (2025), **enabled by default**:

- **Standard Deduction**: $15,750 (single) / $31,500 (married filing jointly)
- **SALT Cap**: Property tax deduction capped at $10,000
- **Mortgage Interest**: Fully deductible for loans up to $750,000
- **Itemized vs Standard**: Only provides benefit when itemized deductions exceed standard deduction
- **HOA Fees**: Generally NOT deductible (default: 0%) unless you have a home office

This accurately reflects the 2017 Tax Cuts and Jobs Act (TCJA) provisions, which significantly reduced the tax advantages of homeownership for many middle-class buyers.

## Live Mortgage Rates

The app can fetch current 30-year fixed mortgage rates from the [Federal Reserve Economic Data (FRED)](https://fred.stlouisfed.org/) API at build time and during runtime.

### For Local Development (Optional)

To enable live rate fetching:

1. Get a free API key from https://fred.stlouisfed.org/docs/api/api_key.html
2. Set the environment variable:
   ```bash
   export FRED_API_KEY=your_api_key_here
   dx serve
   ```
3. The "Live Rate" button in the Interest Rate field will fetch current rates

Without the API key, the app uses a fallback rate of 5.99% and the Live Rate button shows an appropriate message.

### For GitHub Pages Deployment

The GitHub Actions workflow automatically uses the `FRED_API_KEY` secret if configured:

1. Go to your repo's Settings → Secrets and variables → Actions
2. Add a new repository secret named `FRED_API_KEY`
3. The next deployment will include the live rate fetched at build time

The compiled WASM will also use this key for the "Live Rate" button on the deployed site.

## Prerequisites

1. **Install Rust**: https://rustup.rs/
2. **Install wasm32 target**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
3. **Install Dioxus CLI**:
   ```bash
   cargo install dioxus-cli
   ```

## Development

### Run Development Server
```bash
dx serve
```

The app will be available at `http://localhost:8080`

### Build for Production
```bash
dx build --release
```

Output will be in the `dist/` directory.

## Project Structure

```
├── Cargo.toml                  # Dependencies and configuration
├── build.rs                    # Build script (fetches FRED mortgage rates)
├── Dioxus.toml                 # Dioxus CLI configuration
├── index.html                  # HTML template with Tailwind CSS
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library root
│   ├── models/                 # Data models
│   │   ├── property_data.rs    # Buy parameters (includes tax settings, default rates)
│   │   ├── rental_data.rs      # Rent parameters
│   │   ├── investment_params.rs
│   │   ├── financial_results.rs
│   │   └── city_preset.rs      # 16 city presets
│   ├── calculators/            # Financial calculation engines
│   │   ├── mortgage_calculator.rs
│   │   ├── wealth_analysis_engine.rs
│   │   ├── breakeven_analyzer.rs
│   │   └── monte_carlo_simulator.rs
│   ├── components/             # Dioxus UI components
│   │   ├── app.rs
│   │   ├── input/              # Input panels (with smart formatting)
│   │   │   ├── buying_panel.rs
│   │   │   ├── rental_panel.rs
│   │   │   ├── shared_params_panel.rs
│   │   │   └── dual_input_field.rs
│   │   ├── results/            # Results display
│   │   └── monte_carlo_panel.rs
│   └── utils/                  # Utilities
│       ├── config.rs
│       ├── currency_formatter.rs
│       ├── fred_api.rs         # FRED API client (runtime fetching)
│       └── validator.rs
└── .github/workflows/
    └── deploy.yml              # GitHub Actions
```

## Tech Stack

- **Dioxus**: React-like framework for Rust
- **WebAssembly**: Compile to WASM for browser execution
- **Charming**: Rust charting library based on Apache EChart
- **Tailwind CSS**: Utility-first CSS framework
- **Serde**: Serialization/deserialization

## Testing

Run unit tests:
```bash
cargo test
```

## Deployment

The built application is a static site that can be deployed to GitHub Pages, Netlify, Vercel or other static hosting services.

To do so, deploy the contents of the `dist/` directory.

### GitHub Pages

This repo includes a GitHub Actions workflow (`.github/workflows/deploy.yml`) that automatically builds and deploys to GitHub Pages on push to `main`.

**For live mortgage rates:** Add the `FRED_API_KEY` secret to your repository (see [Live Mortgage Rates](#live-mortgage-rates) section above).

## License

MIT License - Luke Aslanian
