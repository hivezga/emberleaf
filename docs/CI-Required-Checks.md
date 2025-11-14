# Required Checks (Branch Protection)

To block merges unless key CI jobs pass:

1) GitHub Settings → **Branches** → Branch protection rules
2) Edit the rule for `main` (or create one)
3) Enable **Require status checks to pass before merging**
4) Add these checks:
   - `QA KWS E2E (PipeWire Loopback)`   # qa-kws-e2e-loopback
   - `Test (Frontend)`                  # test-web
   - `Check (Rust)`                     # check-rust
   - `Lint (TypeScript + Rust)`         # lint
   - (optional) `Quick Validation (Rust unit subset)`  # quick-validation

### CLI (gh) example

```bash
# Requires repository admin permission.
# Creates/updates protection on 'main' with required checks.
gh api \
  -X PUT \
  repos/:owner/:repo/branches/main/protection \
  -f required_status_checks.strict=true \
  -f required_status_checks.enforce_admins=true \
  -f enforce_admins=true \
  -H "Accept: application/vnd.github+json" \
  --input - <<'JSON'
{
  "required_status_checks": {
    "strict": true,
    "checks": [
      {"context": "QA KWS E2E (PipeWire Loopback)"},
      {"context": "Test (Frontend)"},
      {"context": "Check (Rust)"},
      {"context": "Lint (TypeScript + Rust)"},
      {"context": "Quick Validation (Rust unit subset)"}
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": null,
  "restrictions": null
}
JSON
```

> If the job names differ in your workflow UI, match the **exact** check names shown on PRs.

## Verifying Required Checks

After setting up branch protection:

1. Create a test PR from a feature branch
2. Navigate to the PR page
3. Scroll to the merge section at the bottom
4. You should see: **"Merging is blocked — Some checks haven't completed yet"**
5. The required checks will be listed with their status

## Temporarily Bypassing (Admin Only)

If you have admin permissions and need to bypass:

1. Use the **"Merge without waiting for requirements to be met"** option (admin override)
2. Or temporarily disable the branch protection rule

**⚠️ Not recommended for production branches.**

## Troubleshooting

**"Cannot find check with name X"**:
- The check name must match the **job name** in your workflow YAML exactly
- Check the Actions tab on a recent PR to see the exact job names
- Job names are defined by `jobs.<job_id>.name` in `.github/workflows/ci.yml`

**"Required checks don't appear on PRs"**:
- Ensure the workflow runs on `pull_request` events for the target branch
- Check the workflow's `on:` trigger configuration

**"Checks pass but merge still blocked"**:
- You may have other branch protection rules enabled (reviews, signed commits, etc.)
- Check all enabled protection rules in Settings → Branches
