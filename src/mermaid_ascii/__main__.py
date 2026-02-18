"""CLI entry point for mermaid-ascii."""

import sys

import click

from mermaid_ascii.api import render_dsl


@click.command()
@click.argument("input", required=False, type=click.Path(exists=True))
@click.option("--ascii", "-a", "use_ascii", is_flag=True, help="Use plain ASCII instead of Unicode")
@click.option("--direction", "-d", "direction", type=str, default=None, help="Override direction (LR, RL, TD, BT)")
@click.option("--padding", "-p", "padding", type=int, default=1, help="Node padding (spaces inside border)")
@click.option("--output", "-o", "output", type=str, default=None, help="Write output to this file instead of stdout")
def main(input: str | None, use_ascii: bool, direction: str | None, padding: int, output: str | None) -> None:
    """Mermaid flowchart to ASCII/Unicode graph output."""
    if input:
        try:
            with open(input) as f:
                text = f.read()
        except OSError as e:
            click.echo(f"error: cannot read '{input}': {e}", err=True)
            sys.exit(1)
    else:
        text = sys.stdin.read()

    try:
        rendered = render_dsl(text, unicode=not use_ascii, padding=padding, direction=direction)
    except ValueError as e:
        click.echo(f"error: {e}", err=True)
        sys.exit(1)

    if output:
        try:
            with open(output, "w") as f:
                f.write(rendered)
        except OSError as e:
            click.echo(f"error: cannot write '{output}': {e}", err=True)
            sys.exit(1)
    else:
        click.echo(rendered, nl=False)


if __name__ == "__main__":
    main()
