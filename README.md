<p align="center">
    <img src="docs/encore.svg" height="70">
</p>

Encore is a TUI music player that does one thing and one thing only, but does it well.

It _only_ plays music, and it does it so well it has lower overhead than other media players.

> [!IMPORTANT]
> Encore is not yet a feature-complete music player! However, feel free to play around with it if you figure out the ropes.

## I want numbers. Sell me that its lightweight

The bulk (as of right now, all of) Encore's code is written on [this outdated laptop](https://www.ordinateursarabais.com/produit/acer-es1-521-40hc-hdmi-6-go-ram-1-tb/). it takes about 200µs to draw to the tty and i've never seen it use >5% CPU usage. In fact, [it uses less memory than the systemd process when playing a 23 mb .flac file](./docs/img/encore-less-bloated-than-systemd.png).

<!-- TODO: compare resource usage of different audio players, eg vlc, mpv, spotify,. etc -->

## Safe with Rust

Rust (alongside Zig) are the future of programming languages whether you like it or not. No longer will you have to choose between performance (C) or safe code (every other high level language that exists).

Because Encore is written in Rust, you need not worry about getting a remote code execution from a specifically crafted .flac file.

## Vi-inspired

Because Encore runs in the terminal, Encore comes with vi-like keybindings. That means if you are the based ones using a modal editor based on vi or vim then you will find Encore an easy adaptation.

## Cross-platform

Encore natively and will always support these platforms:

- Linux;
- ChromeOS

Encore will always support these platforms on offical releases:

- MacOS;
- FreeBSD;
- DragonFly BSD;
- NetBSD

Support is planned for the following platforms:

- Android (via termux, unrooted?)

> [!NOTE]
> There is no Windows support, though in theory it may work. This will not hold true in future versions.[^1]

[^1]: I don't care about Windows, and eventually I will make it **only** work on Unix-like platforms, such as Linux, MacOS, and \*BSDs. The intent is to make the software **unusable on Windows**. A custom license clause will then be added to forbid the usage of the software on systems primilarly developed by Microsoft.

