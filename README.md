![Build Status](https://github.com/antifuchs/apply-inovelli-defaults/actions/workflows/ci.yml/badge.svg)

> [!WARNING]
> Beware that this tool is extremely "works for me"; I used it successfully a
> few times and have now abandoned it as I no longer have Inovelli switches in
> my walls.

# A tool for applying common configuration values to devices in zigbee2mqtt

I use a bunch of [Inovelli
Blue](https://inovelli.com/products/blue-series-smart-2-1-switch-on-off-or-dimmer)
switches in my home, and while amazing, there's one annoyance: you
can't set preferences for them in bulk. I'd like all my dimmers to be
slightly faster than the default - but I want them all to have the
same speed. That gets extremely tedious in larger installations (10
and up switches), so here's what I built.

This tool talks to the zigbee2mqtt websockets endpoint; it takes a
config file that lists common settings that should be applied to
switches matching a particular criteria (all Dimmers that glow in the
dark, say)... and then it sends the config values you set to every
matching device.

See [examples/inovelli-dimmers.yaml](examples/inovelli-dimmers.yaml)
for an example config file that I use to set up my switches.

# Caution

If you use this, it will absolutely destroy your switches (possibly causing 
fires and other exciting electrical problems) if given wrong-enough config
parameters. If it breaks anything, you get to keep both parts.

This tool is written in Rust (which let me write this over the course
of a Sunday and have it work first try, but means you gotta know about
how to run it). You should understand how to run and ideally debug
rust code, and how to adjust the `RUST_LOG` setting to get more debug
logs.
