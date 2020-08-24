# DigitalOcean

[![crates.io](https://img.shields.io/crates/v/digitalocean.svg)](https://crates.io/crates/digitalocean)
[![docs](https://docs.rs/digitalocean/badge.svg)](https://docs.rs/digitalocean)
[![MIT licensed](https://img.shields.io/crates/l/digitalocean.svg)](./LICENSE.md)

A crate for interacting with DigitalOcean's API. Using this crate, you can
manage droplets, domains, load balancers, images and more using an account
token.

## Rewrite

This branch is a rewrite of the crate. The older version was created and
maintained by [Hoverbear](https://github.com/hoverbear), but Rust has changed a
lot since then. The old version also doesn't support things like ratelimiting,
and they haven't really been maintaining it for a while.

This branch will not be published to `crates.io` until it is complete and
stable. I'd love help with code review and development, feel free to open PRs
for code changes, and new issues for ideas!

---

Note: I'm still not sure if this should be version 1.0.0, or still in 0.x.x...
I'm leaning a bit towards the latter, since this rewrite, even if newer, is
still untested (and uncriticized) code.
