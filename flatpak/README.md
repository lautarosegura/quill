# Flatpak distribution

Files in this directory build Quill as a Flatpak — the standard
sandboxed-app format on Linux, distributed via [Flathub](https://flathub.org).

Once submitted to Flathub, every Linux user can install Quill regardless
of distro:

```bash
flatpak install flathub com.lauta.quill
flatpak run com.lauta.quill
```

## Files

- `com.lauta.quill.yml` — Flatpak manifest. Declares the runtime, SDK
  extensions, build modules (whisper.cpp, Silero VAD, libxdo,
  libayatana-appindicator, the Quill app itself), and runtime
  permissions.
- `com.lauta.quill.desktop` — XDG desktop entry installed under
  `/app/share/applications/`. Surfaces the app in launchers / overviews.
- `com.lauta.quill.metainfo.xml` — AppStream metadata required by
  Flathub for app discovery in software centers.

## Build locally

On any Linux box with `flatpak-builder` installed:

```bash
# One-time SDK + extensions install
flatpak install -y flathub \
  org.freedesktop.Sdk//24.08 \
  org.freedesktop.Platform//24.08 \
  org.freedesktop.Sdk.Extension.rust-stable//24.08 \
  org.freedesktop.Sdk.Extension.node20//24.08 \
  org.freedesktop.Sdk.Extension.llvm18//24.08

# Build + install into the user-scoped Flatpak repo
flatpak-builder --force-clean --user --install \
  --install-deps-from=flathub \
  build-dir flatpak/com.lauta.quill.yml

# Run
flatpak run com.lauta.quill
```

Build takes ~15-25 min on a modern box (whisper.cpp + cargo + pnpm
all from source).

## Submit to Flathub

Submission happens via a PR to the
[`flathub/flathub`](https://github.com/flathub/flathub) repo — this is
NOT something the in-repo CI does automatically. Steps:

1. Fork `flathub/flathub` to your GitHub account.
2. Create a branch `new-pr/com.lauta.quill`.
3. Add a `com.lauta.quill.yml` (this file) — but with the `quill`
   module's source rewritten to point to a tagged release tarball
   instead of the local `..` path:

   ```yaml
   sources:
     - type: archive
       url: https://github.com/lautarosegura/quill/archive/refs/tags/v0.3.0.tar.gz
       sha256: <sha256 of the tarball>
   ```

4. Open a PR. Flathub maintainers review (usually 1-2 weeks
   response) and once approved, the manifest moves to its own repo
   under `flathub/com.lauta.quill`.

5. Future releases: bump the tag/sha in `flathub/com.lauta.quill`'s
   manifest and Flathub auto-rebuilds.

## Verify metadata

Before submitting, validate the AppStream XML:

```bash
flatpak run --command=appstreamcli org.freedesktop.Sdk//24.08 \
  validate flatpak/com.lauta.quill.metainfo.xml
```

And the desktop entry:

```bash
flatpak run --command=desktop-file-validate org.freedesktop.Sdk//24.08 \
  flatpak/com.lauta.quill.desktop
```

## Permissions explained

The `finish-args` in the manifest lock down what the sandboxed Quill
can talk to. Granted:

| Permission | Why |
|---|---|
| `--share=ipc` | Wayland and X11 require IPC sharing |
| `--socket=wayland` | Native Wayland session support |
| `--socket=fallback-x11` | XWayland fallback when running under Wayland |
| `--socket=pulseaudio` | Microphone capture for dictation |
| `--device=dri` | GPU access required by webkit2gtk |
| `--share=network` | Groq API + model downloads |
| `--talk-name=org.freedesktop.portal.RemoteDesktop` | libei auto-paste on Wayland |
| `--talk-name=org.freedesktop.portal.GlobalShortcuts` | Global hotkey on Wayland (GNOME 48+ / KDE 6) |
| `--talk-name=org.freedesktop.secrets` | Groq API key storage via Secret Service |
| `--filesystem=home/.quill:create` | Config + history database |
| `--filesystem=xdg-config/autostart:create` | Autostart toggle |

Anything not listed (random dirs, other apps' data, the user's
keyboard/mouse outside the portal) is blocked by the Flatpak runtime.
