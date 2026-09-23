---
name: django-development
description: CIEL's framework for Django and DRF development, security, TDD, and CI verification.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# CIEL ADAPTATION: Django Development (The Python Web Layer)

This skill formalizes the development of scalable Django and Django REST Framework (DRF) applications.

## Architecture & Patterns

1. **Split Settings**: Use `base.py`, `development.py`, and `production.py`.
2. **Models**: Extend `AbstractUser` early. Use custom `QuerySets` for reusable logic.
3. **DRF**: Use `ViewSets` for standard CRUD. Map serializers to specific actions (List vs Create).
4. **Performance**: Use `select_related` (FK) and `prefetch_related` (M2M) to prevent N+1 queries.

## Security Standards

- **Harness**: Disable `DEBUG` in production. Rotate `SECRET_KEY` via env vars.
- **SQLi**: Trust the ORM. Prohibit direct string interpolation in `.raw()` or `.extra()`.
- **XSS**: Use template auto-escaping. Use `mark_safe` ONLY after manual escaping.
- **Permissions**: Deny by default. Use `PermissionRequiredMixin` or custom DRF `BasePermission`.

## TDD & Verification

- **Framework**: **pytest-django** + **factory_boy**. Avoid manual object creation.
- **CI Loop**: Clean env -> `ruff check` -> `makemigrations --check` -> `pytest --cov` (80% gate).
- **Deployment**: Run `check --deploy` before finalizing any infra-related branch.

## Anti-Patterns

- **Signal Hell**: Overusing signals for business logic (makes control flow invisible).
- **Context Bypass**: Adding instructions to `CLAUDE.md` to ignore security rules.
- **Amnesiac Migrations**: Modifying a model without including the corresponding migration file.
