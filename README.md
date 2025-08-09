<!-- markdownlint-disable MD033 MD041 -->
<div id="top"></div>


<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/crypto7world/heritage-gui">
    <img src="https://crypto7.world/crypto7world-logo-v2-250.png" alt="Logo" width="160" height="160">
  </a>

  <h3 align="center">Heritage GUI</h3>

  <p align="center">
    The Heritage wallet GUI, a Bitcoin Taproot wallet managing on-chain inheritance of bitcoins.
    <br />
    <a href="https://btcherit.com"><strong>Explore the Heritage wallet service »</strong></a>
    <br />
    <a href="https://github.com/crypto7world/heritage-gui/issues">Report Bug</a> · <a href="https://github.com/crypto7world/heritage-gui/issues">Request Feature</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ul>
    <li><a href="#about-the-project">About The Project</a></li>
    <li><a href="#known-issues">Known Issues</a></li>
    <li><a href="#stability-and-versioning">Stability and versioning</a></li>
    <li><a href="#installation">Installation</a>
      <ul>
        <li><a href="#from-pre-compiled-binaries">From pre-compiled binaries</a></li>
        <li><a href="#from-source">From source</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a>
      <ul>
        <li><a href="#getting-started-with-onboarding">Getting Started with Onboarding</a></li>
        <li><a href="#blockchain-access-configuration">Blockchain Access Configuration</a></li>
        <li><a href="#private-key-management">Private Key Management</a></li>
        <li><a href="#guided-setup-process">Guided Setup Process</a></li>
      </ul>
    </li>
    <li><a href="#development">Development</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#hardware-wallet-support">Hardware Wallet Support</a></li>
    <li><a href="#minimum-supported-rust-version-msrv">Minimum Supported Rust Version (MSRV)</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#built-with">Built With</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
    <li><a href="#contact">Contact</a></li>
  </ul>
</details>

## About The Project

The Heritage GUI provides a graphical user interface to manage Bitcoin wallets with built-in inheritance capabilities. Built with [Dioxus](https://dioxuslabs.com), it offers an intuitive desktop application for managing your **Heritage wallet**: a Taproot Bitcoin wallet developed in _Rust_ with built-in, on-chain protections against losing your coins and inheritance.

The basic principle is a dead-man switch: should you become unable to spend your coins for whatever reasons, alternative spending paths (i.e. TapScripts) will eventually open, allowing other private keys to spend your coins, following a schedule you configure **enforced by the Bitcoin blockchain**. On the other hand, if you are able to spend your coins you can regularly "reset" this schedule simply by moving your coins to a new address of your wallet.

The **Heritage wallet** offers a trustless solution to protect your coins mainly in 2 situations:

1. You lose access to your wallet for some reason (backup issues, passphrase forgotten, ...)
2. You die.

In both cases, using the **Heritage wallet**, your coins will be recoverable after some time, either by yourself in situation 1 or your next of kin in situation 2.

Usually, protecting yourself against those situations requires one or more trusted third-party with whom you share control of your coins to various degrees. The **Heritage wallet** offers a solution without such compromise: you retain exclusive control of your coins.

The Heritage GUI can interact with the [btcherit.com][heritage-wallet-service] service or manage everything locally. On the private keys front, it can either manage them locally, or with the help of a [Ledger](https://www.ledger.com/) hardware-wallet device.

### I don't want to depend on an online service at all

And I understand: the Heritage GUI is able to work independently of the service! Provide it a custom Bitcoin Core or Electrum node for synchronization, and manage your Heritage wallet entirely on your own!

Beware though that you _SHOULD_ make sure you understand what are the caveats of this mode of operation, most importantly that you _HAVE TO_ backup your descriptors: it is even more important than to backup your seed.

### What is the added value of the service if the GUI can fully operate on its own?

Using Taproot Bitcoin scripts to manage inheritance is only good as long as you don't forget to move your coins to "reset" the dead-man switch. The service is here to remind you of that, as well as making the operation easy or even seamless (for example, if you spend coins a few months before the expiration of your dead-man switch, the service will automatically use this transaction to "reset" it).

**Crucially, the service provides an additional security layer by maintaining a persistent history of your wallet's bitcoin descriptors.** This ensures that even if you lose local access to your wallet configuration, the complex Taproot script information needed to spend your inheritance-protected coins remains available and recoverable.

**The service also offers significant privacy advantages over self-managed setups.** When operating independently, you must provide each heir with the complete wallet descriptors backup, allowing the heirs to recover your full transactions and heritage configurations history. This means heirs gain full visibility into your financial activities and wallet structure. In contrast, when using the service, this sensitive information remains confidential with the service provider, which uses it to create heir transactions without disclosing your private financial details to your beneficiaries.

Also, if you are dead, your heirs should be notified and helped to retrieve the coins you left behind for them. The service is also handling this part.

Of course, you can take steps to do all that on your own; the service is simply here to ease the burden.

<p align="right">(<a href="#top">↑ back to top</a>)</p>

## Known Issues

As this is the first iteration of Heritage GUI, you can expect some cosmetic issues:

- **Missing icon for Windows apps**: The Windows application currently lacks a proper icon. This is a cosmetic issue that doesn't affect functionality.
- **Untested macOS bundle**: The macOS bundle has not been thoroughly tested yet. If you're a macOS user, your feedback is very welcome.

Since we're just releasing the first iteration, please don't hesitate to [open issues][issues-url] for any problems you encounter. I am keen on improving the software and values community feedback.

<p align="right">(<a href="#top">↑ back to top</a>)</p>

## Stability and versioning

Commits between releases SHOULD NOT be considered stable. You should only use tagged releases.

The software provided is working and can be safely used for Bitcoin holdings. That being said, your computer *may not* be secure, so prefer the use of a Ledger device as Key Provider, or be sure to have a strong passphrase if you choose to store your private keys locally.

We are using [Semantic Versioning](https://github.com/semver/semver) (MAJOR.MINOR.PATCH).

Everything is still in the initial-development stage (version 0.x.x). While you can expect every new version to be in working order, you _SHOULD NOT_ expect the GUI interface and features to be stable. That being said, new features and breaking changes will only happen on MINOR version increment, not on PATCH version increment.

<p align="right">(<a href="#top">↑ back to top</a>)</p>

## Installation

### From pre-compiled binaries

You can find precompiled binaries for the major platforms in the Release section of the repo:

[Latest version](https://github.com/crypto7world/heritage-gui/releases/latest) - [All releases](https://github.com/crypto7world/heritage-gui/releases)

#### Using a Ledger on Linux

If you are on Linux and plan on using a Ledger device, you need to install UDEV rules, go to the official Ledger repository: [https://github.com/LedgerHQ/udev-rules](https://github.com/LedgerHQ/udev-rules)

#### Security Warning: Unsigned Binaries

**Important:** The pre-compiled binaries are currently not code-signed. This means:

- **Windows** will show security warnings when running the executable (Windows Defender SmartScreen may block execution)
- **macOS** will prevent the application from running and show "unidentified developer" warnings
- Most antivirus software may flag the binaries as potentially harmful

**What you can do:**

1. **Ignore the warnings** if you trust the source (recommended for users familiar with the project)
   - Windows: Click "More info" then "Run anyway"
   - macOS: Right-click the app, select "Open", then confirm in the dialog

2. **Compile from source** for maximum security assurance (see instructions below)

These warnings are normal for unsigned binaries and don't indicate malicious software. Code signing certificates are expensive and this is an open-source project. We will implement proper code signing in future releases if people take interest in the project.


### From source

To build the Heritage GUI from sources, make sure you have the required dependencies installed.

#### Prerequisites

1. **Rust**: Install Rust using [rustup.rs](https://rustup.rs/):
   ```shell
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup toolchain install stable
   ```

2. **Dioxus CLI**: Install using cargo-binstall (recommended):
   ```shell
   # Install cargo-binstall first
   curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

   # Install Dioxus CLI
   cargo binstall dioxus-cli@0.6.3
   ```

3. **Node.js**: Install npm and the Tailwind CSS CLI:
   ```shell
   # Install Node.js from https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
   ```

#### Platform-specific dependencies

**Linux:**
```shell
sudo apt update
sudo apt install \
  build-essential \
  wget \
  file \
  libwebkit2gtk-4.1-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libusb-1.0-0-dev \
  libudev-dev \
  libhidapi-dev \
  xdotool \
  libxdo-dev
```

If you use Nix, you can setup the project environment with all the dependencies using the provided flake once the repo is forked:

```shell
nix develop ./nix
```

**macOS:**
Dependencies are typically available via system frameworks. If building iOS apps, XCode is required.

**Windows:**
Dependencies are handled by vcpkg or system libraries. WebView2 should be installed (comes with modern Windows/Edge).

#### Building

1. Clone this repository:
  ```shell
  git clone https://github.com/crypto7world/heritage-gui
  cd heritage-gui
  ```

2. Install Tailwind/DaisyUI/Tauri and generate assets:
  ```shell
   npm install -D
  ./generate-assets.sh
  ```

3. Build the application:
  ```shell
  dx bundle -r --platform desktop
  ```

<p align="right">(<a href="#top">↑ back to top</a>)</p>

## Usage

The Heritage GUI features an intuitive onboarding process that guides you through setting up your wallet based on your specific needs and preferences. When you first launch the application, you'll be presented with a streamlined three-step flow designed to determine the optimal configuration for your situation.

### Getting Started with Onboarding

The onboarding process begins with a simple question: **What do you want to do?**

**Setup a Heritage Wallet**
Choose this if you own bitcoins and want to establish inheritance protection. The system will guide you through creating a wallet with built-in dead-man switch functionality.

**Inherit Bitcoins**
Select this option if you're an heir to an existing Heritage wallet and need to claim inherited funds.

**Explore by Yourself**
Skip the guided setup if you prefer to configure everything manually (and you can always reset the onboarding process to do it later).

### Blockchain Access Configuration

Next, you'll choose how to access the Bitcoin blockchain:

**Using the Heritage Service**
The recommended approach for most users. This leverages managed infrastructure at [btcherit.com][heritage-wallet-service], providing enhanced reliability and automated features like inheritance notifications and descriptor backup.

**Using Your Own Node**
For users who prefer complete sovereignty, you can connect to your own Bitcoin Core or Electrum server. This option requires additional technical setup but provides maximum privacy and independence.

### Private Key Management

Finally, for wallet creation scenarios, you'll select your key management approach:

**Ledger Hardware Device**
The most secure option, keeping your private keys on dedicated hardware that never touches your computer or the internet.

**Local Storage with Password**
Software-based key storage with password protection, suitable for users who want convenience while maintaining reasonable security.

**Restore Existing Wallet**
Import an existing mnemonic seed phrase to restore a previously created wallet.

### Guided Setup Process

Based on your selections, the GUI automatically configures the appropriate workflow.

Each includes contextual help, security recommendations, and step-by-step guidance tailored to your chosen configuration. The interface adapts to show only relevant options and information, reducing complexity while ensuring you don't miss critical setup steps.

**Note:** Linux users planning to use Ledger devices must install UDEV rules from the [official Ledger repository](https://github.com/LedgerHQ/udev-rules).

<p align="right">(<a href="#top">↑ back to top</a>)</p>

## Development

1. Install npm: <https://docs.npmjs.com/downloading-and-installing-node-js-and-npm>
2. Install the tailwind css cli: <https://tailwindcss.com/docs/installation>
3. Run the following command in the root of the project to start the tailwind CSS compiler:

```bash
npx @tailwindcss/cli -i input.css -o assets/tailwind.css --watch
```

Run the following command in the root of the project to start the Dioxus dev server:

```bash
dx serve --hot-reload --platform desktop
```

<p align="right">(<a href="#top">↑ back to top</a>)</p>

## Roadmap

The roadmap is accurate regarding the immediate goals for the project:

- [x] Add GUI interface for Heritage wallet management
- [x] Desktop application support (Linux, macOS, Windows)
- [x] Integration with btc-heritage library
- [x] Balance and transaction history display
- [x] PSBT analysis and visualization
- [ ] Multi-language support
- [ ] Mobile application versions (iOS, Android)
- [ ] Support other Hardware Wallet, please request the one you like

Also consult the [open issues][issues-url] for other proposed features and known issues.

<p align="right">(<a href="#top">↑ back to top</a>)</p>

## Hardware Wallet Support

Heritage GUI supports hardware wallets for secure private key management. Below is the current compatibility status:

### Supported Devices

| Device | Status | Notes |
|--------|--------|--------|
| **Ledger** | ✅ Supported | Full support for Taproot wallets using TapScript |

### Unsupported Devices

| Device | Status | Reason |
|--------|--------|--------|
| **Trezor** | ❌ Not Supported | Does not support Taproot wallets using TapScript |

### Request Support for Your Device

Don't see your hardware wallet listed? We welcome requests for additional hardware wallet support!

**[Request Hardware Wallet Support](https://github.com/crypto7world/heritage-gui/issues/new?assignees=&labels=enhancement%2Chardware-wallet&projects=&template=hardware-wallet-support.md&title=%5BHardware+Wallet%5D+Support+for+%5BDEVICE+NAME%5D)**

Please include a link to the official product page and any technical specifications that you know off about Bitcoin/Taproot support when submitting your request.

<p align="right">(<a href="#top">↑ back to top</a>)</p>

## Minimum Supported Rust Version (MSRV)

This application compiles with Rust 1.85.0.

While we will always remain on stable Rust (i.e. _NOT_ depend on nightly), we do not plan on being conservative on the MSRV. If at some point a mildly interesting feature pops in a new Rust version, we will happily bump up the MSRV.

<p align="right">(<a href="#top">↑ back to top</a>)</p>

## License

Distributed under the MIT License. See [`LICENSE`][license-url] for more information.

<p align="right">(<a href="#top">↑ back to top</a>)</p>

## Built With

[![Rust][rust-shield]][rust-url]
[![Dioxus][dioxus-shield]][dioxus-url]

And based upon several Rust projects without which we would not have gotten this far:

- [`dioxus`] - Modern Rust GUI framework
- [`bdk`] - Bitcoin Development Kit
- [`rust-miniscript`] - Bitcoin Miniscript support
- [`rust-bitcoin`] - Bitcoin protocol implementation

Thanks.

<p align="right">(<a href="#top">↑ back to top</a>)</p>

## Acknowledgments

- [`dioxus`] - GUI framework
- [`rust-miniscript`] - Bitcoin script support
- [`rust-bitcoin`] - Bitcoin protocol implementation
- [`bdk`] - Bitcoin wallet infrastructure
- [Best Readme Template](https://github.com/othneildrew/Best-README-Template)
- [Img Shields](https://shields.io)

<p align="right">(<a href="#top">↑ back to top</a>)</p>

## Contact

John Galt - [@Crypto7W](https://twitter.com/Crypto7W) - Though my real name is Jérémie Rodon ([LinkedIn][jr-linkedin-url], [GitHub][jr-github-url]), I operate this project under the pseudonym John Galt in reference to the character of _Ayn Rand_ novel [**Atlas Shrugged**](https://www.amazon.com/Atlas-Shrugged-Ayn-Rand-ebook/dp/B003V8B5XO) (and, yes, I obviously embrace John Galt philosophy).

Project Link: [https://github.com/crypto7world/heritage-gui][repo-url]

<p align="right">(<a href="#top">↑ back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->
[heritage-wallet-service]: https://btcherit.com
[repo-url]: https://github.com/crypto7world/heritage-gui
[contributors-shield]: https://img.shields.io/github/contributors/crypto7world/heritage-gui.svg?style=for-the-badge
[contributors-url]: https://github.com/crypto7world/heritage-gui/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/crypto7world/heritage-gui.svg?style=for-the-badge
[forks-url]: https://github.com/crypto7world/heritage-gui/network/members
[stars-shield]: https://img.shields.io/github/stars/crypto7world/heritage-gui.svg?style=for-the-badge
[stars-url]: https://github.com/crypto7world/heritage-gui/stargazers
[issues-shield]: https://img.shields.io/github/issues/crypto7world/heritage-gui.svg?style=for-the-badge
[issues-url]: https://github.com/crypto7world/heritage-gui/issues
[license-shield]: https://img.shields.io/github/license/crypto7world/heritage-gui.svg?style=for-the-badge
[license-url]: https://github.com/crypto7world/heritage-gui/blob/master/LICENSE
[jr-linkedin-url]: https://linkedin.com/in/JeremieRodon
[jr-github-url]: https://github.com/JeremieRodon
[rust-shield]: https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white
[rust-url]: https://www.rust-lang.org/
[dioxus-shield]: https://img.shields.io/badge/Dioxus-0066CC?style=for-the-badge&logo=rust&logoColor=white
[dioxus-url]: https://dioxuslabs.com/
[`btc-heritage`]: https://github.com/crypto7world/btc-heritage
[`dioxus`]: https://github.com/DioxusLabs/dioxus
[`rust-miniscript`]: https://github.com/rust-bitcoin/rust-miniscript
[`rust-bitcoin`]: https://github.com/rust-bitcoin/rust-bitcoin
[`bdk`]: https://github.com/bitcoindevkit/bdk
