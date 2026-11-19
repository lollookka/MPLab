# MPLab: Multi-Language Programming Lab

WIF3010 Programming Language Paradigms, Semester 1 2026/2027, Universiti Malaya

MPLab is a web platform where users can write, run and test programs in ten programming languages, and lecturers can create exercises and assess submissions.

## Start Here

- **[API_SPEC.md](API_SPEC.md)**: the execution API and integration rules every group must follow. Read this before writing any code.

## Groups

| Group | Language | Folder | Shared responsibility |
|---|---|---|---|
| 1 | C | `languages/c/` | Shared API specification and integration standards |
| 2 | C++ | `languages/cpp/` | Execution controller and language selection (`controller/`) |
| 3 | Java | `languages/java/` | Shared web interface and code editor (`frontend/`) |
| 4 | Python | `languages/python/` | Lecturer question management (`services/lecturer/`) |
| 5 | JavaScript | `languages/javascript/` | Automated assessment and test-case execution (`services/assessment/`) |
| 6 | Go | `languages/go/` | Student submissions and results management (`services/submissions/`) |
| 7 | Ruby | `languages/ruby/` | Integration testing and quality assurance (`tests/`) |
| 8 | PHP | `languages/php/` | Error reporting and execution diagnostics (`services/diagnostics/`) |
| 9 | Haskell | `languages/haskell/` | Programming language comparison features (`services/comparison/`) |
| 10 | Prolog | `languages/prolog/` | System documentation and deployment coordination (`docs/`) |

## Quick Rules

1. Never push directly to `main`. Work on a branch named `group<number>/<short-description>`, for example `group1/c-executor`.
2. Open a pull request (PR) to merge into `main`. Start the title with your group number and a colon, for example `G1: Add GCC compile step`.
3. Changes only inside your own `languages/` folder can be approved by your own group. Changes to shared folders need approval from at least one member of another group.
4. Only edit folders your group owns, unless you have agreed with the owning group.

Full details are in [API_SPEC.md](API_SPEC.md), Section 5.

## Security Reminder

This repository is public. **Never commit passwords, server login details, API keys or VM access information.** Share those privately with the people who need them.

## Contact

Questions about the API or integration rules: Group 1 leader.
