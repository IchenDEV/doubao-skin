# Security policy

## Supported versions

Security fixes are provided for the latest release and the current `main` branch.

## Reporting a vulnerability

Use GitHub's private vulnerability reporting for this repository. Do not open a public issue for vulnerabilities, credentials, request captures, or conversation data.

Include the affected version, a minimal reproduction, impact, and suggested mitigation. Remove account identifiers, access tokens, cookies, message contents, and unrelated request fields before attaching logs or captures.

## Sensitive surfaces

The live theme mode and protocol bridge use a localhost Chrome DevTools Protocol endpoint. A process that can reach that endpoint may inspect or control the active application page. Run these features only on a trusted machine, keep the bridge bound to loopback, and stop it when testing is complete.

The project does not require or accept credentials for the user's DoubaoWork account. External model API keys must be supplied through local environment variables and must never be committed.
