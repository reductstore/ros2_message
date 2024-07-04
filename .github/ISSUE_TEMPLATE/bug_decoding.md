---
name: Decoding bug report
about: Report a issue with message decoding
title: "[dec] "
labels: bug, decoding
assignees: A-K-O-R-A

---

**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Please provide a code snippet and the `.msg` / `.srv` files you used.
Also provide the binary data as a file (`.bin` / `.mcap`) or the printed out byte slice.

**Expected behavior**
A clear and concise description of what you expected to happen.

**Error Output**
If applicable, please provide the printed error with a backtrace.
To enable backtraces simply rerun your code with the `RUST_BACKTRACE=1`
environment variable. For example: `RUST_BACKTRACE=1 cargo run`

**Additional context**
If you are decoding messages then please provide the endianess of your system,
you can find it via the `lscpu` command.
