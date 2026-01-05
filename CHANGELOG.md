# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Regression test for decoding `std_msgs/msg/Empty` with payload `[0, 1, 0, 0]`.

### Fixed
- Dynamic message decoder now tolerates trailing zero padding, preventing false failures on empty messages.


## 0.1.0 - 2025-01-05

Initial release after fork and many fixes
