# Rent vs Buy Calculator

A comprehensive financial calculator comparing the long-term wealth outcomes of buying vs. renting a home over 30 years, built with Rust and Dioxus for WebAssembly deployment.

## Features

- Complete buy vs. rent financial analysis over 30 years
- Real-time input validation with visual feedback
- 16 city presets (Washington DC, Boston, NYC, San Francisco x 4 bedroom sizes)
- US Tax Benefits calculation (mortgage interest and property tax deductions)
- Monte Carlo simulations for probabilistic analysis
- Interactive charts showing wealth accumulation
- Year-by-year data breakdown table
- Responsive design with Tailwind CSS
- Dark mode support

## Tax Benefits

The calculator includes optional US tax benefit calculations following current tax law (2025):

- **Standard Deduction**: $15,750 (single) / $31,500 (married filing jointly)
- **SALT Cap**: Property tax deduction capped at $10,000
- **Mortgage Interest**: Fully deductible for loans up to $750,000
- **Itemized vs Standard**: Only provides benefit when itemized deductions exceed standard deduction

This accurately reflects the 2017 Tax Cuts and Jobs Act (TCJA) provisions, which significantly reduced the tax advantages of homeownership for most middle-class buyers.

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
.
├── Cargo.toml              # Dependencies and configuration
├── index.html              # HTML template with Tailwind CSS
├── src/
│   ├── main.rs             # Entry point
│   ├── lib.rs              # Library root
│   ├── models/             # Data models
│   │   ├── property_data.rs    # Buy parameters (includes tax settings)
│   │   ├── rental_data.rs      # Rent parameters
│   │   ├── investment_params.rs
│   │   ├── financial_results.rs
│   │   └── city_preset.rs      # 16 city presets
│   ├── calculators/        # Financial calculation engines
│   │   ├── mortgage_calculator.rs
│   │   ├── wealth_analysis_engine.rs
│   │   ├── breakeven_analyzer.rs
│   │   └── monte_carlo_simulator.rs
│   ├── components/         # Dioxus UI components
│   │   ├── app.rs
│   │   ├── input/          # Input panels
│   │   ├── results/        # Results display
│   │   └── monte_carlo_panel.rs
│   └── utils/              # Utilities
│       ├── config.rs
│       ├── currency_formatter.rs
│       └── validator.rs
```

## Technology Stack

- **Dioxus 0.7**: React-like framework for Rust
- **WebAssembly**: Compile to WASM for browser execution
- **Plotters**: Rust-native charting library
- **Tailwind CSS**: Utility-first CSS framework
- **Serde**: Serialization/deserialization

## Testing

Run unit tests:
```bash
cargo test
```

## Deployment

The built application is a static site that can be deployed to:
- GitHub Pages
- Netlify
- Vercel
- Any static hosting service

Deploy the contents of the `dist/` directory.

### GitHub Pages with GitHub Actions

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [ main ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: wasm32-unknown-unknown
    
    - name: Install Dioxus CLI
      run: cargo install dioxus-cli
    
    - name: Build
      run: dx build --release
    
    - name: Deploy
      uses: peaceiris/actions-gh-pages@v3
      with:
        github_token: ${{ secrets.GITHUB_TOKEN }}
        publish_dir: ./dist
```

## License

MIT License - Luke Aslanian
