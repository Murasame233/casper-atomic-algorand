# Intro

Atomic swap between Algorand and Casper

# How it works

See `./principle.md`

# Requirements

- Python 3.9+
- Rust nightly
- WABT (The WebAssembly Binary Toolkit)
- CMake

## Install the pip package

```
pip install -r requirements.txt
```

# Test

For Algorand:

```
cd algorand && bash compile.sh && python3 test.py
```

For Casper:

```
make test
```

# Build

Just

```
make build
```

And the file will be on `build` folder

# P.S.

why no video?

Because I am living a country which have network firewalls (GFW).

This country has strict internet connection restrictions for some foreign servers.

I cannot connect test-net for test.
