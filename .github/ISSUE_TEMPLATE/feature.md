---

name: Feature Request
description: Suggest an idea or enhancement
title: "[FEATURE] "
labels: ["enhancement", "needs-triage"]
body:

- type: textarea
  id: description
  attributes:
  label: Feature Description
  description: A clear and concise description of the feature
  placeholder: What would you like to see added?
  validations:
  required: true

- type: textarea
  id: motivation
  attributes:
  label: Motivation / Use Case
  description: Why is this feature important? What problem does it solve?
  placeholder: As a [type of user], I want [feature] so that [benefit]
  validations:
  required: true

- type: textarea
  id: proposed_solution
  attributes:
  label: Proposed Solution
  description: How do you think this should be implemented?
  placeholder: Share your ideas on how this could work

- type: textarea
  id: alternatives
  attributes:
  label: Alternatives Considered
  description: Have you considered any alternative solutions or workarounds?
  placeholder: Describe any alternative solutions or features you've considered

- type: dropdown
  id: priority
  attributes:
  label: Priority
  description: How important is this feature?
  options: - Critical (blocking development/production) - High (important for next release) - Medium (nice to have) - Low (backlog/consider later)
  validations:
  required: true

- type: checkboxes
  id: contribution
  attributes:
  label: Contribution
  options: - label: I'm interested in contributing to this feature - label: I can help test this feature
