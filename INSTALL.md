# Installation

This document gives a few hints for how to install the software.

The following assumes that you have a relatively recent Linux-distribution (because of the version of the Rust-compiler) and are familiar with using the terminal.

## Grab Rust

There are two main ways of obtaining Rust: Through your package manager (which may be too old, but is always the best way to start) and via RustUp.

Depending on the distribution, you can use a command like these examples:
- Debian and Ubuntu: `apt install rust`
- Arch: `pacman -S rustup` or `pacman -S rust cargo`
- Fedora: `dnf install rust`
- …

Otherwise, head over to https://rustup.rs and follow the instructions on how to install Rust.

Next, compile the binary. You can either use dynamic or static linking, pick the section that suits your needs best:

## Compile: Dynamic

The default way of compiling rust-binaries results in a binary that dynamically links to your operating system's standard library (libc). This is okay if you run only one (version) of an operating system or have a way of automatically building for the systems you wish to run this on (e.g. a build-farm at your workplace).

Simply issue `cargo build --release` and grab the binary from `target/release/ptc-puppet-agent` and run it.

In order to check whether it is indeed dynamically linked, you can use `LDD(1)` on it. For example, this is what it looks like on Arch:
```txt
$ ldd target/release/ptc-puppet-agent
	linux-vdso.so.1 (0x00007c0aa751a000)
	libgcc_s.so.1 => /usr/lib/libgcc_s.so.1 (0x00007c0aa709f000)
	libc.so.6 => /usr/lib/libc.so.6 (0x00007c0aa6ead000)
	/lib64/ld-linux-x86-64.so.2 => /usr/lib64/ld-linux-x86-64.so.2 (0x00007c0aa751c000)
```

## Compile: Static

Compiling static binaries is a bit more trouble, but the resulting binary can run on a large number of operating systems as long as they are not too dissimilar. Compile the binary once and distribute it to systems running Debian Buster, OracleLinux 8 or Arch Linux, and it should™ just work. Be sure to check out the files in `contrib/` for hints at packaging.

To create a _static_ binary, you need to use the `musl` libc, instead of `glibc`. By default, Rust installs the target `x86_64-unknown-linux-gnu`, but for this, we need `x86_64-unknown-linux-musl`. Naturally, the resulting binary is a bit larger in size.

1. Install the `musl` target:
  - For *rustup*-users, run `rustup target install x86_64-unknown-linux-musl`
  - Some distributions package musl, e.g. Arch: `pacman -S rust-musl`
2. On some distributions, you might have to install additional packages such as `build-essential` or a musl-package on top of the Rust-target.
3. Compile the binary using musl: `RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-musl`
4. The resulting binary is: `target/x86_64-unknown-linux-musl/release/ptc-puppet-agent` (and not `target/release/ptc-puppet-agent` as one would commonly assume when compiling Rust binaries)

You can check that the binary is static using `LDD(1)`:
```txt
$ ldd target/x86_64-unknown-linux-musl/release/ptc-puppet-agent
	statically linked
```
This binary can be copied to other `x86_x64` systems running Linux and it should™ just work.

## Distributing the binary

### Option 1: Copy it around

This is especially easy with the static binary. Use whatever means you prefer, such as `scp`, `rsync` and so on, or put Puppet in charge, as in the following example:

Assuming you have a puppet module and placed the binary in its `files/` subdirectory, you can use a manifest like this (Paths for Debian, adjust for other distributions):
```puppet
file { '/usr/share/prometheus-node-exporter-collectors':
  ensure => directory,
}
-> file { '/usr/share/prometheus-node-exporter-collectors/ptc-puppet-agent':
  ensure => file,
  source => "puppet:///modules/${module_name}/ptc-puppet-agent",
  owner  => 'root',
  group  => 'root',
  mode   => '0755',
}
```

Then, deploy a cronjob or systemd unit+timer (see the `contrib/` directory for examples).

### Option 2: Native distribution packages

This is a bit involved and it is suggested that you are familiar with the packaging guidelines of your distribution of choice. Two examples are provided: One for Debian (Bookworm) and an RPM-based distribution.

Both assume that you have compiled a suitable binary (static or dynamic) already, e.g. as part of an earlier stage in your build-pipeline.

The examples are collected in the `contrib/` directory:
- For Debian, read `contrib/package-debian.md`
- For RPM-based distributions, read `contrib/package-rpm.md`

"Can't you just provide ready-to-use packages?", you might ask. You should not take executable code from random people on the internet. The guide should get you most of the way even if you're not too familiar with Rust and distribution packaging, and should be a good starting point for your future herd of ~~sheep~~little packages.

## Setting it up on a host

Assuming that the binary is in `/usr/local/sbin/ptc-puppet-agent` via a method of your choice (also see the packaging section below), it should now be run every so often to update the file for `prometheus-node-exporter` to be picked up. While there are numerous ways (cronjobs come to mind, of course), the author chose to make use of systemd for this.

Using systemd units has a number of benefits over cron and other methods:
- They are well-supported by all major distributions
- Puppet itself has great support for them using modules such as `voxpupuli/puppet-systemd`
- The tooling that comes with it makes it easy to examine what went wrong (`systemctl status` → `systemctl --failed`, `journalctl -u ptc-puppet-agent.service`)
- `prometheus-node-exporter` can export the state of all or a subset of systemd-units, which closes a gap in monitoring: If `ptc-puppet-agent` fails in a way that doesn't allow it to write the output file, it exits non-zero and causes the unit-activation to fail and the node-exporter picks that up. A simple alert-rule is all that is needed to hear of it.

(This is not the place for curb-wars pro/contra systemd! That being said, if you want to donate a different method, don't hesitate to submit a patch that puts it in a file in the `contrib/` directory and link to it from this file!)

**Quick rundown** of how this works:
- A `ptc-puppet-agent.service` one-shot unit runs the binary each time it is started. If it exits with a non-zero exit-code, the unit itself is marked as failed.
- A `ptc-puppet-agent.timer` unit tells systemd to schedule the service-unit of the same name to be run at regular intervals. The author uses a 5-minute interval, with `puppet-agent` running once per hour.


## Distribution packages

This mostly depends on your needs. You may simply copy the binary around (e.g. via a Puppet `file`-resource, ansible, `scp`, `rsync`, …) or create a package for your distribution. This section focuses on the latter, creating packages.

