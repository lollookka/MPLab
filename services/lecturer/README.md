# Lecturer Question Management — MPLab

**Course:** WIF3010 Programming Language Paradigms  
**Project:** MPLab — Multi-Language Programming Lab  
**Group:** Group 4 — Team Fahh  
**Component:** Lecturer Question Management  
**Academic Session:** Semester 1, 2026/2027  
**Institution:** Universiti Malaya

## 1. Overview

Lecturer Question Management is Group 4's shared-platform responsibility in the MPLab project.

This component is intended to allow lecturers to create, retrieve, update and delete programming questions. It will also provide validation and storage functionality to support question management within the integrated MPLab system.

The component will be developed under `services/lecturer/` and coordinated with other groups responsible for the frontend, automated assessment and student submissions.

**Current Status:** Initial setup. Implementation and testing are pending.

## 2. Group Members and Responsibilities

Group 4 consists of six members.

| No. | Member | Overall Project Responsibilities |
|---|---|---|
| 1 | Shaif Ahmad Bin Tufail Ahmad | Task E Integration and Testing, Project Coordination |
| 2 | Keertti A/P Rasanran | Task A Python Investigation, Question Creation and Editing |
| 3 | Sanchay A/L Ravindran | Task C Output/Error Handling, Question Retrieval and Deletion |
| 4 | Yashveena S. Jayaganthan | Task D Python Demonstrations, Task E Testing |
| 5 | Maredu Anes | Task B CPython Installation, Task C Interpreter Execution |
| 6 | Liang Bohao | Lecturer Question Validation, Storage and Quality Assurance |

### Lecturer Question Management Assignments

| Member | Assigned Work |
|---|---|
| Keertti A/P Rasanran | Question creation and editing |
| Sanchay A/L Ravindran | Question retrieval and deletion |
| Liang Bohao | Question validation, database storage and QA |
| Shaif Ahmad Bin Tufail Ahmad | Integration coordination |

The other members contribute primarily to the Python Language Module documented under `languages/python/`.

## 3. Planned Features

| Feature | Description |
|---|---|
| Create Question | Add new programming questions |
| Retrieve Questions | Retrieve stored programming questions |
| Update Question | Edit an existing question |
| Delete Question | Remove an existing question |
| Question Validation | Check the validity of question information |
| Question Storage | Store, update and retrieve question records |

The final question fields and service behavior will be confirmed during integration planning.

## 4. Proposed Technology

| Component | Proposed Technology |
|---|---|
| Backend | Python |
| Database | SQLite |
| Communication | REST API |

These technologies are proposed for development and are not mandated by the execution API specification.

The database schema and endpoints must be agreed upon with the groups that will consume the question management service.

## 5. Planned Folder Structure

```text
services/lecturer/
├── README.md
├── question_service.py
├── question_queries.py
├── question_validation.py
├── question_storage.py
└── tests/
    └── test_lecturer_questions.py
```

This structure is provisional. Implementation files and tests will be added by the assigned members.

### File Responsibilities

| File | Purpose | Assigned Member |
|---|---|---|
| `README.md` | Component documentation | Shaif |
| `question_service.py` | Question creation and editing | Keertti |
| `question_queries.py` | Question retrieval and deletion | Sanchay |
| `question_validation.py` | Question validation | Liang |
| `question_storage.py` | SQLite database operations | Liang |
| `tests/test_lecturer_questions.py` | Component testing and QA | Liang, with integration coordination by Shaif |

## 6. Proposed Component Workflow

The intended workflow is:

1. A lecturer accesses the question management interface.
2. The interface sends a request to the lecturer question service.
3. The service validates the submitted question information.
4. The appropriate create, retrieve, update or delete operation is performed.
5. The database stores or retrieves the relevant records.
6. The service returns the result to the calling component.

The exact request and response structures will be defined during API coordination.

## 7. Question Storage and Validation

### 7.1 Database Storage

SQLite is the proposed database technology.

The final database design will be developed to support the question information required by the lecturer interface and assessment system.

Database table names, attributes, constraints and relationships are pending agreement.

### 7.2 Input Validation

Question validation is intended to check:

- Required question information.
- Supported programming language values.
- Valid data types and formats.
- Missing or invalid question identifiers.
- Data consistency.

Additional validation rules will be determined after the question schema is finalized.

## 8. Integration with Other Groups

| Group | Component | Integration Purpose |
|---|---|---|
| Group 3 | Frontend and code editor | Lecturer question management interface |
| Group 5 | Automated assessment | Programming questions and associated test cases |
| Group 6 | Student submissions | Question references and submission relationships, where required |

The interfaces must be confirmed with the relevant groups before integration.

The Group 1 execution API specification defines the code execution interface, not the lecturer question management REST endpoints.

Therefore, question management endpoints will be proposed and agreed upon separately.

## 9. Testing Plan

The planned tests include:

| Test Case | Expected Result |
|---|---|
| Create a valid question | Question successfully stored |
| Retrieve an existing question | Correct question returned |
| Update an existing question | Stored information updated |
| Delete an existing question | Question removed according to agreed behavior |
| Submit invalid question data | Validation error returned |
| Retrieve an unknown question | Appropriate not-found response |
| Database operation failure | Error handled appropriately |
| Integration with lecturer interface | Requests and responses are compatible |

Testing evidence and results will be added after implementation.

**Testing Status:** Pending.

## 10. GitHub Development and Integration

**Official Repository:** https://github.com/lollookka/MPLab

**Development Branch:** `group4/lecturer-questions`

**Assigned Folder:** `services/lecturer/`

### Contribution Rules

1. Work on a development branch rather than directly on `main`.
2. Keep changes within Group 4's assigned folders.
3. Commit changes using descriptive messages.
4. Test changes before requesting integration.
5. Open pull requests with titles beginning `G4:`.
6. Obtain the required approval before merging.

According to the official MPLab API specification, changes to shared service folders require approval from at least one member of another group.

## 11. Development Status

**Current Phase:** Initial Setup

| Component | Status |
|---|---|
| Component README | Initial documentation |
| Folder structure | Planned |
| Question creation and editing | Pending implementation |
| Question retrieval and deletion | Pending implementation |
| Question validation | Pending implementation |
| SQLite database storage | Pending implementation |
| REST API endpoints | Pending interface agreement |
| Testing and QA | Pending implementation |
| Integration with other groups | Pending coordination |

This document describes the planned component and does not claim that unfinished features have been implemented, deployed or successfully tested.

## 12. References

- [Official MPLab README](../../README.md)
- [Official MPLab API Specification](../../API_SPEC.md)
- [Python Execution Module](../../languages/python/README.md)

---

**Group 4 — Team Fahh**  
WIF3010 Programming Language Paradigms  
Universiti Malaya  
Semester 1, 2026/2027
