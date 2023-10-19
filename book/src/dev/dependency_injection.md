<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Dependency Injection for card reader
To build with NDA features (GPARK Limited or own secret dependency), need adding following command on each `cargo` command.
build, run or any other `cargo` command.

```sh
# dependency injection from git repository
# CAUTION , this should be work but not working
--config "patch.'https://github.com/pmnxis/billmock-app-rs.git'.billmock-plug-card.git = \"https://github.com/user_name/repo_name.git\""

# dependency injection from local repository
# this works
--config "patch.'https://github.com/pmnxis/billmock-app-rs.git'.billmock-plug-card.path = \"../repo_name\""
```

In this repository, experimentally utilize dependency injection that the 'patch' function of 'cargo' to coexist both NDA code and open source example code.
