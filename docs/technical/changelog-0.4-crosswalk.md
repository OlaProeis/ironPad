# Release 0.4.0 changelog and PRD acceptance crosswalk

Release **0.4.0** shipped on 2026-11-20. User-facing notes live in [`CHANGELOG.md`](../../CHANGELOG.md) under `## [0.4.0] - 2026-11-20`. That section lists sort, sidebar overdue, dashboard tag filter, fit the day, bulk actions, Git backup, the listen-port change, the Library, and the GitHub link.

## Automated crosswalk

[`tests/test_changelog_0_4_crosswalk.py`](../../tests/test_changelog_0_4_crosswalk.py) parametrizes one pytest per PRD §11 acceptance theme. Each test requires every phrase in its tuple to appear (case-insensitive) inside the `## [0.4.0]` block only—not in older release sections.

When you extend PRD §11 or edit the 0.4.0 changelog bullets, update `ACCEPTANCE_PHRASES` in that file so the phrases still match that section. The heading is dated 2026-11-20.

## Running the check

Targeted run (preferred for agents):

```bash
python -m pytest tests/test_changelog_0_4_crosswalk.py -q
```

Cyclopsctl’s project verification runs `python -m pytest` from the repo root; this module is what prevents exit code 5 (no tests collected) for doc-only release tasks.

## Related

- Acceptance source: [`PRD-0.4.0.md`](../ai-workflow/PRD-0.4.0.md) §11  
- Planning alignment: [`planning-docs-0.4.md`](../ai-workflow/planning-docs-0.4.md)  
- Human-readable history: [`CHANGELOG.md`](../../CHANGELOG.md)