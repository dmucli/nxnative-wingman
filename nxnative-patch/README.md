# nxnative patch: external controller input (`DM_EXT_PAD`)

Wingman sends its controller over localhost UDP. Stock nxnative doesn't listen for that, so its Mac host needs this small receiver before you build your game:

- `ext_pad_mac.h`: the receiver (Mac only). It does nothing unless the game's `env.txt` has `DM_EXT_PAD=<port>`.
- `front_mac.m.patch`: two lines in `dm/host/front_mac.m` that include and start it.

Apply it from this folder, with `NXNATIVE` set to your nxnative checkout:

```bash
NXNATIVE=~/nxnative   # wherever you cloned https://github.com/seedds/nxnative
cp ext_pad_mac.h "$NXNATIVE/dm/host/"
git -C "$NXNATIVE" apply "$PWD/front_mac.m.patch"
```

Then rebuild the game with nxnative (`scripts/build_ipa_from_nsp.py <game folder> --platform mac-app`). The work files it keeps are reused, so only the host and the app packaging run again.

If `git apply` fails because `front_mac.m` changed upstream, add the two lines by hand:

1. `#include "ext_pad_mac.h"` on the line just before `static void pads_start(void) {`
2. `extpad_start();` inside `pads_start()`, after `gippad_start();`

After a `git pull` in nxnative, `git -C "$NXNATIVE" status` should still show `dm/host/front_mac.m` modified and `dm/host/ext_pad_mac.h` untracked. If they're gone, apply the patch again and rebuild the game.

The patch is released under the MIT license, the same license as nxnative.
