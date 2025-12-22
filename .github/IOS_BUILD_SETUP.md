# iOS Build Setup Guide

Configure GitHub Actions for building iOS IPA files from your Tauri project.

## Workflows

| Workflow | File | Use Case |
|----------|------|----------|
| **iOS Build** | `ios-build.yml` | Standard builds, manual certificate management |
| **iOS Build (Fastlane)** | `ios-build-sparkfabrik.yml` | Team builds with Match certificate sharing |

## Quick Start (No Signing)

For simulator/debug builds without Apple Developer account:

1. Go to **Actions** → **iOS Build**
2. Click **Run workflow**
3. Select `debug` build type
4. Run - you'll get a `.app.zip` for iOS Simulator

## Full Setup (Signed Builds)

### Prerequisites

- Apple Developer Program membership ($99/year)
- macOS for certificate generation
- GitHub repository with Actions enabled

### Step 1: Create App ID

1. Go to [Apple Developer Portal](https://developer.apple.com/account/resources/identifiers/list)
2. Click **+** → **App IDs** → **App**
3. Enter:
   - Description: `iMAGE`
   - Bundle ID: `com.vortex.image` (Explicit)
4. Enable required capabilities
5. Click **Continue** → **Register**

### Step 2: Create Certificates

#### Development Certificate
```bash
# Generate Certificate Signing Request
openssl req -new -newkey rsa:2048 -nodes \
  -keyout ios_dev.key -out ios_dev.csr \
  -subj "/CN=iOS Development/O=Your Name"

# Upload CSR to Apple Developer Portal:
# Certificates → + → Apple Development
# Download the .cer file

# Convert to .p12
openssl x509 -in development.cer -inform DER -out development.pem
openssl pkcs12 -export -out development.p12 \
  -inkey ios_dev.key -in development.pem \
  -password pass:YOUR_PASSWORD
```

#### Distribution Certificate (for App Store)
Same process, select **Apple Distribution** certificate type.

### Step 3: Create Provisioning Profile

1. Go to **Profiles** → **+**
2. Select type:
   - **iOS App Development** - for testing
   - **Ad Hoc** - for limited distribution
   - **App Store Connect** - for TestFlight/App Store
3. Select your App ID
4. Select certificates
5. Select devices (Development/Ad Hoc only)
6. Download `.mobileprovision` file

### Step 4: Encode Secrets

```bash
# Certificate (required)
base64 -i development.p12 | pbcopy
# → Paste as BUILD_CERTIFICATE_BASE64

# Provisioning Profile (required for device builds)
base64 -i profile.mobileprovision | pbcopy
# → Paste as BUILD_PROVISION_PROFILE_BASE64
```

### Step 5: Add GitHub Secrets

Go to **Repository** → **Settings** → **Secrets and variables** → **Actions**

#### Required Secrets

| Secret | Description |
|--------|-------------|
| `BUILD_CERTIFICATE_BASE64` | Base64-encoded .p12 certificate |
| `P12_PASSWORD` | Password for .p12 file |
| `BUILD_PROVISION_PROFILE_BASE64` | Base64-encoded provisioning profile |
| `APPLE_TEAM_ID` | 10-character Team ID from Apple Developer Portal |

#### For TestFlight Upload

| Secret | Description |
|--------|-------------|
| `APPLE_API_KEY_ID` | App Store Connect API Key ID |
| `APPLE_API_ISSUER_ID` | API Key Issuer ID |
| `APPLE_API_KEY_CONTENT` | Contents of .p8 key file |

Create API Key at: **App Store Connect** → **Users and Access** → **Integrations** → **App Store Connect API**

### Step 6: Run Workflow

1. Go to **Actions** → **iOS Build**
2. Click **Run workflow**
3. Configure:
   - **Build type**: `release`
   - **Export method**: `development`, `ad-hoc`, or `app-store`
   - **Upload to TestFlight**: Check for App Store builds
4. Click **Run workflow**

## Fastlane Match Setup (Team Builds)

For teams sharing certificates via Git repository:

### 1. Create Match Repository

```bash
# Create private repo for certificates
gh repo create your-org/ios-certificates --private

# Initialize Match locally
fastlane match init
# Select: git
# Enter repo URL
```

### 2. Generate Certificates with Match

```bash
# Development
fastlane match development --app_identifier com.vortex.image

# Ad Hoc
fastlane match adhoc --app_identifier com.vortex.image

# App Store
fastlane match appstore --app_identifier com.vortex.image
```

### 3. Configure Secrets

| Secret | Description |
|--------|-------------|
| `MATCH_GIT_URL` | `https://github.com/your-org/ios-certificates.git` |
| `MATCH_GIT_BASIC_AUTH` | `echo -n "username:token" \| base64` |
| `MATCH_PASSWORD` | Password set during `match init` |
| `APPLE_TEAM_NAME` | Your team name |

### 4. Enable Sparkfabrik

Set repository variable:
- **Name**: `USE_SPARKFABRIK`
- **Value**: `true`

## Export Methods

| Method | Use Case | Requirements |
|--------|----------|--------------|
| `development` | Testing on registered devices | Development cert + profile |
| `ad-hoc` | Distribution to specific devices (100 max) | Distribution cert + Ad Hoc profile |
| `app-store` | TestFlight / App Store | Distribution cert + App Store profile |

## Troubleshooting

### "No signing certificate found"
- Verify `BUILD_CERTIFICATE_BASE64` is correctly encoded
- Check certificate hasn't expired
- Ensure certificate type matches export method

### "Provisioning profile doesn't match"
- Bundle ID must be `com.vortex.image`
- Profile must include the signing certificate
- For device builds, device UDID must be in profile

### "Code signing is required"
- Simulator builds don't need signing
- Device builds require valid certificate + profile

### Build fails with Rust errors
- Ensure iOS targets are installed: `rustup target add aarch64-apple-ios`
- Check Cargo.toml for iOS-incompatible dependencies

### Sparkfabrik action fails
- Verify Match repository is accessible
- Check `MATCH_PASSWORD` matches what was set during init
- Ensure certificates exist in Match repo for the export method

## Local Development

```bash
# Initialize iOS project
pnpm tauri ios init

# Build for simulator (no signing)
pnpm tauri ios build --debug

# Build for device (requires signing)
pnpm tauri ios build

# Run on simulator
pnpm tauri ios dev

# Run on connected device
pnpm tauri ios dev --device
```

## CI/CD Best Practices

1. **Use debug builds for PRs** - faster, no signing needed
2. **Use release builds for tags** - automatic TestFlight upload
3. **Rotate certificates annually** - Apple certificates expire after 1 year
4. **Use Match for teams** - simplifies certificate management
5. **Cache dependencies** - speeds up builds significantly

## Workflow Triggers

| Trigger | Build Type | TestFlight |
|---------|------------|------------|
| Push to `main` | debug | No |
| Push to `release` | debug | No |
| Tag `v*` | release (app-store) | Yes |
| Pull Request | debug | No |
| Manual | configurable | configurable |
