# Contributing to Emberleaf

Thank you for considering contributing to Emberleaf! This document provides guidelines for contributing to the project.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Adding a New Tauri Command](#adding-a-new-tauri-command)
5. [Testing Guidelines](#testing-guidelines)
6. [Pull Request Process](#pull-request-process)
7. [Code Style](#code-style)

---

## Code of Conduct

Emberleaf is committed to providing a welcoming and inclusive environment. Please be respectful and considerate in all interactions.

---

## Getting Started

### Prerequisites

- **Node.js:** 20+
- **Rust:** 1.75+ (stable)
- **npm:** 10+
- **System Dependencies (Ubuntu/Debian):**
  ```bash
  sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    patchelf \
    libasound2-dev \
    libpulse-dev
  ```

### Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-org/emberleaf.git
   cd emberleaf
   ```

2. **Install dependencies:**
   ```bash
   npm ci
   ```

3. **Run in development mode:**
   ```bash
   npm run dev
   ```

4. **Run tests:**
   ```bash
   # Frontend tests
   npm run test

   # Backend tests
   cargo test --manifest-path src-tauri/Cargo.toml

   # Validation tests only
   cargo test --manifest-path src-tauri/Cargo.toml validation
   ```

---

## Development Workflow

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** and ensure tests pass

3. **Run linters:**
   ```bash
   npm run lint
   cargo fmt --manifest-path src-tauri/Cargo.toml --check
   cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features
   ```

4. **Commit your changes:**
   ```bash
   git add .
   git commit -m "feat: Add your feature description"
   ```

5. **Push to your fork:**
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Open a Pull Request** on GitHub

---

## Adding a New Tauri Command — Checklist

When adding a new Tauri command that accepts user input, follow this checklist to ensure security and consistency:

### 1. Define Inputs and Acceptable Ranges

Determine what inputs your command accepts and define acceptable ranges/constraints:

```rust
// Example: A command that sets audio volume
#[tauri::command]
async fn set_volume(
    volume: f32,  // 0.0 to 1.0
    app: AppHandle,
) -> Result<String, String> {
    // ...
}
```

### 2. Re-use Validators or Add New Ones

Check if a suitable validator exists in `src-tauri/src/validation.rs`. If not, add a new validator:

```rust
/// Validate volume (0.0 to 1.0)
pub fn validate_volume(volume: f32) -> Result<f32, ValidationError> {
    if !(0.0..=1.0).contains(&volume) {
        return Err(ValidationError::InvalidRange(format!(
            "Volume must be between 0.0 and 1.0, got {}",
            volume
        )));
    }
    Ok(volume)
}
```

**Add unit tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_valid() {
        assert!(validate_volume(0.0).is_ok());
        assert!(validate_volume(0.5).is_ok());
        assert!(validate_volume(1.0).is_ok());
    }

    #[test]
    fn test_volume_invalid() {
        assert!(validate_volume(-0.1).is_err());
        assert!(validate_volume(1.1).is_err());
    }
}
```

### 3. Emit `audio:error` on Validation Failure

Import and use `emit_validation_error` to send error events to the frontend:

```rust
use validation::{emit_validation_error, validate_volume};

#[tauri::command]
async fn set_volume(
    volume: f32,
    app: AppHandle,
) -> Result<String, String> {
    // Validate input
    if let Err(e) = validate_volume(volume) {
        emit_validation_error(
            &app,
            "invalid_volume",
            "volume",
            &e.to_string(),
            Some(serde_json::json!(volume)),
        );
        return Err(e.to_string());
    }

    // Rest of command logic...
    Ok("Volume set successfully".to_string())
}
```

### 4. Update i18n Translations

Add localized error messages in `src/lib/i18n/translations.ts`:

```typescript
// In Translations interface
errors: {
  // ... existing errors
  invalid_volume: string;
}

// In English translations
errors: {
  // ... existing errors
  invalid_volume: "Volume out of range (0.0–1.0).",
}

// In Spanish translations
errors: {
  // ... existing errors
  invalid_volume: "Volumen fuera de rango (0.0–1.0).",
}
```

### 5. Update Documentation

Extend `docs/VALIDATION_MATRIX.md` with a row for the new command:

```markdown
| **set_volume** | `volume: f32` | `validate_volume` | `invalid_volume` |
```

### 6. Ensure CI Passes

Run the validation tests to ensure they pass:

```bash
# Quick validation test (runs in CI)
cargo test -q --manifest-path src-tauri/Cargo.toml validation

# Full test suite
cargo test --manifest-path src-tauri/Cargo.toml
npm run test
```

---

## Testing Guidelines

### Unit Tests

- Write tests for all validators
- Test valid boundary values (min, mid, max)
- Test invalid values (below min, above max, edge cases)
- Use descriptive test names (e.g., `test_volume_valid`, `test_volume_invalid`)

### Property-Based Tests

For complex validators, add property-based tests using `proptest`:

```rust
#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn volume_in_valid_range(x in 0.0f32..1.0) {
            assert!(validate_volume(x).is_ok());
        }

        #[test]
        fn volume_outside_valid_range(x in any::<f32>().prop_filter("out of [0,1]", |v| *v < 0.0 || *v > 1.0)) {
            assert!(validate_volume(x).is_err());
        }
    }
}
```

### Integration Tests

For UI changes, ensure:
- Components render without errors
- Error toasts display correctly
- Localization works (test both ES and EN)

---

## Pull Request Process

1. **Update documentation** if you've changed APIs or added features
2. **Ensure all tests pass** locally before pushing
3. **Follow commit message conventions:**
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation changes
   - `refactor:` for code refactoring
   - `test:` for adding tests
   - `chore:` for maintenance tasks

4. **Wait for CI checks** to pass on your PR
5. **Request review** from maintainers
6. **Address feedback** and push updates

---

## Code Style

### Rust

- Follow Rust standard style (`cargo fmt`)
- Use `clippy` to catch common mistakes (`cargo clippy`)
- Prefer explicit types over `auto` where clarity is needed
- Document public functions with `///` doc comments

### TypeScript/React

- Use ESLint configuration (`npm run lint`)
- Prefer functional components and hooks
- Use TypeScript strict mode
- Avoid `any` types when possible

### Commit Messages

- Use present tense ("Add feature" not "Added feature")
- Use imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit first line to 72 characters
- Reference issues and PRs where appropriate

---

## Security

If you discover a security vulnerability:

- **Do NOT open a public GitHub issue**
- Contact maintainers privately via security@lotusemberlabs.com (or DM)
- See [SECURITY.md](docs/SECURITY.md) for full policy

---

## License

By contributing to Emberleaf, you agree that your contributions will be licensed under the same license as the project.

---

**Thank you for contributing to Emberleaf!** 🌿
