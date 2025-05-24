# Roadmap for ch-query-analyzer

## ✅ Already Implemented
- **Multi-node input** support (analyze multiple ClickHouse nodes at once)
- **Output formats**: Text, JSON, YAML
- **Top N queries**: default sorted by weight, group by `normalized_query_hash`
- **Flexible sorting**: top queries can be sorted by any field (e.g. `weight`, `cpu_time`, etc.)
- **Context/profile system**:
  - Named profiles (`dev`, `prod`, …) stored in `~/.config/ch-query-analyzer/config.toml`
  - Each profile has:
    - Multiple `urls` for the cluster
    - `user` and `password`
  - `ch-query-analyzer context set profile <NAME> …` to create/update
  - `ch-query-analyzer context set current <NAME>` to choose default
  - `--context <NAME>` override per-command
- **Error-log Analysis (`errors` command)**
  - Aggregate system.errors by code with counts, last time, message
  - Filters: --last, --min-count, --code, --remote-only
  - Display top errors in Text/JSON/YAML

## 🚀 Planned & Suggested Features

### “Get” & “Stats” Subcommands on queries
- ch-query-analyzer queries get <QUERY_ID>
  - Fetch single system.query_log row by query_id
  - Show full SQL, ProfileEvents, per-query Settings

- ch-query-analyzer queries stats [--date X] [--user Y] [--failed]
  - Single aggregated query: total count, unique queries, success vs failed, I/O & memory summaries


### Smart Filters
- Filter by **database** or **user**
- Time windows (`--from`/`--to`, `--last 1h`)
- Thresholds: `--min-duration`, `--min-rows`, `--min-bytes`, error patterns

### Specialized Weights
- **CPU/Memory score**: weight = f(cpu_time, memory_usage)
- **I/O score**: weight = f(read_bytes, written_bytes)
- **Custom composite**: user-defined formulas (e.g. `2×IO + 1×CPU + 0.5×Memory`)

### Advanced Analysis Modes
- **Unstable queries**: high-variance detection
- **Burst detection**: spikes in query frequency
- **Anomaly detection**: statistical outliers, unusual patterns

### Beyond Query Logs
- **Storage growth**: inspect `system.parts` to find largest tables/partitions and predict growth
- Merge spikes: analyze system.part_log for merge bursts

### Export Integrations
- Flamegraph integration: generate per-query flamegraphs or CPU profiles 
