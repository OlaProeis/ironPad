"""PRD-0.4.0 §11 acceptance items must appear in CHANGELOG [0.4.0]."""

from __future__ import annotations

import re
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
CHANGELOG = ROOT / "CHANGELOG.md"

# Each tuple is (PRD §11 summary, one or more substrings required in the 0.4.0 section).
ACCEPTANCE_PHRASES: list[tuple[str, tuple[str, ...]]] = [
    (
        "Port: default scan 37891–37990 and IRONPAD_PORT pins a port",
        ("37891", "37990", "IRONPAD_PORT"),
    ),
    (
        "Port: startup logs the URL to open",
        ("Startup logs the URL",),
    ),
    (
        "Task view sorts by created, due date, or priority across reload",
        ("Task list sort", "due date", "priority", "localStorage", "reload"),
    ),
    (
        "Subtasks stay under their parent",
        ("Subtasks stay under their parent",),
    ),
    (
        "Sidebar overdue count; due today is not overdue",
        ("Sidebar overdue", "today", "not overdue"),
    ),
    (
        "Dashboard and sidebar agree on overdue count",
        ("sidebar and dashboard agree",),
    ),
    (
        "Dashboard tag chips filter; clearing restores full dashboard",
        ("Dashboard filter by task tag", "restores the full dashboard"),
    ),
    (
        "Day view used/budget, 6h default, amber over budget, remembers budget",
        ("Fit the day", "6 hours", "amber", "localStorage"),
    ),
    (
        "Bulk mark-complete skips done tasks; recurring spawn",
        ("Bulk task actions", "already-completed", "recurring"),
    ),
    (
        "Bulk move/tag consistency; stop on first failure",
        ("completed", "section", "sequentially", "first failure"),
    ),
    (
        "Save private HTTPS/SSH origin; reject embedded credentials",
        ("HTTPS or SSH", "origin", "embedded credentials are rejected"),
    ),
    (
        "Pull fast-forward; diverged history unchanged",
        ("fast-forward", "diverged histories change no files"),
    ),
    (
        "Auth, push, pull, dirty-tree errors (PRD §9.5)",
        ("auth failure", "non-fast-forward push", "non-fast-forward pull", "dirty tree"),
    ),
    (
        "Ahead/behind reflect upstream after push",
        ("ahead/behind", "upstream"),
    ),
    (
        "docs/git-backup.md linked; not live co-editing",
        ("git-backup.md", "not live co-editing"),
    ),
]


def _changelog_0_4_section() -> str:
    text = CHANGELOG.read_text(encoding="utf-8")
    match = re.search(
        r"^## \[0\.4\.0\].*?$([\s\S]*?)(?=^## \[|\Z)",
        text,
        flags=re.MULTILINE,
    )
    assert match is not None, "CHANGELOG.md is missing a ## [0.4.0] section"
    return match.group(1).casefold()


@pytest.fixture(scope="module")
def section_0_4() -> str:
    return _changelog_0_4_section()


@pytest.mark.parametrize(
    "summary,phrases",
    ACCEPTANCE_PHRASES,
    ids=[item[0][:60] for item in ACCEPTANCE_PHRASES],
)
def test_prd_section_11_item_in_changelog_0_4(
    section_0_4: str, summary: str, phrases: tuple[str, ...]
) -> None:
    missing = [p for p in phrases if p.casefold() not in section_0_4]
    assert not missing, f"{summary}: missing in CHANGELOG [0.4.0]: {missing!r}"