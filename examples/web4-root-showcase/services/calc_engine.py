import json
import sys


def main() -> None:
    mode = sys.argv[1] if len(sys.argv) > 1 else "open"
    payload = json.load(sys.stdin)

    a = int(payload.get("a", 0))
    b = int(payload.get("b", 0))
    theme = payload.get("theme") or "aurora"
    total = a + b

    messages = {
        "open": f"Open channel online: {a} + {b} = {total} ({theme})",
        "interactive": f"Interactive approval sealed: {a} + {b} = {total} ({theme})",
        "shadow": f"Shadow lane kept silent: {a} + {b} = {total} ({theme})",
    }

    print(
        json.dumps(
            {
                "sum": total,
                "narrative": messages.get(mode, messages["open"]),
                "engine": f"calc_engine/{mode}",
            }
        )
    )


if __name__ == "__main__":
    main()
