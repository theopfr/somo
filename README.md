<p align="center">
  <img src="./images/somo-logo.png" />
</p>


### A human-friendly alternative to netstat for socket and port monitoring on Linux.

## ✨ Features:
- pleasing to the eye thanks to a nice table-view
- filterable (see filter-options below)
- interactive killing of processes
- from ``netstat -tulpn`` to ``somo -l`` (almost half the characters, can you believe it?)

<br />

<p align="center">
  <img src="./images/somo-example.png" />
</p>

---

## ⬇️ Installation:

### Option 1 - Debian:
If you use a Debian OS go to [releases](https://github.com/theopfr/somo/releases) and download the latest .deb release.

### Option 2 - From crates.io:
```sh
cargo install somo
```
Most of the time you will want to run this in ``sudo`` mode to see all processes and ports. By default, this is not possible when installed via cargo. But you can create a symlink so the binary can be run as root:
```sh
sudo ln -s ~/.cargo/bin/somo /usr/local/bin/somo
sudo somo   # this works now
```

### Option 3 - Nix:

If you use Nix with Flakes, you can build and use the development version.

```sh
nix build github:theopfr/somo
sudo ./result/bin/somo
```

---

## 🏃‍♀️ Running somo:
To run somo just type: 
```sh
sudo somo
```



### Filtering:

You can use the following flags to filter based on different attributes:
| filter flag | description | value |
| :------------- |:------------- | :----- |
| ```--proto``` | filter by either TCP or UDP  | ``tcp`` or ``udp`` | 
| ```--port, -p``` | filter by a local port | port number, e.g ``5433`` |
| ```--remote-port``` | filter by a remote port | port number, e.g ``443`` |
| ```--ip``` | filter by a remote IP | IP address e.g ``0.0.0.0`` |
| ```--program``` | filter by a client program | program name e.g ``chrome`` |
| ```--pid, -p``` | filter by a PID | PID number, e.g ``10000`` |
| ```--open, -o``` | filter by open connections | - |
| ```--listen, -l``` | filter by listening connections | - |
| ```--exclude-ipv6``` | don't list IPv6 connections | - |


### Process killing:
With the ``--kill, -k`` flag you can choose to kill a process after inspecting the connections using an interactive selection option.
![kill-example](./images/somo-kill-example.png)

You can of course also apply filters and the kill-flag at the same time:
```sh
somo --program postgres -k
```
