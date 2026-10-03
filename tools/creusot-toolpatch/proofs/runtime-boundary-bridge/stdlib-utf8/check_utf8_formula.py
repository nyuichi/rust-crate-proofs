#!/usr/bin/env python3
"""Exhaustively compare the UTF-8 arithmetic model to Python's encoder."""

from pathlib import Path


def modeled_utf8(codepoint: int) -> bytes:
    if codepoint < 0x80:
        encoded = (codepoint,)
    elif codepoint < 0x800:
        encoded = (0xC0 + codepoint // 64, 0x80 + codepoint % 64)
    elif codepoint < 0x10000:
        encoded = (
            0xE0 + codepoint // 4096,
            0x80 + (codepoint // 64) % 64,
            0x80 + codepoint % 64,
        )
    else:
        encoded = (
            0xF0 + codepoint // 262144,
            0x80 + (codepoint // 4096) % 64,
            0x80 + (codepoint // 64) % 64,
            0x80 + codepoint % 64,
        )
    return bytes(encoded)


def main() -> None:
    checked = 0
    for codepoint in range(0x110000):
        if 0xD800 <= codepoint <= 0xDFFF:
            continue
        expected = chr(codepoint).encode("utf-8")
        actual = modeled_utf8(codepoint)
        if actual != expected:
            raise SystemExit(
                f"mismatch at U+{codepoint:04X}: {actual!r} != {expected!r}"
            )
        checked += 1

    result = (
        "UTF-8 arithmetic model exhaustive oracle check: PASS\n"
        f"Unicode scalar values checked: {checked:,}\n"
        'Oracle: Python chr(codepoint).encode("utf-8")\n'
        "Surrogate code points excluded, as Rust char excludes them.\n"
        "No third-party dependencies.\n"
    )
    output = Path(__file__).with_name("check_utf8_formula.txt")
    output.write_text(result)
    print(result, end="")


if __name__ == "__main__":
    main()
