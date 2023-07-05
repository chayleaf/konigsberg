# Königsberg

This is like Goldberg, but Königsberg. It wraps the Steam API to make
all DLCs be considered unlocked. It won't help getting the DLC files,
nor will it affect any other Steam functionality.

To use:

1. Rename `libsteam_api.so`/`steam_api.dll` to
   `libsteam_api.orig.so`/`steam_api.orig.dll`
2. Move `libkonigsberg.so`/`konigsberg.dll` to where
   `libsteam_api.so`/`steam_api.dll` used to be

