# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.6](https://github.com/adamgreg/caravel-hal/compare/v1.0.5...v1.0.6) - 2026-04-28

### Other

- Fixes to link on host platforms, for unit tests

## [1.0.5](https://github.com/adamgreg/caravel-hal/compare/v1.0.4...v1.0.5) - 2026-04-28

### Other

- Add mock-user-registers feature, to mock only user (Wishbone) addresses
- Do not trigger CI workflow twice when pushing to a PR branch

## [1.0.4](https://github.com/adamgreg/caravel-hal/compare/v1.0.3...v1.0.4) - 2026-04-24

### Other

- Add HousekeepingSpi driver

## [1.0.3](https://github.com/adamgreg/caravel-hal/compare/v1.0.2...v1.0.3) - 2026-04-24

### Other

- Add mock-registers feature, for convenience
- Re-export user_register_block macro from PAC crate

## [1.0.2](https://github.com/adamgreg/caravel-hal/compare/v1.0.1...v1.0.2) - 2026-04-24

### Other

- Re-export CaravelInterrupt and UserIOBits
- Remove unnecessary argument from Uart::new()

## [1.0.1](https://github.com/adamgreg/caravel-hal/compare/v1.0.0...v1.0.1) - 2026-04-23

### Other

- Add release automation using release-plz
- Lint fixes
- Add associated constants for standard UserIOBits configurations
- Use "mock-registers" feature of caravel-pac for unit tests
- Simplify access to registers from PAC
- Initial commit
