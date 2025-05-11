# Roadmap for ch-query-analyzer

## ✅ Already Implemented
- **Multi-node input** support (analyze multiple ClickHouse nodes at once)
- **Output formats**: Text, JSON, YAML
- **Top N queries**: default sorted by weight
- **Flexible sorting**: top queries can be sorted by any field (e.g. `weight`, `cpu_time`, etc.)
- **Context/profile system**:
  - Named profiles (`dev`, `prod`, …) stored in `~/.config/ch-query-analyzer/config.toml`
  - Each profile has:
    - Multiple `urls` for the cluster
    - `user` and `password`
  - `ch-query-analyzer context set profile <NAME> …` to create/update
  - `ch-query-analyzer context set current <NAME>` to choose default
  - `--context <NAME>` override per-command

## 🚀 Planned & Suggested Features

### Smart Filters
- Filter by **database** or **user**
- **Slow-query** filter: e.g. `query_duration_ms >= X`
- **I/O-volume** filter: `read_bytes`, `written_bytes`
- **Resource** filters: memory usage, CPU time, error codes

### Specialized Weights
- **CPU/Memory score**: weight = f(cpu_time, memory_usage)
- **I/O score**: weight = f(read_bytes, written_bytes)
- **Custom composite**: user-defined formulas (e.g. `2×IO + 1×CPU + 0.5×Memory`)

### Advanced Analysis Modes
- **Unstable queries**: high-variance detection
- **Burst detection**: spikes in query frequency
- **Anomaly detection**: statistical outliers, unusual patterns

### Beyond Query Logs
- **Error frequency**: analyze `system.errors` to find the most common errors
- **Storage growth**: inspect `system.parts` to find largest tables/partitions and predict growth

### Real-time & Integrations
- **Live-stream mode**: `tail -f` style streaming with alerts on anomalies
- **Prometheus export**: expose metrics for Grafana dashboards
- **CSV/JSON reports**: for external analysis

### CLI & UX
- **Interactive wizard**: step-by-step mode for novices
- **Colorized output**: highlight slow or error-prone queries
- **Shell completion**: bash/zsh/fish autocomplete
- **Subcommands**:
  ```bash
  ch-query-analyzer top queries --limit 10 --sort-by read_rows --group-by table --out json
  ch-query-analyzer top tables  --limit 10 --sort-by read_rows          --out text
  ch-query-analyzer top users   --limit 10 --sort-by cpu_time           --out yaml

  ch-query-analyzer unstable --threshold 0.7 --out json
  ch-query-analyzer burst   --window 30s     --out text
  ```
