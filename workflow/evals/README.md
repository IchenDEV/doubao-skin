# Harness evals

Eval cases preserve real lessons from accepted work and incidents. Create one with `./scripts/devflow add-eval <slug> [origin]`; do not invent quota-filling cases.

Every active case defines:

- the original task or incident scenario;
- the behavior an implementation agent must preserve;
- a deterministic command or assertion when one exists;
- the manual, visual, or integration oracle for judgment that cannot be encoded safely;
- the dated result of each regression run.

`./scripts/test-devflow.sh` is the first continuous harness eval: it covers stage ordering, approvals, PR linkage, verification evidence, incident closure, and incident-to-eval linkage. Product incidents should add narrower code or integration regressions, not expand this script into a general test framework.
