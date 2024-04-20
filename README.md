# tohorank
A Touhou character sorter, based on [charasort](https://github.com/execfera/charasort/). Unlike charasort, which runs a bottom-up merge sort, tohorank assigns a numerical rating to each character using the Glicko-2 rating system.
> Technically supports other series as well, but some features, like the PC-98 flags, are hardcoded in.
<img src="https://github.com/randomtwdude/tohorank/assets/105645765/736da729-6432-453a-ae50-09d6b5db010f" width=500>

## Features
- **Precise numerical ratings**: can't stop picking ties like the indecisive shopper in front of a shelf? Now you get to know precisely how much you *love* [insert name here].
<img src="https://github.com/randomtwdude/tohorank/assets/105645765/c6a06c33-07d3-4106-b347-84160f76d6d2" width=500>

- **Incremental updates**: ratings update after every session, which can be as long or short as you like, and you can check your ranking at any time.
> Note that, like tohosort, it still takes a while to get representative results, so don't set aside your tea/coffee
> (Theoretically, tohosort is O(n log n) and as fast as it gets)

- **More options**: Don't know either of the characters or just plain dislike them? You can just say both sides lose instead of being forced to pick ties.

- **Detailed stat tracking**: see where every character stands in various works, historical stats, and more
<img src="https://github.com/randomtwdude/tohorank/assets/105645765/4b19fe59-89c9-478c-94ea-088bb94066c2" width=500>

- **Filters**: include or exclude any works or stages from your sessions

## Installation
0. Clone the repo: `git clone https://github.com/randomtwdude/tohorank` and cd `cd tohorank`

The `install.sh` script does the following for you:
1. compile<br>
   `cargo build --release`
2. the binary is located at: `./target/release/tohorank`, move it wherever you like. The install script chooses `/usr/bin/tohorank`, which requires `sudo`.
3. **IMPORTANT** Move the stock character list at `./src/touhous.txt` to `$HOME/.tohorank/touhous.txt`
> :warning: Warning: `$HOME/.tohorank/touhous.txt` *must* exists or tohorank will panic on startup!

## Usage
If everything goes well you should get the lobby.
- Type 'start' to start a new sorting session.
- Type 'list' to see the current ranking.
- Type 'stat [name]' to see stats of a character (stat! for more stats)
### Filters
You can specify filters after either `start` or `list`. List optionally also takes a number (only show the top-n) and name.<br>
- to only include characters appearing in *Touhou 06 - Koumakyou ~ The Embodiment of Scarlet Devil*, `eosd` or `th06`.
- to exclude them instead, prepend a minus sign in front of the tag, like `-eosd`.
- to specify stages, use `st1` ~ `st6` and `ex`
- to enable PC-98 duplicates (`pc98`), enable non-girls (`notgirl`), or disable nameless characters (`-nameless`)
- multiple tags are separated by spaces
<img src="https://github.com/randomtwdude/tohorank/assets/105645765/846b9856-89dd-4c3b-9d0b-726df545f93a" width=500>

#### Examples
`start in gfw -st4`: only characters from Imperishable Night and Great Fairy Wars, but none from Stage 4.
`list st4 p`: list only characters from Stage 4, with "p" in their name.
