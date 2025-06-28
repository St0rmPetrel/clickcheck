# 🎯 Roadmap

## В разработке и планах

- “Get” & “Stats” Subcommands on queries
  - `clickcheck queries get <FINGERPRINT>`
  - `clickcheck queries stats`: Single aggregated query
- Advanced Analysis Modes
  - **Unstable queries**: high-variance detection
  - **Burst detection**: spikes in query frequency
  - **Anomaly detection**: statistical outliers, unusual patterns
- Beyond Query Logs
  - Check readonly tables
  - **Storage growth**: inspect `system.parts` to find largest tables/partitions and predict growth
  - Merge spikes: analyze system.part_log for merge bursts
- Export Integrations
  - Flamegraph integration:
    - Generate per-query flamegraphs or CPU profiles 
    - Generate memory flamegrapht: database -> table -> column/partition
