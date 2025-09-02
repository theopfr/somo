# Changelog

## [1.2.0] - 02.09.2025
- **add** new `--ipv4/-4` and `--ipv6/-6` flags, deprecating `--exclude-ipv6`, by @cma5
- **add** config file for setting default flags, by @theopfr
- **add** port annotation with IANA service names, by @pyjuan91
- **add** new `--established/-e` flag for only showing established connections, by @theopfr
- **update** table readability and refactor, by @reneleonhardt @theopfr
- **add** Dependabot, by @reneleonhardt
- **add** tests for parsing connections on macOS, by @theopfr

---

## [1.1.0] - 09.07.2025
- **add** macOS support, by @belingud
- **add** sorting by column, by @gerelef
- **fix** panicking when being piped, by @gerelef
- **add** compact table view, by @theopfr
- **add** format and clippy check to CI, by @jerry1098
- **update** CLI arguments with `--tcp`, `--udp` flags, by @celeo
- **add** shell completions, by @polponline
- **add** Nix packaging support, by @kachick
- **update** kill functionality to use SIGTERM, by @rongyi
- **add** JSON and custom formattable output, by @aptypp @gerelef
- **fix** typos in README, by @cma5 @robinhutty
- **add** MIT license, by @theopfr

---

## [1.0.0] - 04.06.2025
- **update** flags, by @theopfr
    - move ``--local-port`` to ``--port``
    - move ``--port`` to ``--remote-port``
    - add ``--listen``
- **remove** abuseipdb scanning, by @theopfr
- **add** logo, by @theopfr
- **add** tests, by @theopfr

---