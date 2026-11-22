# <img src="web/public/brand.svg" alt="" width="36" align="top"> Clapier

[![ci](https://github.com/synthe-se/clapier/actions/workflows/ci.yml/badge.svg)](https://github.com/synthe-se/clapier/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/synthe-se/clapier)](https://github.com/synthe-se/clapier/releases)
[![changelog](https://img.shields.io/badge/changelog-keep%20a%20changelog-E05735)](CHANGELOG.md)
[![license](https://img.shields.io/github/license/synthe-se/clapier)](LICENSE)

The rabbits' home: one binary serves a Nabaztag:tag its brain, its
sounds and its dances, and gives you a web app to watch and drive the
whole burrow: gestures, colours, ears, sounds, and its firmware,
updated over the network it already lives on. Pairs with the
[uplg/nabgcc](https://github.com/uplg/nabgcc) firmware, which teaches
the 2006 rabbit WPA2/WPA3.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screenshot-dark.png">
  <img src="docs/screenshot.png" alt="Clapier's web app: Garen, awake, the garenne and the firmware it runs, its state and its gestures">
</picture>

## Install

Grab the [latest release](https://github.com/synthe-se/clapier/releases): Linux
(x86_64, aarch64, ARMv6 for the Raspberry Pi 1), macOS and Windows
archives carry the server, `garenne.bin` (the rabbit's brain), a
firmware image, the firmware flasher, the rabbit remote and the
choreography encoder. Nothing else to install.

1. **Flash once.** Head button + power: the rabbit opens a `NabaztagXX`
   access point. Join it, run `./flash-nabaztag Nab-*.sim`. This
   rewrites the whole firmware and nothing sits behind it: keep the
   power steady. A flash that went wrong is recovered over JTAG
   ([nabgcc's guide](https://github.com/uplg/nabgcc/blob/master/docs/jtag-debrick.md)).
   It is the last time the access point is needed.
2. **Serve.** `./clapier --bind 0.0.0.0:80 --overlay overlay` on any
   always-on box (port 80 on Linux: root, or
   `sudo setcap 'cap_net_bind_service=+ep' ./clapier`).
3. **Point the rabbit.** Config mode again, browse to
   `http://192.168.0.1`: pick your network, WPA2, your key, and your
   server's address (`192.168.1.10/vl`). On its first boot fetch
   clapier adopts it: `garenne.bin` lands in `overlay/rabbits/<mac>/`.
4. **Open `http://<server>/`.** Name your rabbit, make it dance.

The rabbit refetches its bytecode at every boot: a bad `bc.jsp` costs a
power cycle, never a brick.

## The web app

One screen, in French or English, light or dark, from a phone to a wide
display. Each rabbit is a tile: its name, whether it is awake, its
address, the garenne it runs and the firmware build under it, then what
it last said about itself (uptime, Wi-Fi, ears, sound, dance). Beside
it, the journal: what the rabbits asked the burrow for, newest first.

| | |
|---|---|
| **Gestures** | Ping, salute, hush, reboot (behind a confirmation). |
| **Dance** | Any `.chor` of the burrow, or one composed on the spot. |
| **Colour and ears** | Eight colours for its belly. Ears on two sliders, with a rabbit that shows where they will stand before anything is sent, and three poses one tap away. |
| **Sound** | Every MP3 of the burrow, a volume in percent, and an MP3 dropped from the browser (4 MB at most). |
| **Firmware** | Drop a `.sim`, have the rabbit check it, flash it. |

**Ears that agree on "up".** No two ears have their encoder's zero at
the same place, so position 0 can mean one ear up and the other folded.
Bring both straight up with the sliders once, press *Both ears are up
now*: clapier keeps that reference for this rabbit, and from then on 0
is up for both, in what the app draws, says and asks.

**Sounds.** The rabbit streams an MP3 as it is; nothing is converted on
the way. Its 2006 decoder is happiest with mono, 32 kHz, constant
bitrate.

## Firmware over the air

Past the first flash, a rabbit running garenne updates its firmware
from where it sits, no config mode and no access point:

1. In the rabbit's tile, **Firmware**, add a `.sim` file. Clapier
   decodes it, keeps only what really is a firmware, and serves it from
   `overlay/common/vl/fw/`.
2. **Check**: the rabbit fetches the image and verifies it, and writes
   nothing. The verdict shows in the tile.
3. **Flash**, behind a confirmation: the same road, then the write. The
   rabbit goes dark for about forty seconds and comes back on the new
   build.

Nothing is written unless the transfer ended well, exactly the announced
number of bytes arrived, they open on the ARM vectors every firmware
carries, they fit below the config sector, and their checksum is the one
clapier computed from the file it serves. There is still no bootloader
behind a flash: do not unplug the rabbit during those seconds, and keep
the JTAG guide within reach for a firmware that turns out not to boot.

The tile names the build a rabbit runs on when its firmware says so
(nabgcc's `wpa23-gtk` r3 and later; older images show nothing and
update all the same).

By hand, the same thing in two commands:

```console
$ ./flash-nabaztag Nab-new.sim --export-bin overlay/common/vl/fw/nab-new.bin
$ ./garenne-ctl --ip 192.168.1.155 fwcheck /vl/fw/nab-new.bin 105088 40490399
```

`flash-nabaztag` prints the exact verbs, `fwcheck` then `fw`.

## Moving house

A rabbit follows its server's address, kept in its config sector. To
move it to another clapier without the config portal:

```console
$ ./garenne-ctl --ip 192.168.1.155 server 192.168.1.10/vl
```

The rabbit first fetches its own bytecode from the candidate. Only if
that works does it rewrite the address and reboot onto its new home; a
wrong address changes nothing.

## The rabbit remote

`garenne-ctl` speaks to a rabbit's control port (UDP 9998, LAN only);
the web app sends the same verbs through clapier's gate.

| Verb | |
|---|---|
| `ping`, `conf`, `heap` | who it is, its config sector's identity card, its memory |
| `reboot`, `reassoc` | restart; scan and associate again |
| `color RRGGBB`, `led N RRGGBB` | its breathing colour; one LED |
| `ears R L` | both ears, 0 to 16 around the turn |
| `play /vl/…`, `stop`, `vol N` | stream an MP3; hush; attenuation, 40 loudest to 254 |
| `dance`, `chor /vl/…` | the salute; a choreography of the burrow |
| `server IP[:PORT][/PATH]` | move to another clapier, once it proved it serves the bytecode |
| `fwcheck`, `fw` `/vl/… LEN ADLER32` | verify a firmware image; verify and flash it |
| `listen` | not a verb: follow the rabbits' log broadcasts |

The rabbit also answers on its own: `http://<rabbit>/status` is its
state, line by line.

## The burrow's files

```
overlay/common/vl/            what every rabbit may ask for
overlay/common/vl/fw/         firmware images (raw, as the rabbits fetch them)
overlay/common/vl/sound/      MP3s added from the web app
overlay/rabbits/<mac>/vl/     one rabbit's own: its bc.jsp, its choreographies
overlay/rabbits/<mac>/name    what you called it
overlay/rabbits/<mac>/ears-up where its ears stand when they are up
```

A rabbit's own file wins over the tribe's, which wins over the base
tree (`--root`, the original Violet tree if you serve one). The JSON
behind the web app lives under `/api`, its files under `/assets`;
`/health` answers `ok`.

## From source

```console
$ cd web && bun install && bun run build && cd ..   # the web app, embedded at build time
$ cargo build --release
$ ./target/release/clapier --bind 0.0.0.0:8080 --overlay garenne/overlay
```

Or `just serve`. On a Raspberry Pi 1 (Alpine/OpenRC):
`PI_HOST=root@<pi> just pi` cross-builds and installs the service,
which then answers at `http://clapier.local/` (mDNS through avahi).
`./garenne/build.sh test` runs the rabbit's golden suite,
`./garenne/build.sh` builds its brain.

`just check` runs what CI runs: fmt, clippy and tests for the server,
svelte-check, the i18n and CSS checks and the tests for the web app;
`just deny` and `just vet` guard the supply chain; `just release X.Y.Z`
cuts a release.

## Architecture

```
crates/clapier         the server: CLI, router, JSON API, fleet listener, the rabbit remote
crates/clapier-vl      the file service in the rabbit's 2006 HTTP dialect
crates/clapier-pages   the embedded web app, directory listings
crates/clapier-fleet   the fleet register, learned from the wire
crates/clapier-*       journal, choreography encoder, firmware flasher
web/                   the web app: Svelte 5, Bits UI, Paraglide (fr, en)
i18n/                  its messages
garenne/               the rabbit's application (Metal bytecode, own IP stack)
docs/violet/           the original Violet and Metal documentation
```

## Standing on friendly shoulders

[Sylvain Huet](https://www.sylvain-huet.com) and the Violet team made the
rabbit and released its [sources](https://code.google.com/archive/p/nabaztag-source-code/source/default/source)
(preserved on [the Violet shelf](docs/violet/README.md));
[RedoXyde's nabgcc](https://github.com/RedoXyde/nabgcc) kept the firmware
alive; [ServerlessNabaztag](https://github.com/andreax79/ServerlessNabaztag)
showed the way.

## License

MIT. Fonts: Cal Sans and Borel (OFL 1.1, in `web/src/fonts`), Fraunces
(OFL 1.1).
