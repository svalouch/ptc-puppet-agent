# Packaging for Debian

In this folder (`contrib/`), you will find a folder named `debian`, which is an example package that should be ready to use for most people. It assumes that the binary has already been built (if you have a build pipeline, you might choose to compile a dynamic binary for each of the versions of Debian (Bookworm, Trixie, …) and package it, but this assumes a pre-built static binary that should™ then work on all Debian versions with the exact same .deb file).

## Preparations

A few packages should be installed before creating the package: `sudo apt install build-essential debhelper devscripts dpkg-dev fakeroot`

To build, start by choosing a folder and copy the binary (`ptc-puppet-agent`) and the entire `debian` folder into it. This is done because the Debian build-process is picky about additional files that are not part of the build, and should avoid spurious errors. It should then look like this:
```txt
$ tree
.
├── debian
│   ├── changelog
│   ├── control
│   ├── install
│   ├── postinst
│   ├── postrm
│   ├── rules
│   ├── service
│   └── timer
└── ptc-puppet-agent

2 directories, 9 files
```

You need to change the name in the `changelog` and `control` files, and the version number in the `changelog` if a new release should be made! As a rule of thumb, the version should match the version in `Cargo.toml`. If you rebuild it, increment the revision (the number after the minus following the version), so the systems know that it is a newer version. Other than that, you should be good to go and can head to the next step. Read on for a quick run-down of the files.

*Hint*: Consult the official Debian guides for in-depth information on how these things work in detail.

- `changelog` is what sets the version of the resulting package. You need an entry that matches the version of the binary (see the `version` field in `Cargo.toml`). If you just want to get it over with, adjust the version number in the first line and the date in the last line, but be careful not to add or remove lines or white-space at the beginning of lines. Also, please adjust the name, so people can find out who packaged this :)
- `control` allows you to tell Debian what the package needs to be built (Build-Depends) and run (Depends), and what to show in `apt show` and `apt search`). You most likely do not have to touch this file except for changing the `Maintainer:` line to your own details.
  - The package depends on a package named `puppet-agent`, as it makes sense to have that installed. If you use a differently-named package for the agent, you will need to adapt or remove the entry.
- `install` tells `dpkg` where to put the binary, the path in the example is standard for Debian-packaged collectors.
- `postinst` is a shell-script run after the installation. This is similar to how Debian handles other textfile-collectors and creates a few directories and the `prometheus` system-user (which this package doesn't use, but if the directories aren't owned by this user it may cause trouble if other textfile-collectors are installed)
- `postrm` is run after the package has been removed. It removes the default output-file if you purge it (`apt purge ptc-puppet-agent`), so you won't have to manually clean up the stale file.
- `rules` is a Makefile, but it is empty because all we do is copying the already-compiled binary into the package (via the `install`-file).
- `service` is the systemd service unit, it gets installed as `<packagename>.service`, so here, it'll end up as `ptc-puppet-agent.service`. It is a one-shot unit that simply starts the tool without any parameters.
  - *hint* In order to change it (e.g. to add environment variables to change paths), you can use a systemd dropin rather than changing this file. Read below for an example.
- `timer` is the systemd timer unit that starts the corresponding `service` file. It ends up as `<packagename>.timer` → `ptc-puppet-agent.timer`. Run `systemctl list-timers` to view it after installation.

## Building the package

With the above done and with your prompt in the folder _containing the `debian` folder_, trigger the build: `dpkg-buildpackage -us -uc -d`

This will quickly fill your terminal with text. Debian is very picky with a lot of things regarding file content (especially the `changelog` file) and how things are laid out in the filesystem. If the build fails, start from the top, it usually tells you pretty directly what it didn't like. Correct the problems and retry.

Finally, you should have a file named like this: `ptc-puppet-agent_0.3.2-1_amd64.deb` (version `0.3.2`, revision `1`).

Put the file in a debian repository (e.g. via `aptly` if that's what you're using) and install it. Note that by Debian's standards, the timer will automatically be started. If you don't need to divert from the defaults, you're done at this point.

## Running and configuring

If you do not divert from Debian's and PuppetLab's defaults regarding file placement, you should not need to do anything: installing the package automatically activates the timer and runs the collector at the configured interval.

However, if you have different locations for where the `puppet-agent` puts its files or where your `prometheus-node-exporter` reads the textfiles from, you need to change paths. One way is to edit the file `debian/service` and add arguments to the end of the `ExecStart=`-line, or add `Environment=KEY=VALUE`-lines to the `[Service]`-section. The same can be done on the host after installation (don't forget to run `systemctl daemon-reload` to apply your changes!). The by far easiest way is to use a systemd drop-in file.

**Note**: Drop-ins can only _add_ to the resulting service file. You can not change the existing `ExecStart`, only add another (which results in an invalid unit file). Instead, add as many `Environment`-lines as you need to adapt it to your needs.

Drop-in files are plain files anding in `.conf` that sit in a special location. In this case, it would be `/etc/systemd/system/ptc-puppet-agent.service.d/myawsomedropin.conf`. Suppose you want to change the location of the output file because your node-exporter is configured to check for textfiles in a non-standard place. The dropin-file would look like this:

```ini
[Service]
Environment=OUTPUT_FILE=/opt/node_exporter/textfile_collector/puppet-agent.prom
```

Typically, there are three ways to handle this:

_Manually_ creating the directory and the file, followed by running `systemctl daemon-reload` to apply it. You should see a `Drop-In:`-line in `systemctl status ptc-puppet-agent.service` afterwards.

_Using systemctl_, by issuing `systemctl edit ptc-puppet-agent.service`, which opens the file in your `$EDITOR` and does the daemon-reload for you.

_Using Puppet_, which is probably the best way since this is about Puppet to begin with. Use the module `voxpupuli/puppet-systemd` and configure a dropin like so (installing the package is here for completeness sake, and to make sure the order is followed):
```puppet
package { 'ptc-puppet-agent':
  ensure => installed,
}
-> systemd::dropin_file { 'mycustomconfig.conf':
  unit    => 'ptc-puppet-agent.service',
  content => "[Service]\nEnvironment=OUTPUT_FILE=/opt/node_exporter/textfile_collector/puppet-agent.prom",
}
```

## Uninstalling

The package cleans up after itself, within reason (similar to official textfile-collector packages you get from the official repositories): If the package is purged rather than just removed (by running `sudo apt purge ptc-puppet-agent`), it removes the default output file `/var/lib/prometheus/node-exporter/puppet-agent.prom`, so it won't leave a stale file behind that might throw off your monitoring.

If you manage it with Puppet, ensure that your hosts run a manifest similar to the following before removing that section from your environment:
```puppet
package { 'ptc-puppet-agent':
  ensure => purged,
}
```

However, if you _changed_ the output file's location, you have to remove that file manually, as the package itself (the `postrm`-script, rather) has no idea that you did that. You should still purge it, as the first run right after installation might have created a stale file in the default location, and then simply `rm` the actual file.

With Puppet, you can use a manifest like this:
```puppet
file { '/my/custom/location/puppet-agent.prom':
  ensure => absent,
}
```

