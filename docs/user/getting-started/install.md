---
title: Install EvoPhoto
description: Download EvoPhoto for Windows 10 or 11, macOS or Linux from upstream-site.com or GitHub, or install it with Upstream Launcher.
order: 10
verified: 0.8.2
---

The easiest way is the [EvoPhoto page on upstream-site.com](https://evora.st). You can also install it with [Upstream Launcher](https://evora.st), which lists EvoPhoto. For a specific build or file format, take it from the [latest release](https://github.com/tazerty/evophoto/releases/latest) on GitHub. In the file names below, `<version>` stands for the version number, for example `0.8.2`.

## Windows

EvoPhoto needs Windows 10 or later. On Windows 7 and 8 the installer stops with "EvoPhoto needs Windows 10 or later."

1. Open the [latest release](https://github.com/tazerty/evophoto/releases/latest).
2. Download `evophoto-<version>-windows-x64.msi`. It fits most PCs (64-bit Intel and AMD).
3. Run the file and follow the installer.

To run without installing, download `evophoto-<version>-windows-x64-portable.zip` instead and unzip it.

| Your PC                            | Installer                                | Portable                                          |
|------------------------------------|------------------------------------------|---------------------------------------------------|
| 64-bit Intel or AMD                | `evophoto-<version>-windows-x64.msi`   | `evophoto-<version>-windows-x64-portable.zip`   |
| Arm (Snapdragon and other Arm PCs) | `evophoto-<version>-windows-arm64.msi` | `evophoto-<version>-windows-arm64-portable.zip` |
| 32-bit                             | `evophoto-<version>-windows-x86.msi`   | `evophoto-<version>-windows-x86-portable.zip`   |

The installers and executables are code-signed.

## macOS

1. Download `evophoto-<version>-macos-universal.dmg`. It works on Apple silicon and Intel Macs, and is signed and notarized by Apple.
2. Open it and drag EvoPhoto to Applications.

## Linux

Pick one file from the release. Use `x86_64` for Intel and AMD, or `aarch64` for Arm.

| Format              | File                                         | Notes                                                                              |
|---------------------|----------------------------------------------|------------------------------------------------------------------------------------|
| AppImage            | `evophoto-<version>-linux-x86_64.AppImage` | Runs anywhere and needs no install.                                                |
| Debian and Ubuntu   | `evophoto-<version>-linux-x86_64.deb`      |                                                                                    |
| Fedora and openSUSE | `evophoto-<version>-linux-x86_64.rpm`      |                                                                                    |
| Flatpak             | `evophoto-<version>-linux-x86_64.flatpak`  | Run `flatpak install --user <file>`, then `flatpak run ai.storyteller.evophoto`. |
| Tarball             | `evophoto-<version>-linux-x86_64.tar.gz`   | Unpack it anywhere.                                                                |

The first time you run the AppImage, it adds its icon and menu entry under `~/.local/share`. Set `EVOPHOTO_NO_DESKTOP_INTEGRATION=1` to skip that. For RISC-V, take the `riscv64` file: `evophoto-<version>-linux-riscv64.tar.gz`, `.deb` or `.rpm`. The RISC-V tarball needs glibc 2.39 or newer.

On a Wayland session, files dropped on the EvoPhoto window do not open yet. Use **File › Open…** instead, or copy the image in your file manager and paste it with Ctrl+V. To drag and drop, start EvoPhoto under XWayland with `WAYLAND_DISPLAY= evophoto`. EvoPhoto shows the right command for your install in a notice until you dismiss it.

## FreeBSD and the web

FreeBSD 14 (x86_64) has a tarball, `evophoto-<version>-freebsd-x86_64.tar.gz`, laid out like `/usr/local`. The web version comes as `evophoto-web-<version>.zip`, a static site you host on any web server. It runs in a modern browser.

## Check your download

`SHA256SUMS.txt` on the release lists a checksum for every file.

## Next step

Open an image: [Open and save files](../guides/open-and-save-files.md).
