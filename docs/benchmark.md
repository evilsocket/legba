---
title: legba vs THC-Hydra benchmark - 1.5x to 55x faster
description: Reproducible benchmark of legba versus THC-Hydra on HTTP, SSH, MySQL, and MSSQL. Same hardware, same wordlist. 1.5x to 55x faster on every test.
---

# Benchmark

Here's a benchmark of `legba` versus `thc-hydra` running some common plugins, both targeting the same test servers on localhost. The benchmark has been executed on a macOS laptop with an M1 Max CPU, using a wordlist of 1000 passwords with the correct one being on the last line. Legba was compiled in release mode, Hydra compiled and installed via [brew formula](https://formulae.brew.sh/formula/hydra).

Far from being an exhaustive benchmark (some legba features are simply not supported by hydra, such as CSRF token grabbing), this table still gives a clear idea of how using an asynchronous runtime can drastically improve performances.

| Test Name | Hydra Tasks | Hydra Time | Legba Tasks | Legba Time |
| --------- | ----------- | ---------- | ----------- | ---------- |
| HTTP basic auth | 16 | 7.100s | 10 | 1.560s **(🚀 4.5x faster)** |
| HTTP POST login (wordpress) | 16 | 14.854s | 10 | 5.045s **(🚀 2.9x faster)** |
| SSH | 16 | 7m29.85s * | 10 | 8.150s **(🚀 55.1x faster)** |
| MySQL | 4 ** | 9.819s | 4 ** | 2.542s **(🚀 3.8x faster)** |
| Microsoft SQL | 16 | 7.609s | 16 | 4.789s **(🚀 1.5x faster)** |

<sup>* While this result would suggest a default delay between connection attempts used by Hydra. I've tried to study the source code to find such delay but to my knowledge there's none. For some reason it's simply very slow.</sup><br/>
<sup>** For MySQL hydra automatically reduces the amount of tasks to 4, therefore legba's concurrency level has been adjusted to 4 as well.</sup>