"""Smoke tests: imports work, CLI --help works."""

from click.testing import CliRunner

from mermaid_ascii.__main__ import main


def test_import():
    from mermaid_ascii.api import render_dsl

    assert render_dsl is not None


def test_cli_help():
    runner = CliRunner()
    result = runner.invoke(main, ["--help"])
    assert result.exit_code == 0
    assert "Mermaid flowchart" in result.output
