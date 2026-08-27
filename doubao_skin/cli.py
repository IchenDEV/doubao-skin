"""Command line interface.

    python -m doubao_skin list
    python -m doubao_skin apply <theme-id|theme-dir>
    python -m doubao_skin remove
    python -m doubao_skin live <theme> [--once] [--port 9222]
"""
import argparse
import sys

from . import __version__, build, live, theme as theme_mod


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
    p_live = sub.add_parser(
        "live", help="theme the ORIGINAL app at runtime via CDP (no file changes)")
    p_live.add_argument("theme", help="theme id or path to a theme directory")
    p_live.add_argument("--port", type=int, default=live.DEFAULT_PORT)
    p_live.add_argument("--once", action="store_true",
                        help="inject current pages once and exit (no watching)")
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
    if args.cmd == "live":
        live.run(theme_mod.load(args.theme), port=args.port, once=args.once)
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
