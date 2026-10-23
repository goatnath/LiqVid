#!/usr/bin/env python3
"""Run the shared Rust cavity solver and plot its measured reference comparison."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def plot_report(path):
    # Optional plotting dependency; the numerical runner itself only needs Rust.
    os.environ.setdefault("MPLCONFIGDIR", str(Path(tempfile.gettempdir()) / "liqvid-matplotlib"))
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    plt.rcParams["svg.hashsalt"] = "liqvid-cavity-re100"

    result = json.loads(path.read_text())
    profile, comparison = result["profile"], result["comparison"]
    history = result["history"] + [{
        "time": result["time"], "velocity_change_rate": result["velocity_change_rate"],
        "max_divergence": result["max_divergence"],
    }]
    n = result["dimensions"][0]
    fig, (flow, residual) = plt.subplots(1, 2, figsize=(10, 4.8), constrained_layout=True)
    flow.plot([p["u"] for p in profile], [p["y"] for p in profile], label=f"LiqVid {n} × {n} × 1", color="#1764a0", linewidth=2)
    flow.scatter([p["reference"] for p in comparison], [p["y"] for p in comparison], label="Ghia et al. (1982), Table I", color="#c34936", s=30, facecolors="none", zorder=3)
    flow.set(xlabel="Horizontal velocity u / U_lid", ylabel="y / L", ylim=(0, 1), title="Vertical centerline x/L = 0.5")
    flow.grid(alpha=0.2)
    flow.legend(fontsize=8, loc="lower right")
    residual.semilogy([p["time"] for p in history], [max(p["velocity_change_rate"], 1e-16) for p in history], label="max |Δu| / Δt", color="#1764a0")
    residual.semilogy([p["time"] for p in history], [max(p["max_divergence"], 1e-16) for p in history], label="max |div u|", color="#c34936")
    residual.axhline(result["criteria"]["steady_change_rate"], color="#1764a0", linestyle=":", label="Steady-change limit")
    residual.axhline(result["criteria"]["max_divergence"], color="#c34936", linestyle=":", label="Divergence limit")
    residual.set(xlabel="Time t U_lid / L", title="Steady-change and conservation checks")
    residual.grid(alpha=0.2)
    residual.legend(fontsize=8)
    status = "PASS" if result["passed"] else "FAIL"
    fig.suptitle(f"Lid-driven cavity · Re=100 · {status}\n"
                 f"max centerline error={result['max_centerline_error']:.5f} U_lid; "
                 f"max divergence={result['max_divergence']:.3e} U_lid/L", fontsize=12)
    fig.savefig(path.parent / "comparison.png", dpi=180, metadata={"Software": "LiqVid cavity benchmark"})
    fig.savefig(path.parent / "comparison.svg", metadata={"Date": None, "Creator": "LiqVid cavity benchmark"})
    svg = path.parent / "comparison.svg"
    svg.write_text("\n".join(line.rstrip() for line in svg.read_text().splitlines()) + "\n")
    plt.close(fig)
    # One row per published sample, ordered by increasing y.
    rows = ["y_over_L,u_over_U_lid,ghia_u_over_U_lid,absolute_error"]
    rows += [f"{p['y']:.8f},{p['simulated']:.10f},{p['reference']:.8f},{p['absolute_error']:.10f}" for p in sorted(comparison, key=lambda p: p['y'])]
    (path.parent / "comparison.csv").write_text("\n".join(rows) + "\n")
    summary = (f"Re=100, grid {n} × {n} × 1: {status}\n"
               f"Steady-change criterion: {result['converged']} after {result['steps']} steps (t={result['time']:.3f})\n"
               f"Maximum centerline error: {result['max_centerline_error']:.8f} (limit {result['criteria']['max_centerline_error']})\n"
               f"RMS centerline error: {result['rms_centerline_error']:.8f}\n"
               f"Maximum discrete divergence: {result['max_divergence']:.8e} (limit {result['criteria']['max_divergence']})\n")
    (path.parent / "summary.txt").write_text(summary)
    print(summary)
    return result["passed"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--n", type=int, default=32, help="even XY cell count (default: 32)")
    parser.add_argument("--dt", type=float, default=0.002)
    parser.add_argument("--max-steps", type=int, default=30000)
    parser.add_argument("--output", type=Path, default=Path("docs/validation/cavity-re100"))
    parser.add_argument("--plot-only", action="store_true", help="plot existing results.json without rerunning")
    parser.add_argument("--check", action="store_true", help="exit 2 if any benchmark criterion fails, after saving plots")
    args = parser.parse_args()
    output = args.output.resolve()
    if not args.plot_only:
        subprocess.run(["cargo", "run", "--release", "--example", "cavity", "--",
                        "--n", str(args.n), "--dt", str(args.dt), "--max-steps", str(args.max_steps),
                        "--output", str(output)], cwd=ROOT, check=True)
    passed = plot_report(output / "results.json")
    return 2 if args.check and not passed else 0


if __name__ == "__main__":
    sys.exit(main())
