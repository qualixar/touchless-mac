# Security policy

touchless-mac reads your camera and controls your mouse and keyboard, so security reports matter.

## Reporting a vulnerability

Please report privately through GitHub: **Security → Report a vulnerability** on this repository ([direct link](https://github.com/qualixar/touchless-mac/security/advisories/new)). Don't open a public issue for a security problem.

Include what you found, how to reproduce it, and the version (`touchless-mac --version`). You'll get a first reply within seven days.

## What the app does and doesn't do

- Camera frames are processed in memory on your Mac and never saved or sent anywhere. The app makes no network requests at run time.
- The only file it writes is your calibration: `~/Library/Application Support/touchless-mac/reach-box.txt`.
- The hand models are fetched once at install time and checked against SHA-256 fingerprints.
- The ready-made build isn't signed by an Apple developer account yet. If you'd rather not run an unsigned binary, build from source: the README's quick start shows how.

## Supported versions

Security fixes go into the latest release.
