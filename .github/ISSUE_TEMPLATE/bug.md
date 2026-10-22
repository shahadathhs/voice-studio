---

name: Bug Report
description: Report a problem or unexpected behavior
title: "[BUG] "
labels: ["bug", "needs-triage"]
body:

- type: textarea
  id: description
  attributes:
  label: Description
  description: A clear and concise description of what the bug is.
  placeholder: What happened? What did you expect to happen?
  validations:
  required: true

- type: textarea
  id: steps
  attributes:
  label: Steps to Reproduce
  description: Steps to reproduce the behavior
  placeholder: | 1. Go to '...' 2. Click on '....' 3. Scroll down to '....' 4. See error
  validations:
  required: true

- type: textarea
  id: expected
  attributes:
  label: Expected Behavior
  description: What you expected to happen
  validations:
  required: true

- type: textarea
  id: actual
  attributes:
  label: Actual Behavior
  description: What actually happened
  validations:
  required: true

- type: textarea
  id: environment
  attributes:
  label: Environment
  description: Environment information
  placeholder: | - OS: [e.g. Ubuntu 22.04] - Application Version: [e.g. 1.0.0] - Interface: [desktop app or CLI]
  validations:
  required: true

- type: textarea
  id: logs
  attributes:
  label: Logs/Screenshots
  description: Add any relevant logs, error messages, or screenshots
  render: shell

- type: checkboxes
  id: terms
  attributes:
  label: Confirmation
  options: - label: I've searched existing issues to avoid duplicates
  required: true
