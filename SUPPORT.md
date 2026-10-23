# Support

7-Zip Hatch is an independent, volunteer-maintained desktop preview. Replies
depend on maintainer availability. There is no paid support or response SLA.

| Need | Where to go |
| --- | --- |
| Downloads, requirements, verification, removal | [Installation guide](Hatch/DOWNLOADS.md) |
| Desktop options and integration | [Desktop guide](Hatch/DESKTOP.md) |
| CLI, format limits, and recovery | [CLI guide](Hatch/CLI.md) |
| Questions and usage help | [GitHub Discussions](https://github.com/chai-in/7zip-hatch/discussions) |
| A reproducible bug | [Bug report](https://github.com/chai-in/7zip-hatch/issues/new?template=bug_report.yml) |
| A concrete feature proposal | [Feature request](https://github.com/chai-in/7zip-hatch/issues/new?template=feature_request.yml) |
| A security vulnerability | [Private security reporting](SECURITY.md) |
| Conduct concerns | [Code of conduct and reporting](CODE_OF_CONDUCT.md#reporting) |

Before posting, search existing issues and include the project version,
package name, OS, archive format, operation, exact error, and minimal steps.
Use a disposable archive to reproduce a problem. Redact passwords, tokens,
personal paths, and private filenames from logs and screenshots.

For a paused consuming extraction, retain the remaining archive, its
`.hatchzip` journal, and `.hatchzip-tail` file. Do not delete recovery files
or repeatedly experiment on the only remaining copy while seeking help.
Partial restore rebuilds the unextracted remainder; it is not a full undo.

Report fork-specific behavior here. The upstream projects do not maintain
this fork or its installers.
