# Prometheus Textfile Collector – Puppet Agent

Aka `ptc-puppet-agent`

The goal of this software is to export the current status of the local `puppet-agent` to Prometheus.

**Example output**:
```txt
# HELP puppet_agent_collector_success Indicates whether the collector was successful or encountered problems
# TYPE puppet_agent_collector_success gauge
puppet_agent_collector_success 1
# HELP puppet_agent_enabled Whether the agent is enabled (allowed to run)
# TYPE puppet_agent_enabled gauge
puppet_agent_enabled{message=""} 1
# HELP puppet_agent_info Information about the agent and the environment
# TYPE puppet_agent_info gauge
puppet_agent_info{environment="my_precious_prod",version="7.34.0"} 1
# HELP puppet_agent_run_events_failed Number of events in the most recent run that failed
# TYPE puppet_agent_run_events_failed gauge
puppet_agent_run_events_failed 0
# HELP puppet_agent_run_events_noop Number of events if the most resent run was a NOOP run
# TYPE puppet_agent_run_events_noop gauge
puppet_agent_run_events_noop 0
# HELP puppet_agent_run_resources_failed Amount of failed resources in the most recent run
# TYPE puppet_agent_run_resources_failed gauge
puppet_agent_run_resources_failed 0
# HELP puppet_agent_run_resources_failed_to_restart Amount of resources in the most recent run that failed their restart-action
# TYPE puppet_agent_run_resources_failed_to_restart gauge
puppet_agent_run_resources_failed_to_restart 0
# HELP puppet_agent_run_resources_total Total number of resources in the most recent run
# TYPE puppet_agent_run_resources_total gauge
puppet_agent_run_resources_total 605
# HELP puppet_agent_run_successful Whether the most recent run was a success
# TYPE puppet_agent_run_successful gauge
puppet_agent_run_successful 0
# HELP puppet_agent_run_time Timestamp of the most recent puppet run
# TYPE puppet_agent_run_time gauge
puppet_agent_run_time 1739997502
```

By default, it outputs the "disabled" message as part of the `puppet_agent_enabled`-metric. This should not cause considerable churn unless you constantly disable your agents with varying messages on thousands of machines. Disable with `--no-disabled-message`.

## Quick start

1. Install a recent version of Rust (either through your OS or head to https://rustup.rs/ and follow the instructions)
2. Clone the repository and change into the newly created folder (`git clone https://github.com/svalouch/ptc-puppet-agent; cd ptc-puppet-agent`)
3. Compile the binary (`cargo build --release`), this creates the binary: `target/release/ptc-puppet-agent`
4. Copy the binary to a machine that is managed by Puppet
5. Check if it works: `sudo ./ptc-puppet-agent --stdout` (if your system matches the defaults, it should print metrics. If not or you aren't running it as root, red error messages should give a hint at the problem(s).
6. Schedule for it to be run periodically. A 5 minute timer is a good fit if you run puppet-agent once per hour.

For more in-depth info, read the `INSTALL.md` next to this file and also check the `contrib/` directory.

## Running

Have a look at the files in the `contrib/` directory for examples of what is discussed here. The quick rundown of the normal operation is as follows:

1. Put the binary onto your system (see `INSTALL.md` and the `contrib/` directory).
2. Configure some form of periodic job, for example in 5 minutes intervals
3. Use `prometheus-node-exporter` to fetch the output from the binary.
4. Use alert rules and dashboards to stay up-to-date.

In case there is a fatal error, the software will terminate with a non-zero return-code. If it is run with a systemd service+timer-combination, you can use the `systemd`-collector offered by `prometheus-node-exporter` to get word of it (enable unit state export, possibly limiting it to services named `ptc-.*` to limit the output volume).

Example help output:
```txt
Prometheus Textfile Collector – Puppet Agent

Exports whether puppet-agent has been disabled locally (via e.g. "puppet agent --disable
[message]") and some metrics about the most recent run by examining the last run summary and
run report files.

To debug it, set the environment variable "PTC_LOG" to "debug" to make its output more verbose.
Use the "--stdout" switch to make it print instead of writing to the filesystem.

Usage: ptc-puppet-agent [OPTIONS]

Options:
  -o, --output-file <OUTPUT_FILE>
          Path to the file where the output will be written to (unless --stdout is set)

          [env: OUTPUT_FILE=]
          [default: /var/lib/prometheus/node-exporter/puppet-agent.prom]

  -a, --agent-disabled-lockfile <AGENT_DISABLED_LOCKFILE>
          Agent disable lockfile, its existence indicates whether the agent has been
          --disabled. The optional disabled-message is exported unless --no-disabled-message is
          set

          [env: AGENT_DISABLED_LOCKFILE=]
          [default: /opt/puppetlabs/puppet/cache/state/agent_disabled.lock]

  -r, --lastrunfile <LASTRUNFILE>
          Last run summary with metrics on the success of the most recent run

          [env: LASTRUNFILE=]
          [default: /opt/puppetlabs/puppet/public/last_run_summary.yaml]

  -l, --lastrunreport <LASTRUNREPORT>
          Last run report with additional metrics for cached catalog status

          [env: LASTRUNREPORT=]
          [default: /opt/puppetlabs/puppet/cache/state/last_run_report.yaml]

  -n, --no-disabled-message
          Do not include the disabled-message as label value

          [env: NO_DISABLED_MESSAGE=]

  -s, --stdout
          Print to stdout instead of the output-file

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

The `--help`-output shows how to overwrite the defaults both on the command line and via environment variables (which you could set using a systemd dropin file, for example). Some examples can be found in the `contrib/`-directory.

## Dependencies

**Linux**

The software is developed and used on Linux, `x86_64` to be precise. It may run on BSDs and other operating systems or architectures just fine, but that is not the goal and your mileage may wary. If code-changes are required to make it run on other platforms, feel free to submit a patch and we might consider it if it isn't too intrusive to the code or creates a huge maintenance burden. Defaults will likely not be touched, as they can be overwritten easily.

**Puppet-Agent**

The software has been written with the official `puppet-agent` package from PuppetLabs in mind, specifically version 7 and newer. This mainly concerns the defaults: if this package is used, none of the paths that direct the collector where to find the output files have to be touched. The format of the files should™ be fairly stable and only a subset required for the software to work is actually parsed. Other implementations should work just fine as long as they don't touch the output format in a way incompatible to the collector's assumptions.

**Prometheus Node Exporter**

A soft dependency, but pretty much the accepted way to get metrics from a system without writing yet another exporter that runs as a daemon is to use `prometheus-node-exporter` and enable the `textfile-collector`. The defaults assume the output directory to be the same as on the Debian Bookworm package for the node-exporter, which is `/var/lib/prometheus/textfile-collector/`.

## Limitations

**Root**: Unfortunately, the way the `puppet-agent` works does not allow unprivileged accounts to read the files necessary for consistently figuring out if the most recent run has been successful or not. In particular, the `last_run_report.yaml`-file contains potentially sensitive information such as diffs of changed file-resources, which may contain credentials. There are multiple failure-scenarios the author of the software considers to be a failed run, in which the `puppet-agent` does **not** set the failure flag. Thus, both the `last_run_summary.yaml` (public and harmless) as well as the `last_run_report.yaml` have to be examined. The Puppet configuration reference makes hints at a way to change the owner of the `last_run_summary.yaml`, but this is plainly wrong and the code does something different behind the scenes.

## Troubleshooting

First, make sure it is run as root to be able to read the puppet-agent's output files like `last_run_report.yaml`.

The fastest way is figure out what's wrong is to run it on the terminal with debug output enabled: `sudo PTC_LOG=debug /usr/libexec/prometheus-node-exporter-collectors/ptc-puppet-agent --stdout`

This will print a number of debug-level lines, first telling you where it searches for files and then how it's chugging along. The `--stdout`-switch tells it to print the metrics instead of dumping them to the output-file, thus eliminating one source for errors. The output is colourized to denote the different log levels.

Example (happy case):
```txt
2025-02-19T21:38:56.575571Z DEBUG ptc_puppet_agent: Output file is set to "/var/lib/prometheus/node-exporter/puppet-agent.prom"
2025-02-19T21:38:56.575633Z DEBUG ptc_puppet_agent: 'lastrunfile' is set to "/opt/puppetlabs/puppet/public/last_run_summary.yaml"
2025-02-19T21:38:56.575647Z DEBUG ptc_puppet_agent: 'lastrunreport' is set to "/opt/puppetlabs/puppet/cache/state/last_run_report.yaml"
2025-02-19T21:38:56.575658Z DEBUG ptc_puppet_agent: 'agent_disabled_lockfile' is set to "/opt/puppetlabs/puppet/cache/state/agent_disabled.lock"
2025-02-19T21:38:56.575691Z DEBUG ptc_puppet_agent: Lockfile does not exist
2025-02-19T21:38:56.575703Z DEBUG ptc_puppet_agent: Agent is disabled: false
2025-02-19T21:38:56.575748Z DEBUG ptc_puppet_agent: Parsing lastrunfile: /opt/puppetlabs/puppet/public/last_run_summary.yaml
2025-02-19T21:38:56.576364Z DEBUG ptc_puppet_agent: Parsing lastrunreport: /opt/puppetlabs/puppet/cache/state/last_run_report.yaml
# HELP puppet_agent_collector_success Indicates whether the collector was successful or encountered problems
# TYPE puppet_agent_collector_success gauge
puppet_agent_collector_success 1
[more metrics]
```

Example where the lastrunfile was not found:
```txt
2025-02-19T21:40:33.416297Z DEBUG ptc_puppet_agent: Output file is set to "/var/lib/prometheus/node-exporter/puppet-agent.prom"
2025-02-19T21:40:33.416365Z DEBUG ptc_puppet_agent: 'lastrunfile' is set to "/i-do-not-exist"
2025-02-19T21:40:33.416378Z DEBUG ptc_puppet_agent: 'lastrunreport' is set to "/opt/puppetlabs/puppet/cache/state/last_run_report.yaml"
2025-02-19T21:40:33.416389Z DEBUG ptc_puppet_agent: 'agent_disabled_lockfile' is set to "/opt/puppetlabs/puppet/cache/state/agent_disabled.lock"
2025-02-19T21:40:33.416423Z DEBUG ptc_puppet_agent: Lockfile does not exist
2025-02-19T21:40:33.416435Z DEBUG ptc_puppet_agent: Agent is disabled: false
2025-02-19T21:40:33.416477Z DEBUG ptc_puppet_agent: Parsing lastrunfile: /i-do-not-exist
2025-02-19T21:40:33.416512Z ERROR ptc_puppet_agent: Parsing lastrunfile /i-do-not-exist failed: File /i-do-not-exist does not exist
# HELP puppet_agent_collector_success Indicates whether the collector was successful or encountered problems
# TYPE puppet_agent_collector_success gauge
puppet_agent_collector_success 0
# HELP puppet_agent_enabled Whether the agent is enabled (allowed to run)
# TYPE puppet_agent_enabled gauge
puppet_agent_enabled{message=""} 1
```

As can be seen, it has a metric `puppet_agent_collector_success` to tell it has run into problems, but there are cases (e.g. the output file can't be written) where it can not update the metric. If it fails hard, it exits on-zero, which systemd should pick up (`systemctl --failed`).

If the error is in the past and the tool is run using the systemd unit (see `INSTALL.md`), you can check the journal: `journalctl -xu ptc-puppet-agent.service`

## Disclaimer

Neither the project nor its authors are in any way associated with PuppetLabs. Do not ask them for help with this software!

Copyright (c) 2025 Stefan Valouch

