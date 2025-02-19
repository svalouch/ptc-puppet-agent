# Contrib folder

Additional bits and pieces. Some parts are explained in the top-level `INSTALL.md`-file.

## Package-building
After building a binary (see `../INSTALL.md`), these explain how to go about building a package for your distribution:

- Debian-based: `package-debian.md` and the `debian/`-folder
- RPM-based (Fedora, enterprise linux): `package-rpm.md` and the `rpmbuild/`-folder

## Alerts

Some basic alert-rules are in the file `alert-rules.yml`, these should get you started.

If you're using a frontend like Karma, you can add annotations for dashboards or foreman and so on, one of the rules has a commented-out example for how to add these (the complicated part is getting Karma to make a clickable link, refer to Karma's documentation).

## Dashboards

Nothing yet, but planned!

