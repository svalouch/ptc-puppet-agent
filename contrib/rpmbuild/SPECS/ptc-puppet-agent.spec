Summary:       Prometheus Textfile Collector -- Puppet Agent
Name:          ptc-puppet-agent
Version:       0.4.0
Release:       1
License:       GPL-3.0-only

Source:        ptc-puppet-agent
Source1:       ptc-puppet-agent.service
Source2:       ptc-puppet-agent.timer

Requires:      prometheus-node-exporter
Requires:      puppet-agent
BuildRequires: systemd-rpm-macros

%{?systemd_requires}

%description
Collects metrics about the local puppet agent.

%install
install -D -m 755 %{SOURCE0} %{buildroot}%{_libexecdir}/prometheus-node-exporter-collectors/ptc-puppet-agent
install -D -m 644 %{SOURCE1} %{buildroot}%{_unitdir}/ptc-puppet-agent.service
install -D -m 644 %{SOURCE2} %{buildroot}%{_unitdir}/ptc-puppet-agent.timer

%post
%systemd_post ptc-puppet-agent.service ptc-puppet-agent.timer

%preun
%systemd_preun ptc-puppet-agent.service ptc-puppet-agent.timer

%postun
%systemd_postun_with_restart ptc-puppet-agent.service ptc-puppet-agent.timer
rm -f /var/lib/prometheus/node-exporter/puppet-agent.prom

%files
%{_libexecdir}/prometheus-node-exporter-collectors/ptc-puppet-agent
%{_unitdir}/ptc-puppet-agent.service
%{_unitdir}/ptc-puppet-agent.timer

