# Cinny desktop

<a href="https://github.com/cinnyapp/cinny-desktop/releases">
  <img alt="GitHub release downloads" src="https://img.shields.io/github/downloads/cinnyapp/cinny-desktop/total?style=social"></a>

Cinny is a matrix client focusing primarily on simple, elegant and secure interface. The desktop app is made with Tauri.

## Download

Installers for macOS, Windows and Linux can be downloaded from [Github releases](https://github.com/cinnyapp/cinny-desktop/releases). Releases are signed with a [Ed25519](https://ed25519.cr.yp.to/) public-key.

Operating System | Download
---|---
Windows | <a href='https://github.com/cinnyapp/cinny-desktop/releases/latest/download/Cinny_desktop-x86_64.msi'>Get it on Windows</a>
macOS | <a href='https://github.com/cinnyapp/cinny-desktop/releases/latest/download/Cinny_desktop-universal.dmg'>Get it on macOS</a>
Linux | <a href='https://github.com/cinnyapp/cinny-desktop/releases/latest/download/Cinny_desktop-x86_64.AppImage'>Get it on Linux</a> · <a href='https://flathub.org/apps/details/in.cinny.Cinny'>Flatpak</a>

Decoded public key:
> RWRflTUQD3RHFtn25QNANCmePR9+4LSK89kAKTMEEB4OKpOFpLMgc64z

To verify release files, you need to download [minisign](https://jedisct1.github.io/minisign/) tool and [decode](https://www.base64decode.org/) the *.sig* file before running:
>  minisign -Vm ***RELEASE_FILE.msi.zip*** -P RWRflTUQD3RHFtn25QNANCmePR9+4LSK89kAKTMEEB4OKpOFpLMgc64z -x ***SINGATURE.msi.zip.sig***

## Local development

Firstly, to setup Rust, NodeJS and build tools follow [Tauri documentation](https://tauri.app/v1/guides/getting-started/prerequisites).

Now, to setup development locally run the following commands:
* `git clone --recursive https://github.com/cinnyapp/cinny-desktop.git`
* `cd cinny-desktop/cinny`
* `npm ci`
* `cd ..`
* `npm ci`

To build the app locally, run:
* `npm run tauri build`

To start local dev server, run:
* `npm run tauri dev`

## Submodule Management

This project uses `cinny` as a git submodule pointing to [cinny-i18n](https://github.com/zhao-wuyan/cinny-i18n).

> **Note**: Submodules are **NOT** automatically updated. They are pinned to a specific commit. You must manually update when needed.

### Update Submodule to Latest

**Method 1: Quick update (recommended)**
```bash
# Pull latest commit from remote
git submodule update --remote cinny
git add cinny
git commit -m "chore: update cinny submodule"
```

**Method 2: Merge/Rebase mode (for local changes)**
```bash
# Merge mode
git submodule update --remote --merge cinny

# Rebase mode
git submodule update --remote --rebase cinny
```

**Method 3: Manual update**
```bash
cd cinny
git checkout main
git pull origin main
cd ..
git add cinny
git commit -m "chore: update cinny submodule"
```

**One-liner (complete flow):**
```bash
git submodule update --remote --merge cinny && git add cinny && git commit -m "chore: update cinny submodule"
```

### Initialize Submodule

**For new clone with `--recursive`:**
```bash
git clone --recursive https://github.com/zhao-wuyan/cinny-desktop.git
```

**For existing clone without submodule:**
```bash
git submodule update --init cinny
```
