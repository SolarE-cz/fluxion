# FluxION - Home Assistant Addon Restructuring Summary

This document summarizes the restructuring performed to conform with the
[Home Assistant Example Addon](https://github.com/hassio-addons/addon-example) structure.

## Changes Made

### 1. Directory Structure ✅

- **Renamed**: `addon/` → `fluxion/` (addon name matches directory)
- The addon directory is now named after the addon slug

### 2. Documentation ✅

- **Created**: `fluxion/DOCS.md` - Comprehensive user documentation
  - Installation instructions
  - Complete configuration options with descriptions
  - Usage examples
  - Support information

### 3. Translations ✅

- **Created**: `fluxion/translations/` directory
- **Created**: `fluxion/translations/en.yaml` - English UI translations
- **Created**: `fluxion/translations/cs.yaml` - Czech UI translations
- These provide translated configuration option names and descriptions in the Home Assistant UI

### 4. S6 Overlay Service Structure ✅

- **Converted**: From old `services.d` to modern `s6-overlay/s6-rc.d` structure
- **Created**: S6 overlay service hierarchy:
  ```
  fluxion/rootfs/etc/s6-overlay/s6-rc.d/
  ├── init-fluxion/          # Initialization service (oneshot)
  │   ├── run
  │   ├── type
  │   ├── up
  │   └── dependencies.d/
  │       └── base
  ├── fluxion/               # Main service (longrun)
  │   ├── run
  │   ├── type
  │   └── dependencies.d/
  │       └── init-fluxion
  └── user/
      └── contents.d/
          ├── init-fluxion
          └── fluxion
  ```
- **Created**: `fluxion/rootfs/usr/bin/fluxion.sh` - Main execution script
- **Removed**: Old `services.d` directory

### 5. Dockerfile Updates ✅

- **Updated**: `fluxion/Dockerfile` to match example structure:
  - Uses `ghcr.io/hassio-addons/base:18.2.1` as base image
  - Multi-stage build with named builder stage
  - Pinned package versions for reproducibility
  - Proper BUILD\_\* ARG variables for metadata
  - Complete OCI image labels
  - Copies rootfs structure properly

### 6. GitHub Community Files ✅

- **Created**: `.github/CODE_OF_CONDUCT.md` - Community conduct guidelines
- **Created**: `.github/CONTRIBUTING.md` - Contribution guidelines
- **Created**: `.github/workflows/` directory (ready for CI/CD workflows)

### 7. Repository README ✅

- **Updated**: Root `README.md` to match addon repository structure
  - Added architecture badges (aarch64, amd64, armv7)
  - Added release and project stage shields
  - Reorganized to focus on addon usage
  - Added proper badge references
  - Added support section

### 8. Icon and Logo ⚠️ TODO

- **TODO**: Create `fluxion/icon.png` (256x256px)
- **TODO**: Create `fluxion/logo.png` (512x512px)
- **Created**: `fluxion/ICON_TODO.md` with instructions

## File Structure Overview

```
fluxion/                           # Root repository directory
├── .github/
│   ├── CODE_OF_CONDUCT.md
│   ├── CONTRIBUTING.md
│   └── workflows/                 # Ready for CI/CD workflows
├── fluxion/                       # Addon directory (renamed from addon/)
│   ├── build.yaml                 # Build configuration
│   ├── config.yaml                # Addon configuration & options
│   ├── Dockerfile                 # Multi-stage build with proper labels
│   ├── DOCS.md                    # User documentation
│   ├── README.md                  # Addon info (existing)
│   ├── ICON_TODO.md              # Instructions for icon/logo
│   ├── rootfs/                    # Root filesystem overlay
│   │   ├── etc/
│   │   │   └── s6-overlay/
│   │   │       └── s6-rc.d/       # S6 overlay services
│   │   │           ├── init-fluxion/
│   │   │           ├── fluxion/
│   │   │           └── user/
│   │   └── usr/
│   │       └── bin/
│   │           └── fluxion.sh     # Main execution script
│   └── translations/              # UI translations
│       ├── en.yaml                # English
│       └── cs.yaml                # Czech
├── crates/                        # Rust workspace crates
├── docs/                          # Project documentation
├── README.md                      # Repository README (updated)
└── ...other project files
```

## What's Different from the Example?

### Similarities ✅

1. Directory structure matches (addon name = directory name)
2. S6 overlay service structure
3. DOCS.md for user documentation
4. translations/ directory with language files
5. Dockerfile with proper labels and build args
6. GitHub community files

### FluxION-Specific Differences

1. **Multi-stage Rust build** instead of simple shell scripts
2. **Workspace structure** with multiple Rust crates
3. **More complex runtime** (Rust binary instead of bash scripts)
4. **Advanced configuration** schema with nested options
5. **Multiple architectures** support (aarch64, amd64, armv7)

## Next Steps

### Immediate Actions Needed

1. **Create icon.png and logo.png** (see `fluxion/ICON_TODO.md`)
2. **Update repository URLs** in:
   - `fluxion/config.yaml` (url field)
   - `fluxion/DOCS.md` (issue links)
   - `README.md` (badge links)
3. **Test the addon build**:
   ```bash
   docker build -t fluxion:test -f fluxion/Dockerfile .
   ```

### Optional Enhancements

1. **Add GitHub Actions workflows** in `.github/workflows/`:
   - CI (build, test, lint)
   - Release automation
   - Container registry publishing
2. **Add issue/PR templates** in `.github/`
3. **Add CHANGELOG.md** for version tracking
4. **Create repository.json** for addon repository hosting

## Testing the Restructured Addon

1. Build the Docker image:

   ```bash
   docker build -t fluxion:test -f fluxion/Dockerfile .
   ```

2. Test the addon in Home Assistant:

   - Copy the `fluxion/` directory to your Home Assistant addons folder
   - Restart Home Assistant
   - Install and configure the addon

## References

- [Home Assistant Example Addon](https://github.com/hassio-addons/addon-example)
- [Home Assistant Add-on Documentation](https://developers.home-assistant.io/docs/add-ons)
- [S6 Overlay Documentation](https://github.com/just-containers/s6-overlay)
- [Bashio Documentation](https://github.com/hassio-addons/bashio)

## Notes

- The restructuring maintains full backward compatibility with existing Rust code
- All Rust crates remain unchanged
- The addon now follows Home Assistant best practices
- Multi-language support (EN, CZ) is properly integrated
- Debug mode and all configuration options are preserved
