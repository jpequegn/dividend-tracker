# Contributing to Dividend Tracker

We welcome contributions from the community! Whether you're fixing a bug, adding a new feature, improving documentation, or helping with testing, your contributions make this project better for everyone.

## Table of Contents

1. [How to Contribute](#how-to-contribute)
2. [Development Setup](#development-setup)
3. [Coding Standards](#coding-standards)
4. [Testing Guidelines](#testing-guidelines)
5. [Documentation](#documentation)
6. [Submitting Changes](#submitting-changes)
7. [Reporting Issues](#reporting-issues)
8. [Feature Requests](#feature-requests)
9. [Code Review Process](#code-review-process)
10. [Community Guidelines](#community-guidelines)

## How to Contribute

### Types of Contributions

We appreciate all kinds of contributions:

- **Bug fixes**: Help us fix issues and improve reliability
- **New features**: Add functionality that benefits dividend tracking
- **Documentation**: Improve guides, examples, and API documentation
- **Testing**: Add test cases, improve test coverage, or test on different platforms
- **Performance**: Optimize algorithms and reduce resource usage
- **Security**: Identify and fix security vulnerabilities
- **UI/UX**: Improve command-line interface and user experience

### Getting Started

1. **Check existing issues** to see if your contribution is already being worked on
2. **Open an issue** to discuss major changes before implementing
3. **Fork the repository** and create a feature branch
4. **Make your changes** following our coding standards
5. **Test thoroughly** on your local machine
6. **Submit a pull request** with a clear description

## Development Setup

### Prerequisites

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **Editor**: We recommend VS Code with rust-analyzer extension

### Initial Setup

```bash
# Fork and clone the repository
git clone https://github.com/your-username/dividend-tracker.git
cd dividend-tracker

# Create a development branch
git checkout -b feature/your-feature-name

# Build the project
cargo build

# Run tests to ensure everything works
cargo test

# Format and lint
cargo fmt
cargo clippy
```

### Development Environment

#### Recommended VS Code Extensions

- **rust-analyzer**: Rust language server
- **CodeLLDB**: Debugging support
- **Better TOML**: TOML file support
- **GitLens**: Enhanced Git integration

#### Useful Commands

```bash
# Watch mode for development
cargo watch -x test

# Documentation generation
cargo doc --open

# Check for outdated dependencies
cargo outdated

# Security audit
cargo audit

# Benchmark tests
cargo bench
```

### Project Structure

```
dividend-tracker/
├── src/
│   ├── main.rs           # CLI application entry point
│   ├── models.rs         # Core data structures
│   ├── analytics.rs      # Portfolio analytics
│   ├── tax.rs            # Tax reporting features
│   ├── projections.rs    # Future income projections
│   ├── api.rs            # External API integration
│   ├── persistence.rs    # Data storage and loading
│   ├── holdings.rs       # Portfolio management
│   ├── notifications.rs  # Alerts and calendar
│   └── config.rs         # Configuration management
├── tests/                # Integration tests
├── examples/             # Sample data files
├── docs/                 # Additional documentation
└── target/               # Build artifacts (gitignored)
```

## Coding Standards

### Rust Style Guidelines

We follow the official Rust style guidelines with some project-specific conventions:

#### Formatting

```bash
# Always format code before committing
cargo fmt

# Check formatting in CI
cargo fmt -- --check
```

#### Linting

```bash
# Address all clippy warnings
cargo clippy -- -D warnings

# Specific lints we enforce
cargo clippy -- -D clippy::unwrap_used -D clippy::expect_used
```

#### Code Quality

1. **Error Handling**: Use `anyhow::Result<T>` for functions that can fail
   ```rust
   use anyhow::{Result, Context};

   fn process_dividend(data: &str) -> Result<Dividend> {
       let parsed = parse_data(data)
           .with_context(|| format!("Failed to parse dividend data: {}", data))?;
       Ok(parsed)
   }
   ```

2. **Documentation**: All public functions must have documentation
   ```rust
   /// Calculate the total dividend income for a specific year
   ///
   /// # Arguments
   /// * `year` - The year to calculate income for
   ///
   /// # Returns
   /// Total dividend income as a `Decimal`
   ///
   /// # Example
   /// ```
   /// let total = tracker.get_total_income_for_year(2024);
   /// ```
   pub fn get_total_income_for_year(&self, year: i32) -> Decimal {
       // Implementation
   }
   ```

3. **Type Safety**: Use strong typing and avoid stringly-typed interfaces
   ```rust
   // Good
   pub enum TaxClassification {
       Qualified,
       NonQualified,
   }

   // Avoid
   pub fn set_classification(classification: String) // Too loose
   ```

4. **Financial Precision**: Always use `Decimal` for monetary values
   ```rust
   use rust_decimal::Decimal;

   // Good
   pub amount: Decimal,

   // Never use for money
   pub amount: f64, // Floating point errors!
   ```

### Naming Conventions

- **Functions**: `snake_case` and descriptive
- **Structs/Enums**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

```rust
// Good examples
pub struct DividendTracker;
pub enum TaxClassification;
pub fn calculate_total_income() -> Decimal;
pub const DEFAULT_YIELD_THRESHOLD: Decimal;
```

### Error Handling Patterns

```rust
use anyhow::{bail, ensure, Context, Result};

// Use anyhow::Result for public APIs
pub fn add_dividend(&mut self, dividend: Dividend) -> Result<()> {
    // Validate input
    ensure!(!dividend.symbol.is_empty(), "Symbol cannot be empty");
    ensure!(dividend.amount_per_share > Decimal::ZERO, "Amount must be positive");

    // Check for duplicates
    if self.has_duplicate(&dividend) {
        bail!("Duplicate dividend record for {} on {}", dividend.symbol, dividend.ex_date);
    }

    // Add with context
    self.dividends.push(dividend);
    self.save().context("Failed to save dividend data")?;

    Ok(())
}
```

## Testing Guidelines

### Test Structure

We use a multi-layered testing approach:

1. **Unit Tests**: Test individual functions and methods
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test CLI commands and workflows
4. **Property Tests**: Test invariants with generated data

### Writing Tests

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_dividend_calculation() {
        let dividend = Dividend::new(
            "AAPL".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 9).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            Decimal::from_str("0.24").unwrap(),
            Decimal::from_str("100").unwrap(),
        ).unwrap();

        assert_eq!(dividend.total_amount, Decimal::from_str("24.00").unwrap());
    }

    #[test]
    fn test_invalid_dividend_amount() {
        let result = Dividend::new(
            "AAPL".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 9).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            Decimal::from_str("-0.24").unwrap(), // Invalid negative amount
            Decimal::from_str("100").unwrap(),
        );

        assert!(result.is_err());
    }
}
```

#### Integration Tests

```rust
// tests/integration_tests.rs
use dividend_tracker::models::DividendTracker;
use tempfile::TempDir;

#[test]
fn test_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let mut tracker = DividendTracker::new();

    // Test complete workflow
    let dividend = create_test_dividend();
    tracker.add_dividend(dividend).unwrap();

    let total = tracker.get_total_income_for_year(2024);
    assert_eq!(total, Decimal::from_str("24.00").unwrap());
}
```

#### CLI Tests

```rust
// tests/cli_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_add_dividend_command() {
    let mut cmd = Command::cargo_bin("dividend-tracker").unwrap();
    cmd.args(&["add", "AAPL", "--ex-date", "2024-02-09", "--pay-date", "2024-02-15", "--amount", "0.24", "--shares", "100"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added dividend"));
}
```

### Test Data

Use the sample files in `examples/` for consistent test data:

```rust
fn load_test_data() -> DividendTracker {
    let mut tracker = DividendTracker::new();
    // Load from examples/sample_dividends.csv
    tracker.import_csv("examples/sample_dividends.csv").unwrap();
    tracker
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_dividend_calculation

# Run integration tests only
cargo test --test integration_tests

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

## Documentation

### Types of Documentation

1. **Code Documentation**: Inline docs for all public APIs
2. **User Guides**: README, Tutorial, FAQ
3. **API Reference**: Library usage documentation
4. **Examples**: Working code samples

### Writing Documentation

#### Code Documentation

```rust
/// Calculate year-over-year dividend growth
///
/// This function analyzes dividend payments across multiple years to determine
/// the growth rate for each stock and the overall portfolio.
///
/// # Arguments
/// * `years` - Vector of years to analyze (must be in chronological order)
///
/// # Returns
/// `Result<GrowthAnalysis>` containing growth rates and statistics
///
/// # Errors
/// Returns an error if:
/// - Years vector is empty
/// - Insufficient data for analysis
/// - Data integrity issues
///
/// # Example
/// ```
/// use dividend_tracker::analytics::PortfolioAnalytics;
///
/// let analytics = PortfolioAnalytics::new(&tracker);
/// let growth = analytics.growth_analysis(vec![2022, 2023, 2024])?;
/// println!("Growth rate: {}%", growth.overall_growth_rate);
/// ```
pub fn growth_analysis(&self, years: Vec<i32>) -> Result<GrowthAnalysis> {
    // Implementation
}
```

#### Examples

Provide working examples in documentation:

```rust
/// # Examples
///
/// Basic usage:
/// ```
/// # use dividend_tracker::models::{DividendTracker, Dividend};
/// # use chrono::NaiveDate;
/// # use rust_decimal::Decimal;
/// # use std::str::FromStr;
/// let mut tracker = DividendTracker::new();
///
/// let dividend = Dividend::new(
///     "AAPL".to_string(),
///     NaiveDate::from_ymd_opt(2024, 2, 9).unwrap(),
///     NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
///     Decimal::from_str("0.24").unwrap(),
///     Decimal::from_str("100").unwrap(),
/// )?;
///
/// tracker.add_dividend(dividend)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
```

### Documentation Generation

```bash
# Generate and open documentation
cargo doc --open

# Check for documentation warnings
cargo doc --no-deps 2>&1 | grep warning
```

## Submitting Changes

### Pull Request Process

1. **Create descriptive branch name**:
   ```bash
   git checkout -b feature/add-tax-classification
   git checkout -b fix/csv-import-bug
   git checkout -b docs/improve-api-examples
   ```

2. **Make atomic commits**:
   ```bash
   git add -p  # Stage specific changes
   git commit -m "Add tax classification support for dividends

   - Add TaxClassification enum with US tax categories
   - Update Dividend struct to include tax_classification field
   - Add methods for classifying dividends by symbol
   - Include migration for existing data (defaults to Unknown)

   Fixes #123"
   ```

3. **Keep commits clean**:
   ```bash
   # Squash fixup commits before submitting
   git rebase -i HEAD~3
   ```

4. **Write clear PR description**:
   ```markdown
   ## Summary

   Adds tax classification support for dividend tracking to help with tax reporting.

   ## Changes

   - [ ] Add `TaxClassification` enum
   - [ ] Update `Dividend` struct with tax classification field
   - [ ] Add classification methods to CLI
   - [ ] Update documentation and examples
   - [ ] Add comprehensive tests

   ## Testing

   - [ ] All existing tests pass
   - [ ] New unit tests for tax classification
   - [ ] Integration tests for CLI commands
   - [ ] Manual testing with sample data

   ## Documentation

   - [ ] Updated API documentation
   - [ ] Added examples to tutorial
   - [ ] Updated CLI help text

   ## Breaking Changes

   None - new field defaults to `Unknown` for backward compatibility.

   Closes #123
   ```

### Commit Message Guidelines

Follow conventional commit format:

```
type(scope): description

body (optional)

footer (optional)
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, no code change
- `refactor`: Code change that neither fixes bug nor adds feature
- `test`: Adding missing tests
- `chore`: Changes to build process or auxiliary tools

**Examples:**
```
feat(tax): add tax classification for dividends

fix(csv): handle UTF-8 BOM in CSV imports

docs(api): add examples for portfolio analytics

test(models): increase test coverage for Dividend struct
```

### Pre-submission Checklist

Before submitting a pull request:

- [ ] Code follows style guidelines (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] All tests pass (`cargo test`)
- [ ] Documentation is updated
- [ ] Examples work correctly
- [ ] Commit messages follow convention
- [ ] PR description is complete
- [ ] Breaking changes are documented

## Reporting Issues

### Bug Reports

When reporting bugs, include:

1. **Clear description** of the problem
2. **Steps to reproduce** the issue
3. **Expected behavior** vs actual behavior
4. **Environment information**:
   - Operating system and version
   - Rust version (`rustc --version`)
   - Application version
5. **Sample data** if relevant (anonymized)
6. **Error messages** or logs

**Template:**
```markdown
## Bug Description
Brief description of what went wrong.

## Steps to Reproduce
1. Run command: `dividend-tracker add AAPL --ex-date 2024-02-09 --pay-date 2024-02-15 --amount 0.24 --shares 100`
2. Expected: Dividend added successfully
3. Actual: Error message "Invalid date format"

## Environment
- OS: macOS 14.0
- Rust: 1.70.0
- App Version: 0.1.0

## Sample Data
```csv
symbol,ex_date,pay_date,amount,shares
AAPL,2024-02-09,2024-02-15,0.24,100
```

## Error Output
```
Error: Invalid date format: "2024-02-09"
```
```

### Security Issues

For security vulnerabilities:

1. **Do NOT** open a public issue
2. **Email security concerns** to: [security@example.com]
3. **Include** detailed description and reproduction steps
4. **Allow time** for investigation before public disclosure

## Feature Requests

### Before Requesting

1. **Check existing issues** to avoid duplicates
2. **Consider scope** - is this aligned with project goals?
3. **Think about alternatives** - could existing features solve this?

### Request Template

```markdown
## Feature Description
Clear description of the proposed feature.

## Problem Statement
What problem does this solve? Who benefits?

## Proposed Solution
How should this work? Include examples.

## Alternatives Considered
What other approaches were considered?

## Additional Context
Mockups, examples, research, etc.
```

### Feature Priority

We prioritize features based on:

1. **User impact**: How many users benefit?
2. **Alignment**: Does it fit project goals?
3. **Complexity**: Implementation effort vs benefit
4. **Maintenance**: Long-term support requirements

## Code Review Process

### Review Criteria

We review for:

1. **Correctness**: Does the code work as intended?
2. **Performance**: Are there obvious performance issues?
3. **Security**: Any security implications?
4. **Maintainability**: Is code readable and well-structured?
5. **Testing**: Adequate test coverage?
6. **Documentation**: Public APIs documented?

### Review Timeline

- **Initial response**: Within 2-3 days
- **Full review**: Within 1 week
- **Complex features**: May require multiple review cycles

### Addressing Feedback

1. **Respond to all comments** - even if just "Fixed"
2. **Ask for clarification** if feedback is unclear
3. **Separate fixup commits** initially, squash before merge
4. **Re-request review** after addressing feedback

## Community Guidelines

### Code of Conduct

We are committed to providing a welcoming and inclusive environment:

1. **Be respectful** in all interactions
2. **Be constructive** when providing feedback
3. **Be patient** with new contributors
4. **Be collaborative** in problem-solving

### Communication Channels

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: General questions, ideas
- **Pull Requests**: Code review and discussion
- **Documentation**: Inline comments for specific clarification

### Recognition

We value all contributions:

- **Contributors** are listed in release notes
- **Significant contributors** may be invited as maintainers
- **Good first issues** are labeled for newcomers

## Getting Help

### Documentation

1. [README.md](README.md) - Overview and quick start
2. [TUTORIAL.md](TUTORIAL.md) - Comprehensive user guide
3. [API.md](API.md) - Library API reference
4. [FAQ.md](FAQ.md) - Common questions

### Community Support

- **GitHub Discussions**: For questions and ideas
- **GitHub Issues**: For bugs and feature requests
- **Code comments**: For implementation questions

### Maintainer Contact

For maintainer-specific questions:

- Open a GitHub issue with `@maintainer` mention
- Use appropriate labels (`question`, `help wanted`)

---

Thank you for contributing to Dividend Tracker! Your efforts help make financial tracking more accessible and accurate for everyone. 🚀📈
