# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `context delete <PROFILE_NAME>` command to delete context profile.
- `Network Impact` for `queries` command.
- `total` command which calculate total weights by filters.
- `inspect <FINGERPRINT>` command which show detail metrics information about query group.

### Deprecated
- Remove ability sort by non impact columns in queries command.

## [0.2.1] - 2025-06-22

### Changed
- Improve documentation.

### Fixed
- Enable `keyring` on linux and windows.

## [0.2.0] - 2025-06-15

### Added
- Time window support for `queries` command via `--last` duration flag (e.g., `--last 1h`)
- Threshold filters for `queries` command:
  - `--min-query-duration` (e.g., `100ms`)
  - `--min-read-rows` (e.g., `1000`)
  - `--min-read-data` (e.g., `1000000`)
- New filters for `queries` command:
  - `--user` to filter by user accounts
  - `--database` to filter by database names
  - `--table` to filter by table names
- Added specialized impact metrics for detailed analysis:
  - `cpu_impact` (CPU time × 10k)
  - `memory_impact` (memory usage × 10)
  - `io_impact` (rows × 100 + bytes × 1)
  - `time_impact` (duration × 1M)
- New `context config-path` command to display configuration file location

### Changed
- **Metrics rework in `queries` command**:
  - Renamed `weight` metric to `total_impact` for clarity
  - Moved impact calculation logic to dedicated `QueryMetrics` model
  - Improved human-readable formatting for all impact values
- **Flag validation rules**:
  - `--last` is now mutually exclusive with `--from`
  - `--from` or `--last` is required
  - `--sort-by` now accepts all impact types
- **BREAKING CHANGE**: `accept_invalid_certificate` field in profiles is now required (changed from `Option<bool>` to `bool`)
  - Existing configs must add `accept_invalid_certificate = true/false` to each profile

### Deprecated
- `weight` field in JSON output (use `total_impact`)
- `--sort-by weight` (use `--sort-by total_impact`)

### Fixed
- **Time zone handling in query filters**: 
  - Fixed incorrect time filtering by explicitly specifying UTC timezone in DateTime conversions
  - Changed from: `event_time >= ?` (implicit timezone)
  - Changed to: `event_time >= toDateTime(?, 'UTC')` (explicit UTC)
  - Impact: Queries now correctly filter by absolute time ranges regardless of server timezone settings
  - Affected commands: All commands using time filters (`queries`, `errors`)
- Improved error messages for malformed configuration files

### Security
- **Passwords now stored in encrypted system storage** (Keychain/Secret Service/Credential Manager)  
- All passwords handled via `secrecy::Secret<String>` for:
  - Automatic memory zeroization
  - Protection against accidental logging
  - Explicit access control
- Removed password storage in config files (`#[serde(skip)]`)
- Enhanced security for `context show` command
  - Passwords are now masked by default (`[REDACTED]`)
  - Requires new `--show-secrets` flag to reveal passwords
- New `--interactive-password` flag to explicitly request password input via secure prompt

## [0.1.0] - 2025-05-31

### Added
- **Core CLI Framework**:
  - Base command structure with `queries`, `errors`, and `context` subcommands
  - Multi-node support (multiple URLs)
  - Support for `--config`, `--context`, and `--out` global options
  - Output formats: Text, JSON, YAML

- **Profile Management**:
  - Context/profile system for managing ClickHouse cluster connections
  - Context commands:
    - `context list` - Show available profiles
    - `context current` - Display active context
    - `context show <name>` - Show profile details
    - `context set profile <NAME>` - Create/update profiles
    - `context set current <NAME>` - Set default context

- **Query Analysis**:
  - `queries` command for analyzing ClickHouse query_log
  - Grouping by `normalized_query_hash` with weight calculation
  - Flexible sorting via `--sort-by` option (weight, cpu-time, query-duration, etc.)
  - Time filtering with `--from` and `--to`

- **Error Monitoring**:
  - `errors` command for analyzing system.errors
  - Aggregation by error code with counts and last occurrence
  - Filters:
    - `--last` - Time-based filtering
    - `--min-count` - Minimum occurrence threshold
    - `--code` - Filter by specific error codes

- **Connection Options**:
  - `--accept-invalid-certificate` for development environments
  - URLs, user, and password parameters for direct connection
  - URLs, user and password parameters can be combined with profile (context) system
