"""Command line interface.

    python -m doubao_skin list
    python -m doubao_skin apply <theme-id|theme-dir>
    python -m doubao_skin remove
"""
import argparse
import sys

from . import __version__, build, theme as theme_mod


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        prog="doubao-skin",
        description="Give the DoubaoWork desktop app a new skin.",
    )
    parser.add_argument("--version", action="version", version=__version__)
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("list", help="list bundled themes")
    p_apply = sub.add_parser("apply", help="build the skinned app with a theme")
    p_apply.add_argument("theme", help="theme id or path to a theme directory")
    sub.add_parser("remove", help="delete the skinned app")
    args = parser.parse_args(argv)

    if args.cmd == "list":
        for t in theme_mod.list_themes():
            icon = " +icon" if t.icon else ""
            print(f"{t.id:<16} {t.name}  {t.description}{icon}")
        return 0
    if args.cmd == "apply":
        build.apply(theme_mod.load(args.theme))
        return 0
    if args.cmd == "remove":
        build.remove()
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
