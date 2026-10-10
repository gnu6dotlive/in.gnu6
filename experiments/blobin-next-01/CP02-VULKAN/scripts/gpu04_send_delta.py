#!/usr/bin/env python3
"""GPU-04 fixture sender: a send is not a renderer acceptance acknowledgement."""
import argparse
import json
import socket
from pathlib import Path


def main() -> None:
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("--socket", required=True, type=Path)
    cli.add_argument("--fixture", required=True, type=Path)
    cli.add_argument("--from-sequence", type=int, required=True)
    cli.add_argument("--to-sequence", type=int, required=True)
    cli.add_argument("--scene-time", type=float, required=True)
    cli.add_argument("--control-x", type=float, required=True)
    args = cli.parse_args()
    if args.to_sequence <= args.from_sequence:
        cli.error("sequence must advance")
    if not -20 <= args.control_x <= 20:
        cli.error("control x outside [-20,20]")
    if not 0 <= args.scene_time <= 1_000_000:
        cli.error("scene time out of range")
    doc = json.loads(args.fixture.read_text())
    if doc["schema"] != "blobin.gpu03.scene-stream.v1":
        cli.error("unsupported fixture")
    found = [e for e in doc["initial"]["entities"] if e["entityId"] == "cavity-control"]
    if len(found) != 1:
        cli.error("one cavity-control required")
    entity = json.loads(json.dumps(found[0]))
    entity["worldPose"]["position"]["x"] = args.control_x
    packet = {
        "schema": "blobin.gpu04.delta.v1",
        "delta": {
            "schemaVersion": 1,
            "fromSequence": args.from_sequence,
            "toSequence": args.to_sequence,
            "sceneTime": args.scene_time,
            "motionMode": "normal",
            "order": [e["entityId"] for e in doc["initial"]["entities"]],
            "births": [], "updates": [entity], "deaths": [],
        },
    }
    raw = json.dumps(packet, separators=(",", ":")).encode() + b"\n"
    if len(raw) > 8193:
        cli.error("packet too large")
    with socket.socket(socket.AF_UNIX) as conn:
        conn.settimeout(2)
        conn.connect(str(args.socket))
        conn.sendall(raw)
    print(f"GPU04_DELTA_SENT_UNACKNOWLEDGED sequence={args.to_sequence} bytes={len(raw)}")


if __name__ == "__main__":
    main()
