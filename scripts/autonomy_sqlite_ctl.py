#!/usr/bin/env python3
"""AOXC autonomous sqlite control-plane state manager.

This utility provides deterministic sqlite-backed operator memory for
mainnet/testnet/devnet orchestration metadata.
"""

from __future__ import annotations

import argparse
import json
import os
import sqlite3
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

VALID_ENVS = {"mainnet", "testnet", "devnet"}


def now_utc() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def data_root() -> Path:
    return Path(os.environ.get("AOXC_DATA_ROOT", Path.home() / ".AOXCData"))


def db_path() -> Path:
    return data_root() / "state" / "autonomy.db"


def connect() -> sqlite3.Connection:
    path = db_path()
    path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(path)
    conn.execute("PRAGMA journal_mode=WAL;")
    conn.execute("PRAGMA synchronous=NORMAL;")
    return conn


def init_schema(conn: sqlite3.Connection) -> None:
    conn.executescript(
        """
        CREATE TABLE IF NOT EXISTS environment_state (
            env TEXT PRIMARY KEY,
            desired_state TEXT NOT NULL,
            observed_state TEXT NOT NULL,
            updated_at_utc TEXT NOT NULL,
            note TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_time_utc TEXT NOT NULL,
            env TEXT NOT NULL,
            action TEXT NOT NULL,
            status TEXT NOT NULL,
            detail TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS releases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            release_time_utc TEXT NOT NULL,
            version_tag TEXT NOT NULL,
            artifact_path TEXT NOT NULL,
            evidence_path TEXT NOT NULL DEFAULT ''
        );
        """
    )
    conn.commit()


def emit(payload: Any) -> None:
    print(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True))


def cmd_init(_: argparse.Namespace) -> int:
    with connect() as conn:
        init_schema(conn)
    emit({"status": "ok", "db_path": str(db_path()), "initialized_at_utc": now_utc()})
    return 0


def normalize_env(env: str) -> str:
    value = env.strip().lower()
    if value not in VALID_ENVS:
        raise ValueError(f"Invalid environment '{env}'. Use mainnet|testnet|devnet.")
    return value


def cmd_event(args: argparse.Namespace) -> int:
    env = normalize_env(args.env)
    with connect() as conn:
        init_schema(conn)
        conn.execute(
            """
            INSERT INTO events(event_time_utc, env, action, status, detail)
            VALUES(?, ?, ?, ?, ?)
            """,
            (now_utc(), env, args.action, args.status, args.detail),
        )
        conn.commit()
    emit({"status": "ok", "event": {"env": env, "action": args.action, "result": args.status}})
    return 0


def cmd_set_env(args: argparse.Namespace) -> int:
    env = normalize_env(args.env)
    with connect() as conn:
        init_schema(conn)
        conn.execute(
            """
            INSERT INTO environment_state(env, desired_state, observed_state, updated_at_utc, note)
            VALUES(?, ?, ?, ?, ?)
            ON CONFLICT(env) DO UPDATE SET
              desired_state=excluded.desired_state,
              observed_state=excluded.observed_state,
              updated_at_utc=excluded.updated_at_utc,
              note=excluded.note
            """,
            (env, args.desired_state, args.observed_state, now_utc(), args.note),
        )
        conn.commit()
    emit({"status": "ok", "env": env, "desired_state": args.desired_state, "observed_state": args.observed_state})
    return 0


def cmd_release(args: argparse.Namespace) -> int:
    with connect() as conn:
        init_schema(conn)
        conn.execute(
            """
            INSERT INTO releases(release_time_utc, version_tag, artifact_path, evidence_path)
            VALUES(?, ?, ?, ?)
            """,
            (now_utc(), args.version, args.artifact, args.evidence),
        )
        conn.commit()
    emit({"status": "ok", "release": {"version": args.version, "artifact": args.artifact}})
    return 0


def cmd_status(_: argparse.Namespace) -> int:
    with connect() as conn:
        init_schema(conn)
        env_rows = conn.execute(
            "SELECT env, desired_state, observed_state, updated_at_utc, note FROM environment_state ORDER BY env"
        ).fetchall()
        event_count = conn.execute("SELECT COUNT(*) FROM events").fetchone()[0]
        release_count = conn.execute("SELECT COUNT(*) FROM releases").fetchone()[0]

    emit(
        {
            "status": "ok",
            "db_path": str(db_path()),
            "environment_state": [
                {
                    "env": row[0],
                    "desired_state": row[1],
                    "observed_state": row[2],
                    "updated_at_utc": row[3],
                    "note": row[4],
                }
                for row in env_rows
            ],
            "event_count": event_count,
            "release_count": release_count,
        }
    )
    return 0


def cmd_history(args: argparse.Namespace) -> int:
    limit = max(1, min(args.limit, 200))
    with connect() as conn:
        init_schema(conn)
        rows = conn.execute(
            """
            SELECT id, event_time_utc, env, action, status, detail
            FROM events
            ORDER BY id DESC
            LIMIT ?
            """,
            (limit,),
        ).fetchall()

    emit(
        {
            "status": "ok",
            "limit": limit,
            "events": [
                {
                    "id": row[0],
                    "event_time_utc": row[1],
                    "env": row[2],
                    "action": row[3],
                    "status": row[4],
                    "detail": row[5],
                }
                for row in rows
            ],
        }
    )
    return 0


def parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="AOXC autonomous sqlite control-plane state manager")
    sub = p.add_subparsers(dest="cmd", required=True)

    sub.add_parser("init", help="Initialize sqlite state schema").set_defaults(func=cmd_init)
    sub.add_parser("status", help="Print sqlite state summary").set_defaults(func=cmd_status)

    ev = sub.add_parser("event", help="Append autonomous event")
    ev.add_argument("--env", required=True)
    ev.add_argument("--action", required=True)
    ev.add_argument("--status", required=True)
    ev.add_argument("--detail", default="")
    ev.set_defaults(func=cmd_event)

    se = sub.add_parser("set-env", help="Set desired/observed env control-plane state")
    se.add_argument("--env", required=True)
    se.add_argument("--desired-state", required=True)
    se.add_argument("--observed-state", required=True)
    se.add_argument("--note", default="")
    se.set_defaults(func=cmd_set_env)

    rel = sub.add_parser("release", help="Record released artifact metadata")
    rel.add_argument("--version", required=True)
    rel.add_argument("--artifact", required=True)
    rel.add_argument("--evidence", default="")
    rel.set_defaults(func=cmd_release)

    hist = sub.add_parser("history", help="Read recent event history")
    hist.add_argument("--limit", type=int, default=30)
    hist.set_defaults(func=cmd_history)

    return p


def main() -> int:
    args = parser().parse_args()
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
