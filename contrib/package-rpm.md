# Packaging for RPM-based distributions

This guide should work fine for at least the Fedora-based distributions (including commercial derivatives), as the setup was made following the Fedora packaging guidelines as closely as possible.

In this folder ('contrib/'), you will find a folder named `rpmbuild`, which is an example package that should be ready to use for most people. It assumes that the binary has already been built and assumes the static version. The result should be a ready-to-install RPM-package. In contrast to Debian, Fedora's packaging guidelines forbid automatically activating service- and timer-units, but examples are provided further down.

## Preparations

A few packages should be installed before creating the package: `sudo dnf install rpmdevtools systemd-devel yum-utils`  
Additionally:
- For distributions derived from enterprise linux 8: `epel-rpm-macros-systemd`
- For distributions derived from enterprise linux 9: `systemd-rpm-macros`

Copy the `ptc-puppet-agent`-binary to the folder `rpmbuild/SOURCES`, which should then look like this:
```txt
$ tree
.
└── rpmbuild
    ├── SOURCES
    │   ├── ptc-puppet-agent
    │   ├── ptc-puppet-agent.service
    │   └── ptc-puppet-agent.timer
    └── SPECS
        └── ptc-puppet-agent.spec

4 directories, 4 files
```

- `SOURCES/ptc-puppet-agent` is the binary
- `SOURCES/ptc-puppet-agent.service` is the service unit
- `SOURCES/ptc-puppet-agent.timer` is the corresponding timer unit
- `SPECS/ptc-puppet-agent.spec` is the file telling `rpmbuild` how to create the package.

The only fields you might have to adapt in the `.spec`-file are the `Version:` and `Revision:` lines. A changelog entry is optional and omitted for ease-of-use; you can add one to keep track of your internal builds, for example.

## Building the package

With the above done and with your prompt in the folder _containing the `rpmbuild` folder_, trigger the build: `rpmbuild --define "_topdir $(pwd)/rpmbuild -ba rpmbuild/SPECS/ptc-puppet-agent.spec`

If all goes well it creates a number of directories beneath `rpmbuild`, only one is of interest: `rpmbuild/RPMS`. There should be a single RPM, named like `ptc-puppet-agent-0.3.2-1.x86_64.rpm`, which should work on all `x86_64`-based systems.

Note that the service and timer are not configured to run right after installation, in conformance with the Fedora packaging guidelines. Read on how to change that.

## Running and configuring

By default, the package remains inert and you have to enable the timer and its service yourself.

You can do so _manually_ by issuing `systemctl enable --now ptc-puppet-agent.timer && systemctl enable ptc-puppet-agent.service`

Or, since you are most likely already using Puppet, use this manifest:
```puppet
package { 'ptc-puppet-agent':
  ensure => installed,
}
service { 'ptc-puppet-agent.timer':
  ensure  => true,
  enable  => true,
  require => Package['ptc-puppet-agent'],
}
```

For the rest of the configuration and a short introduction to systemd drop-in files, head over to `contrib/package-debian.md` and its "Running and configuring"-section.

## Uninstalling

The package cleans up after itself, that is, it removes the textfile-collector output file at its default location. If you changed the location (e.g. by supplying a `OUTPUT_FILE` environment variable), you will have to remove the resulting file yourself (e.g. via Puppet) or you will end up with a stale file that might throw your monitoring off.

