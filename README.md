# Roadmap for ch-query-analyzer

## ✅ Already Implemented
- **Multi-node input** support (analyzing multiple ClickHouse nodes at once)
- **Output in different formats:** Text, JSON, YAML
- **Top N queries output:** Default sorted by weight
- **Flexible sorting:** Top queries can be sorted by any field (e.g., `weight`, `cpu_time`, etc.)

## 🚀 Planned and Suggested Features

### Smart Filters
- **Filter by database or user:** Focus on queries from a particular database or user.
- **Slow query filter:** Filter queries by minimum execution time (`query_duration_ms > X`).
- **I/O or data volume filter:** Filter by `read_bytes`, `written_bytes`, etc., to find heavy I/O queries.
- **Resource consumption filters:** Filter queries by memory usage, CPU time, and error types.

### Specialized Query Weights
- **CPU/Memory heavy score:** Create a weight based on CPU and memory usage.
- **I/O heavy score:** Create a weight based on data read/written volumes.
- **Custom composite scores:** Allow users to define custom scoring (e.g., `2x IO + 1x CPU + 0.5x Memory`).

### Advanced Analysis
- **Unstable (high-variance) queries:** Detect queries with high runtime variability.
- **Query burst detection:** Identify sudden spikes in query frequency.
- **Anomaly detection:** Automatically detect outlier queries based on statistical models.

### Real-time Monitoring and Dashboards
- **Grafana/Prometheus integration:** Export metrics for real-time dashboards and alerts.
- **Live streaming mode:** `tail -f`-like mode to watch incoming queries and highlight anomalies.

### Export & Integration Features
- **Prometheus metrics export:** Export aggregated results in a Prometheus-compatible format.
- **CSV/JSON exports:** Save reports for external processing or dashboards.

### CLI UX Enhancements
- **Interactive mode:** A guided interactive CLI where users can select options step-by-step.
- **Colored and formatted output:** Use colors (e.g., red for slow queries) and align text into tables.
- **Autocompletion and command history:** Improve usability with shell autocompletion and recent command history.

### 📝 Add context/profile system for connection management

Implement a kube-like context system to manage and switch between multiple ClickHouse cluster profiles.

- [ ] Support multiple named profiles (`dev`, `prod`, etc.) stored in a config file (e.g. `~/.config/ch-query-analyzer/config.toml`).
- [ ] Each profile can contain:
  - Multiple `urls` for ClickHouse cluster nodes.
  - `user` and `password` credentials.
- [ ] Allow setting the current active context (e.g., `ch-query-analyzer context use dev`).
- [ ] Add commands to list, view, and switch contexts:
  - `context use <name>`
  - `context current`
  - `context list`
- [ ] Optional: Add `login` command to create a new profile and optionally set it as current.
- [ ] Ensure profile usage integrates cleanly with existing `CliArgs`.

---

These features aim to make **ch-query-analyzer** not just a query log viewer, but a powerful tool for:
- Developers — to debug slow or unstable queries
- DBAs — to optimize system performance and monitor resource usage
- SREs — to detect anomalies, load spikes, and prepare alerts

# TODO

```
ch-query-analyzer top queries --limit 10 --sort-by read_rows --group-by table --out json
ch-query-analyzer top tables  --limit 10 --sort-by read_rows          --out text
ch-query-analyzer top users   --limit 10 --sort-by cpu_time            --out yaml

ch-query-analyzer unstable --threshold 0.7 --out json
ch-query-analyzer burst   --window-secs 30 --out text
```

