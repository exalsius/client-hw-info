# Changelog

## [2.2.0](https://github.com/exalsius/client-hw-info/compare/v2.1.0...v2.2.0) (2025-09-25)


### Features

* add network detection ([65f32f7](https://github.com/exalsius/client-hw-info/commit/65f32f7c6b19d029aaddb11971abb06bcabbe610))

## [2.1.0](https://github.com/exalsius/client-hw-info/compare/v2.0.1...v2.1.0) (2025-09-23)


### Features

* the GPU detection does not rely on lspci anymore but reads out pci directly in the file system ([84f1d49](https://github.com/exalsius/client-hw-info/commit/84f1d49f632576524eb30ac9eb46f047bea02eeb))


### Bug Fixes

* remove lspci command ([712b144](https://github.com/exalsius/client-hw-info/commit/712b144300bf5adc8bc1533a0ff975b411b40838))

## [2.0.1](https://github.com/exalsius/client-hw-info/compare/v2.0.0...v2.0.1) (2025-09-23)


### Bug Fixes

* change auth to access ([10777d8](https://github.com/exalsius/client-hw-info/commit/10777d8b061b7672a80c0b7aebaa07f681b1f990))

## [2.0.0](https://github.com/exalsius/client-hw-info/compare/v1.5.0...v2.0.0) (2025-09-23)


### ⚠ BREAKING CHANGES

* change the way of communication with the API

### Features

* change the way of communication with the API ([1fad302](https://github.com/exalsius/client-hw-info/commit/1fad302bd1bfb0d0caf8910788fd04026ca03819))

## [1.5.0](https://github.com/exalsius/client-hw-info/compare/v1.4.0...v1.5.0) (2025-09-23)


### Features

* add tests for multiple distros ([aeb753b](https://github.com/exalsius/client-hw-info/commit/aeb753b4c49712105d3bdd5b3426d6c9d5a21bc1))

## [1.4.0](https://github.com/exalsius/client-hw-info/compare/v1.3.0...v1.4.0) (2025-09-21)


### Features

* it allows skipping the heartbeat to only collect hardware information ([9c3b846](https://github.com/exalsius/client-hw-info/commit/9c3b846e3393bcd313e3c7a0ece4b85005bcedd8))

## [1.3.0](https://github.com/exalsius/client-hw-info/compare/v1.2.1...v1.3.0) (2025-09-19)


### Features

* implement logging and refactored code ([a0ea24e](https://github.com/exalsius/client-hw-info/commit/a0ea24e39ec9887efad20088c7eaecff149772ae))
* implement logging and refactored code ([a0ea24e](https://github.com/exalsius/client-hw-info/commit/a0ea24e39ec9887efad20088c7eaecff149772ae))

## [1.2.1](https://github.com/exalsius/client-hw-info/compare/v1.2.0...v1.2.1) (2025-09-19)


### Bug Fixes

* change unknown to UNKNOWN to comply with API reference ([c3a26e8](https://github.com/exalsius/client-hw-info/commit/c3a26e8966210615cee7b4c23a1779c59d61269e))

## [1.2.0](https://github.com/exalsius/client-hw-info/compare/v1.1.1...v1.2.0) (2025-09-18)


### Features

* add uploading build to release ([a473378](https://github.com/exalsius/client-hw-info/commit/a4733782cde6deb242205b7f8745dd9323548f09))

## [1.1.0](https://github.com/exalsius/client-hw-info/compare/v1.0.1...v1.1.0) (2025-09-18)


### Features

* automatic generation of auth tokes by using a refresh token ([#5](https://github.com/exalsius/client-hw-info/issues/5)) ([bfbd4e7](https://github.com/exalsius/client-hw-info/commit/bfbd4e77120ea50489c56dd7c969c472a8bdeb58))

## [1.0.1](https://github.com/exalsius/client-hw-info/compare/v1.0.0...v1.0.1) (2025-09-18)


### Bug Fixes

* remove the version name for now to build binary ([828c726](https://github.com/exalsius/client-hw-info/commit/828c7260fc7462ca24dcbf01678b61c769782952))

## 1.0.0 (2025-09-18)


### Features

* add basic release-please workflow ([39109f6](https://github.com/exalsius/client-hw-info/commit/39109f613e44718adbf6e9944098319b1620e9ff))
* add heartbeat API patch call ([e13acb8](https://github.com/exalsius/client-hw-info/commit/e13acb80145f750afad388bd84d934ca940718f8))
* dynamically create env with given parameters ([14e2807](https://github.com/exalsius/client-hw-info/commit/14e28070304ad990729d66c44e1dd96867134842))
* initial version for client hardware info tool ([6b63432](https://github.com/exalsius/client-hw-info/commit/6b634324d19350bf3a3a46b0f478234a8e78ab58))
* static musl build for Linux x64 when running cargo --release ([c398d92](https://github.com/exalsius/client-hw-info/commit/c398d92490c75d054205af30ca8aec4632508603))


### Bug Fixes

* add musl install to ci ([9754d83](https://github.com/exalsius/client-hw-info/commit/9754d830f3ca4769de55136794cfef2d7346bfc3))
