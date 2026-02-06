---
name: rust-architecture-review
description: Review a Rust backend as a senior engineer and score architecture, software pattern usage, SOLID, DRY, and security out of 5.
---

# Rust Architecture Review Skill

## Goal
Provide a practical senior-level code review for Rust backend services with numeric scores and concrete findings.

When the request is `review @src/backend/rust/ in all aspects`, review the full backend surface:
- code architecture and design
- implementation quality
- testing and reliability
- performance and scalability risks
- observability and operations
- security posture

## Output Contract
Always return:
1. Findings first, ordered by severity (`high`, `medium`, `low`).
2. File references with line numbers.
3. Scores out of 5 for:
   - Architecture
   - Software pattern usage
   - SOLID
   - DRY
   - Security
4. One short rationale per score.
5. Top 3 next actions.
6. A short `All-Aspect Notes` section covering non-scored areas: testing, performance, observability, and deployment/runtime ops.

## Review Checklist

### 1) Architecture
- Layering clarity: API, service/domain, persistence, infra.
- Dependency direction and coupling.
- Boundaries around side effects (I/O vs pure logic).
- Runtime behavior under failure (fallbacks, partial writes, retries).

### 2) Software Pattern Usage
- Appropriate trait abstraction and dependency inversion.
- Repository/service patterns used consistently.
- Error handling pattern consistency (`Result`, typed errors).
- Concurrency/control patterns (optimistic locking, idempotency).

### 3) SOLID
- S: single responsibility of modules/structs.
- O: extension points without modifying core logic.
- L: trait impl substitutability.
- I: trait granularity (avoid fat interfaces).
- D: high-level logic depends on abstractions, not concretions.

### 4) DRY
- Duplicate orchestration logic across handlers/services.
- Repeated validation or mapping code.
- Reusable helper extraction opportunities.

### 5) Security
- AuthN/AuthZ correctness and fail-closed behavior.
- Session/cookie flags (`Secure`, `HttpOnly`, `SameSite`) by environment.
- CSRF/XSRF protections and origin checks.
- Input validation and bounds checks.
- Secrets/config validation and secure defaults.
- Dependency and runtime hardening signals.

## Scoring Guide (0-5)
- 5.0: exemplary, minimal risk.
- 4.0: strong with minor gaps.
- 3.0: acceptable, several meaningful improvements needed.
- 2.0: weak, notable design/risk issues.
- 1.0: poor, high likelihood of defects/security issues.
- 0.0: non-functional or critically unsafe.

## Severity Rubric
- high: can cause data loss, auth bypass, major security exposure, or production instability.
- medium: maintainability/performance/reliability issue with moderate impact.
- low: clarity or style issue, limited impact.

## Review Style
- Be direct and factual.
- Avoid generic praise.
- Prefer concrete, actionable fixes.
