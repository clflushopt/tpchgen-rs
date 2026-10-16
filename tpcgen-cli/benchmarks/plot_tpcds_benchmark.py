#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["matplotlib>=3.8"]
# ///
"""Plot the CSV written by tpcds_benchmark.sh as two stacked bar charts.

    uv run plot_tpcds_benchmark.py tpcds_benchmark.csv [--output-dir DIR] [--subtitle TEXT]

Writes tpcds-tbl-performance.png and tpcds-parquet-performance.png.
Rows with status != ok are ignored; wall_sec is averaged over runs.
"""

import argparse
import csv
from collections import defaultdict
from pathlib import Path

import matplotlib.pyplot as plt

BLUE, ORANGE, AQUA = "#2a78d6", "#eb6834", "#1baf7a"
INK, MUTED, GRID = "#0b0b0b", "#52514e", "#e6e5e2"


def fmt_secs(s):
    if s < 60:
        return f"{s:.1f} s" if s < 10 else f"{s:.0f} s"
    m, sec = divmod(round(s), 60)
    return f"{m}m {sec:02d}s"


def load(path):
    """-> {(format, tool, step, sf): mean wall_sec}, subtitle"""
    sums, counts, hosts, threads = defaultdict(float), defaultdict(int), set(), set()
    with open(path, newline="") as f:
        for r in csv.DictReader(f):
            if r["status"] != "ok":
                continue
            key = (r["format"], r["tool"], r["step"], float(r["scale_factor"]))
            sums[key] += float(r["wall_sec"])
            counts[key] += 1
            hosts.add(r["host"])
            threads.add(r["threads"])
    times = {k: sums[k] / counts[k] for k in sums}
    subtitle = f"{'/'.join(sorted(threads))} threads per generator, {', '.join(sorted(hosts))}"
    return times, subtitle


def new_figure(title, subtitle):
    fig, ax = plt.subplots(figsize=(13, 6.5), dpi=150)
    fig.subplots_adjust(top=0.80, bottom=0.14, left=0.09, right=0.98)
    fig.text(0.09, 0.95, title, fontsize=15, color=INK, ha="left", va="top")
    fig.text(0.09, 0.895, subtitle, fontsize=9.5, color=MUTED, ha="left", va="top")
    for side in ("top", "right"):
        ax.spines[side].set_visible(False)
    ax.spines["left"].set_color(GRID)
    ax.spines["bottom"].set_color(GRID)
    ax.yaxis.grid(True, color=GRID, linewidth=0.8)
    ax.set_axisbelow(True)
    ax.tick_params(colors=MUTED, length=0)
    ax.set_ylabel("Time (seconds)", color=MUTED)
    ax.set_xlabel("TPC-DS scale factor", color=MUTED)
    return fig, ax


def vs_fastest(y, best):
    """'\n(3.2x)' relative to the fastest bar in the group; '' for the fastest."""
    if not y or not best or y <= best:
        return ""
    r = y / best
    return f"\n({r:.1f}x)" if r < 9.95 else f"\n({r:.0f}x)"


def label(ax, x, y, text, color=INK):
    ax.annotate(text, (x, y), xytext=(0, 3), textcoords="offset points",
                ha="center", va="bottom", fontsize=8, linespacing=1.1, color=color)


def plot_tbl(times, sfs, subtitle, out):
    tools = [("tpcgen-cli", "tpcgen-cli", BLUE), ("dsdgen", "dsdgen (C)", ORANGE), ("trino", "Trino (Java)", AQUA)]
    fig, ax = new_figure("TPC-DS tbl generation time", subtitle)
    width = 0.3
    ys_by_tool = {tool: [times.get(("tbl", tool, "generate", sf), 0) for sf in sfs] for tool, _, _ in tools}
    best = [min((ys[j] for ys in ys_by_tool.values() if ys[j]), default=0) for j in range(len(sfs))]
    for i, (tool, name, color) in enumerate(tools):
        xs = [j + (i - 1) * width for j in range(len(sfs))]
        ys = ys_by_tool[tool]
        ax.bar(xs, ys, width - 0.02, color=color, label=name)
        for j, (x, y) in enumerate(zip(xs, ys)):
            label(ax, x, y, fmt_secs(y) + vs_fastest(y, best[j]) if y else "n/a", INK if y else MUTED)
    finish(fig, ax, sfs, out)


def plot_parquet(times, sfs, subtitle, out):
    fig, ax = new_figure("TPC-DS Parquet generation time", subtitle)
    width = 0.32
    xs_t = [j - width / 2 for j in range(len(sfs))]
    xs_d = [j + width / 2 for j in range(len(sfs))]
    gen = [times.get(("parquet", "tpcgen-cli", "generate", sf), 0) for sf in sfs]
    d_gen = [times.get(("parquet", "duckdb", "generate", sf), 0) for sf in sfs]
    d_conv = [times.get(("parquet", "duckdb", "convert", sf)) for sf in sfs]
    ax.bar(xs_t, gen, width - 0.02, color=BLUE, label="tpcgen-cli")
    ax.bar(xs_d, d_gen, width - 0.02, color=ORANGE, label="DuckDB: CALL dsdgen()")
    ax.bar(xs_d, [c or 0 for c in d_conv], width - 0.02, bottom=d_gen, color=AQUA,
           label="DuckDB: EXPORT DATABASE (parquet)")
    d_tot = [g + (c or 0) for g, c in zip(d_gen, d_conv)]
    best = [min((v for v in (t, d) if v), default=0) for t, d in zip(gen, d_tot)]
    for x, y, b in zip(xs_t, gen, best):
        label(ax, x, y, fmt_secs(y) + vs_fastest(y, b) if y else "n/a", INK if y else MUTED)
    for x, g, c, t, b in zip(xs_d, d_gen, d_conv, d_tot, best):
        if not g:
            label(ax, x, 0, "n/a", MUTED)
        elif c is None:
            label(ax, x, g, fmt_secs(g) + vs_fastest(g, b) + "\nexport failed")
        else:
            label(ax, x, t, fmt_secs(t) + vs_fastest(t, b) + f"\nexport {fmt_secs(c)}")
    finish(fig, ax, sfs, out)


def finish(fig, ax, sfs, out):
    ax.set_xticks(range(len(sfs)))
    ax.set_xticklabels([f"{sf:g}" for sf in sfs])
    ax.legend(loc="lower left", bbox_to_anchor=(0, 1.0), ncol=3, frameon=False, fontsize=9,
              labelcolor=MUTED, handlelength=1, handleheight=1)
    fig.savefig(out)
    plt.close(fig)
    print(f"wrote {out}")


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("csv", type=Path)
    p.add_argument("--output-dir", type=Path, default=Path("."))
    p.add_argument("--subtitle", help="override the generated subtitle (threads, host)")
    args = p.parse_args()

    times, subtitle = load(args.csv)
    subtitle = args.subtitle or subtitle
    sfs = sorted({k[3] for k in times})
    plot_tbl(times, sfs, subtitle, args.output_dir / "tpcds-tbl-performance.png")
    plot_parquet(times, sfs, subtitle, args.output_dir / "tpcds-parquet-performance.png")


if __name__ == "__main__":
    main()
