# Changelog

All notable changes to this project will be documented in this file.

## [0.16.1](https://github.com/catppuccin/rockdove/releases/tag/0.16.1) - 2024-09-22


### 🐛 Bug Fixes

- Only include discussion body on `created` event ([#24](https://github.com/catppuccin/rockdove/issues/24)) ([99b801f](https://github.com/catppuccin/rockdove/commit/99b801f))


### 📚 Documentation

- Update changelog template ([cf6dcb6](https://github.com/catppuccin/rockdove/commit/cf6dcb6))



## [v0.16.0](https://github.com/catppuccin/rockdove/releases/tag/v0.16.0) - 2024-09-22


### 🚀 Features

- Support `discussion_comment` event ([#23](https://github.com/catppuccin/rockdove/issues/23)) ([3a2060e](https://github.com/catppuccin/rockdove/commit/3a2060e))
- Support `discussion` event ([#20](https://github.com/catppuccin/rockdove/issues/20)) ([85a06f3](https://github.com/catppuccin/rockdove/commit/85a06f3))
- Support `commit_comment` event ([#19](https://github.com/catppuccin/rockdove/issues/19)) ([1e33930](https://github.com/catppuccin/rockdove/commit/1e33930))


### 🚜 Refactor

- Split into separate files ([#21](https://github.com/catppuccin/rockdove/issues/21)) ([4f37de3](https://github.com/catppuccin/rockdove/commit/4f37de3))


### 🧪 Testing

- Add release fixture ([#22](https://github.com/catppuccin/rockdove/issues/22)) ([aeffbdb](https://github.com/catppuccin/rockdove/commit/aeffbdb))


### ⚙️ Miscellaneous Tasks

- Bump version to 0.16.0 ([e8f0621](https://github.com/catppuccin/rockdove/commit/e8f0621))



## [v0.15.2](https://github.com/catppuccin/rockdove/releases/tag/v0.15.2) - 2024-09-21


### 🚜 Refactor

- Remove most colour exceptions ([#18](https://github.com/catppuccin/rockdove/issues/18)) ([788fe24](https://github.com/catppuccin/rockdove/commit/788fe24))


### 📚 Documentation

- Update CHANGELOG ([6cbe5ba](https://github.com/catppuccin/rockdove/commit/6cbe5ba))
- Add git-cliff changelog ([86052b4](https://github.com/catppuccin/rockdove/commit/86052b4))


### 🧪 Testing

- Add snapshot tests via insta.rs ([#17](https://github.com/catppuccin/rockdove/issues/17)) ([ade7f55](https://github.com/catppuccin/rockdove/commit/ade7f55))
- Reorganise for `pull_request` event ([#16](https://github.com/catppuccin/rockdove/issues/16)) ([0a7d594](https://github.com/catppuccin/rockdove/commit/0a7d594))
- Reorganise for `repository` event ([#15](https://github.com/catppuccin/rockdove/issues/15)) ([7417eaf](https://github.com/catppuccin/rockdove/commit/7417eaf))
- Reorganise tests for `issues` event ([#14](https://github.com/catppuccin/rockdove/issues/14)) ([21ffc5a](https://github.com/catppuccin/rockdove/commit/21ffc5a))
- Add config to automate fixtures ([#13](https://github.com/catppuccin/rockdove/issues/13)) ([bf65d28](https://github.com/catppuccin/rockdove/commit/bf65d28))


### ⚙️ Miscellaneous Tasks

- Bump version to 0.15.2 ([7dc892a](https://github.com/catppuccin/rockdove/commit/7dc892a))



## [v0.15.1](https://github.com/catppuccin/rockdove/releases/tag/v0.15.1) - 2024-09-21


### 🐛 Bug Fixes

- Set pr reviews to the right colour ([ec48aa5](https://github.com/catppuccin/rockdove/commit/ec48aa5))



## [v0.15.0](https://github.com/catppuccin/rockdove/releases/tag/v0.15.0) - 2024-09-19


### 🐛 Bug Fixes

- Handle repo rename/transfer correctly ([e74e3b8](https://github.com/catppuccin/rockdove/commit/e74e3b8))



## [v0.14.0](https://github.com/catppuccin/rockdove/releases/tag/v0.14.0) - 2024-09-19


### 🚀 Features

- Refactor with octocrab ([#12](https://github.com/catppuccin/rockdove/issues/12)) ([7171150](https://github.com/catppuccin/rockdove/commit/7171150))


### ⚙️ Miscellaneous Tasks

- Bump version ([8482f94](https://github.com/catppuccin/rockdove/commit/8482f94))



## [v0.13.0](https://github.com/catppuccin/rockdove/releases/tag/v0.13.0) - 2024-09-18


### 🚀 Features

- Hash action too ([54309a5](https://github.com/catppuccin/rockdove/commit/54309a5))



## [v0.12.0](https://github.com/catppuccin/rockdove/releases/tag/v0.12.0) - 2024-09-18


### 🚀 Features

- Use catppuccin colours in embeds ([#11](https://github.com/catppuccin/rockdove/issues/11)) ([314dc17](https://github.com/catppuccin/rockdove/commit/314dc17))
- Support team `membership` changes ([#10](https://github.com/catppuccin/rockdove/issues/10)) ([0cb0c78](https://github.com/catppuccin/rockdove/commit/0cb0c78))


### ⚙️ Miscellaneous Tasks

- Bump version ([cca0a32](https://github.com/catppuccin/rockdove/commit/cca0a32))



## [v0.11.0](https://github.com/catppuccin/rockdove/releases/tag/v0.11.0) - 2024-09-18


### 🚀 Features

- Support `pull_request_review` ([#7](https://github.com/catppuccin/rockdove/issues/7)) ([1f79c60](https://github.com/catppuccin/rockdove/commit/1f79c60))


### 🐛 Bug Fixes

- `issue_comment` can also be on pull requests ([#8](https://github.com/catppuccin/rockdove/issues/8)) ([547c580](https://github.com/catppuccin/rockdove/commit/547c580))


### 🚜 Refactor

- Ignore most issue events ([#6](https://github.com/catppuccin/rockdove/issues/6)) ([4ad1880](https://github.com/catppuccin/rockdove/commit/4ad1880))


### ⚙️ Miscellaneous Tasks

- Bump to 0.11.0 ([#9](https://github.com/catppuccin/rockdove/issues/9)) ([3caef8d](https://github.com/catppuccin/rockdove/commit/3caef8d))



## [v0.10.0](https://github.com/catppuccin/rockdove/releases/tag/v0.10.0) - 2024-09-16


### 🐛 Bug Fixes

- Silly mistake & run tests in CI ([bb86bb8](https://github.com/catppuccin/rockdove/commit/bb86bb8))
- Replace underscores instead of hyphens in actions ([#5](https://github.com/catppuccin/rockdove/issues/5)) ([b5d4ec5](https://github.com/catppuccin/rockdove/commit/b5d4ec5))


### 🚜 Refactor

- (PR): Ignore events besides `opened`, `closed` & `reopened` ([#4](https://github.com/catppuccin/rockdove/issues/4)) ([9465512](https://github.com/catppuccin/rockdove/commit/9465512))


### ⚙️ Miscellaneous Tasks

- Bump version ([9c1ecbc](https://github.com/catppuccin/rockdove/commit/9c1ecbc))



## [v0.9.0](https://github.com/catppuccin/rockdove/releases/tag/v0.9.0) - 2024-09-15


### 🚀 Features

- Replace - with space in actions ([0652db0](https://github.com/catppuccin/rockdove/commit/0652db0))



## [v0.8.0](https://github.com/catppuccin/rockdove/releases/tag/v0.8.0) - 2024-09-15


### 🚀 Features

- Ignore all edited events ([3051a02](https://github.com/catppuccin/rockdove/commit/3051a02))



## [v0.7.0](https://github.com/catppuccin/rockdove/releases/tag/v0.7.0) - 2024-09-15


### 🐛 Bug Fixes

- Limit length of `description` embed field ([#3](https://github.com/catppuccin/rockdove/issues/3)) ([5773bf0](https://github.com/catppuccin/rockdove/commit/5773bf0))


### ⚙️ Miscellaneous Tasks

- Bump version number ([c17c54f](https://github.com/catppuccin/rockdove/commit/c17c54f))



## [v0.6.0](https://github.com/catppuccin/rockdove/releases/tag/v0.6.0) - 2024-09-14


### 🚀 Features

- Maybe fix comments a bit ([c93ae01](https://github.com/catppuccin/rockdove/commit/c93ae01))


### ⚙️ Miscellaneous Tasks

- Bump version number ([2e0b16e](https://github.com/catppuccin/rockdove/commit/2e0b16e))



## [v0.5.1](https://github.com/catppuccin/rockdove/releases/tag/v0.5.1) - 2024-09-14


### ⚙️ Miscellaneous Tasks

- Bump to 0.5.1 ([#2](https://github.com/catppuccin/rockdove/issues/2)) ([2f023a8](https://github.com/catppuccin/rockdove/commit/2f023a8))



## [v0.5.0](https://github.com/catppuccin/rockdove/releases/tag/v0.5.0) - 2024-09-14


### 📚 Documentation

- (readme): Rename ([5711bd7](https://github.com/catppuccin/rockdove/commit/5711bd7))


### ⚙️ Miscellaneous Tasks

- Bump version number ([b561721](https://github.com/catppuccin/rockdove/commit/b561721))


### Build

- "waaaahhh" - @sgoudham ([f2ac2e6](https://github.com/catppuccin/rockdove/commit/f2ac2e6))
- Rename package ([7055b8f](https://github.com/catppuccin/rockdove/commit/7055b8f))



## [v0.4.0](https://github.com/catppuccin/rockdove/releases/tag/v0.4.0) - 2024-06-11


### 🚀 Features

- Implement all the github events we care about ([17e4016](https://github.com/catppuccin/rockdove/commit/17e4016))


### ⚙️ Miscellaneous Tasks

- Bump version number ([b6993eb](https://github.com/catppuccin/rockdove/commit/b6993eb))
- Enable & fix clippy lints ([5ae4923](https://github.com/catppuccin/rockdove/commit/5ae4923))



## [v0.3.0](https://github.com/catppuccin/rockdove/releases/tag/v0.3.0) - 2024-06-11


### 🚀 Features

- Make port configurable ([909ec81](https://github.com/catppuccin/rockdove/commit/909ec81))


### 📚 Documentation

- Add readme ([3f0c314](https://github.com/catppuccin/rockdove/commit/3f0c314))


### ⚙️ Miscellaneous Tasks

- Bump version number ([b147b61](https://github.com/catppuccin/rockdove/commit/b147b61))
- Add license ([7cce859](https://github.com/catppuccin/rockdove/commit/7cce859))



## [v0.2.0](https://github.com/catppuccin/rockdove/releases/tag/v0.2.0) - 2024-06-11


### 🚀 Features

- Support separate bot hook ([75894ed](https://github.com/catppuccin/rockdove/commit/75894ed))


### ⚙️ Miscellaneous Tasks

- Bump version number ([f3bc5a8](https://github.com/catppuccin/rockdove/commit/f3bc5a8))



## [v0.1.0](https://github.com/catppuccin/rockdove/releases/tag/v0.1.0) - 2024-06-11


### 🚀 Features

- Initial commit ([d1491d1](https://github.com/catppuccin/rockdove/commit/d1491d1))


### ⚙️ Miscellaneous Tasks

- Add release workflow ([87ec05a](https://github.com/catppuccin/rockdove/commit/87ec05a))



