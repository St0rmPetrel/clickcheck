# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Time window support for `queries` command via `--last` duration flag (e.g., `--last 1h`)
- Threshold filters for `queries` command:
  - `--min-duration` (e.g., `100ms`)
  - `--min-rows` (e.g., `1000`)
  - `--min-bytes` (e.g., `1000000`)
- New filters for `queries` command:
  - `--user` to filter by user accounts
  - `--database` to filter by database names
  - `--table` to filter by table names

### Changed
- **Metrics rework in `queries` command**:
  - Renamed `weight` metric to `general_weight` for clarity
  - Moved `weight` calculation logic to `QueryLog` model
  - Improved human-readable formatting for `weight` values
- **Flag validation rules**:
  - `--from` now requires `--to` when used
  - `--last` is now mutually exclusive with `--from`/`--to`

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
