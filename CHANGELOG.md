# Changelog

## [0.7.0](https://github.com/THernandez03/n/compare/v0.6.0...v0.7.0) (2026-06-29)


### Features

* ✨ Gold-colored version manager and program names in output ([84b1b0d](https://github.com/THernandez03/n/commit/84b1b0d7e7de112b8ff391a7867adc07eff9cfe5))

## [0.6.0](https://github.com/THernandez03/n/compare/v0.5.1...v0.6.0) (2026-05-24)


### Features

* ✨ Colored help, -H/-v aliases, styled info/uninstall ([6eba05a](https://github.com/THernandez03/n/commit/6eba05a59cbf42c3aa62e85c160c6676209a5368))
* add binary releases and install.sh ([a848c77](https://github.com/THernandez03/n/commit/a848c77fdf4ab06a16fff181ea77fdb555ad709a))
* add nightly and edge aliases to canary channel ([9d76bb0](https://github.com/THernandez03/n/commit/9d76bb01084564024069a206d6a21d3f624fe338))
* colorized install messages; fix uninstall() return type ([4c5b8fc](https://github.com/THernandez03/n/commit/4c5b8fcb11a40ab1f3175743bba60f9bfb361505))
* display from/to version during activation ([7632e13](https://github.com/THernandez03/n/commit/7632e1399a728a97623a2c080e322af07df2f3a7))
* initial Node.js version manager implementation ([0607bd2](https://github.com/THernandez03/n/commit/0607bd2109f1d1042b12f62f6bec5fb54887fd81))
* restructure CLI, add Makefile, update README ([bd70216](https://github.com/THernandez03/n/commit/bd7021628efefbeff8a71406e209b1c876486466))
* skip activation when version is already active ([1e94fd7](https://github.com/THernandez03/n/commit/1e94fd7b5dc7964f28f2af8e12776d26cd045d1f))


### Bug Fixes

* 🐛 Strip name prefix from self-update version tag ([74fcaf6](https://github.com/THernandez03/n/commit/74fcaf63f3aada076d0a6689f2eba366fce34135))
* remove stale uninstall tests, fix needless borrow in install.rs ([860e401](https://github.com/THernandez03/n/commit/860e401f7e2a3e7c571ef6f8aa1acda63736b079))
* run tests single-threaded to avoid env-var data race between modules ([4eb0e5f](https://github.com/THernandez03/n/commit/4eb0e5fa9cacf9144979945524fec65b3e996f9d))


### Documentation

* 📝 Document prune --force and uninstall --yes flags ([eceb11f](https://github.com/THernandez03/n/commit/eceb11ffb506fd41095dc7ea1e4e07dc4c18004a))
* add related projects section ([2c2cec3](https://github.com/THernandez03/n/commit/2c2cec310173c13b87326040def0932df9fe74b6))

## [0.5.1](https://github.com/THernandez03/n/compare/n-v0.5.0...n-v0.5.1) (2026-05-24)


### Bug Fixes

* 🐛 Strip name prefix from self-update version tag ([74fcaf6](https://github.com/THernandez03/n/commit/74fcaf63f3aada076d0a6689f2eba366fce34135))

## [0.5.0](https://github.com/THernandez03/n/compare/n-v0.4.0...n-v0.5.0) (2026-05-24)


### Features

* ✨ Colored help, -H/-v aliases, styled info/uninstall ([6eba05a](https://github.com/THernandez03/n/commit/6eba05a59cbf42c3aa62e85c160c6676209a5368))
* add binary releases and install.sh ([a848c77](https://github.com/THernandez03/n/commit/a848c77fdf4ab06a16fff181ea77fdb555ad709a))
* add nightly and edge aliases to canary channel ([9d76bb0](https://github.com/THernandez03/n/commit/9d76bb01084564024069a206d6a21d3f624fe338))
* colorized install messages; fix uninstall() return type ([4c5b8fc](https://github.com/THernandez03/n/commit/4c5b8fcb11a40ab1f3175743bba60f9bfb361505))
* display from/to version during activation ([7632e13](https://github.com/THernandez03/n/commit/7632e1399a728a97623a2c080e322af07df2f3a7))
* initial Node.js version manager implementation ([0607bd2](https://github.com/THernandez03/n/commit/0607bd2109f1d1042b12f62f6bec5fb54887fd81))
* restructure CLI, add Makefile, update README ([bd70216](https://github.com/THernandez03/n/commit/bd7021628efefbeff8a71406e209b1c876486466))
* skip activation when version is already active ([1e94fd7](https://github.com/THernandez03/n/commit/1e94fd7b5dc7964f28f2af8e12776d26cd045d1f))


### Bug Fixes

* remove stale uninstall tests, fix needless borrow in install.rs ([860e401](https://github.com/THernandez03/n/commit/860e401f7e2a3e7c571ef6f8aa1acda63736b079))
* run tests single-threaded to avoid env-var data race between modules ([4eb0e5f](https://github.com/THernandez03/n/commit/4eb0e5fa9cacf9144979945524fec65b3e996f9d))


### Documentation

* 📝 Document prune --force and uninstall --yes flags ([eceb11f](https://github.com/THernandez03/n/commit/eceb11ffb506fd41095dc7ea1e4e07dc4c18004a))
* add related projects section ([2c2cec3](https://github.com/THernandez03/n/commit/2c2cec310173c13b87326040def0932df9fe74b6))

## [0.4.0](https://github.com/THernandez03/n/compare/v0.3.1...v0.4.0) (2026-05-24)


### Features

* ✨ Add --force to prune and --yes/-y to uninstall ([c4f4fb2](https://github.com/THernandez03/n/commit/c4f4fb22931ee989c20632739b039c839e2a630b))
