Hey team. Our current goal is to rigorously re-design and standardize our API for interface backend communications drawing inspiration from every possible mature mpd client implementation. Our final goal is to have most mature, feature rich, stable, highly ergonomic but expressive, and correctly designed API layer that sits between lock files and running MPD.

We will draw inspiration and outright borrow functionality from such projects as:

- [myMPD](https://github.com/jcorporation/myMPD)
- [rmpc](https://github.com/mierak/rmpc)

We will study the API design and boundaries heavily to ensure every future UI has unmatched power when it comes to interfacing albums.

Every MPD call will be prefixed as `/api/mpd/` to clearly set the boundary.


## `/api/control/`

Stateless POST API calls.

- `toggle_pause` Toggles between play and pause
- `toggle_repeat` Toggles the repeat mode
- `toggle_random` Toggles the random mode
- `toggle_single` Cycles between the single and the normal mode
- `toggle_oneshot` Cycles between the oneshot and the normal mode
- `toggle_consume` Toggles the consume mode
- `volume_delta` Sends normalized between -100 and 100 delta to the MPD volume
- `seek_delta` Sends normalized delta in milliseconds to the MPD clock

## `/api/albums/`

- Receives INIT_DICT when called by itself
- `{id}` Receives `album.lock.json` payload
- `{id}/play` Replaces queue -> plays immediately `{"offset": 0}`
- `{id}/queue` Appends album to the queue `{"position": "tail" | "next"}`
- `{id}/discs/{discnumber}/play` Replaces queue with album disc -> plays immediately `{"offset": 0}`
- `{id}/discs/{discnumber}/queue` Appends album disc to the queue `{"position": "tail" | "next"}`
