"""Exercise the built MCP server without invoking desktop or keyring mutations."""

import json
import subprocess
import sys
from pathlib import Path

binary = str(Path(sys.argv[1]).resolve())
requests = [
    {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
    {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
    {"jsonrpc": "2.0", "id": 3, "method": "resources/list"},
]
result = subprocess.run(  # noqa: S603 -- Run the exact built executable supplied by the CI job; no shell.
    [binary],
    input="\n".join(json.dumps(item) for item in requests) + "\n",
    capture_output=True,
    text=True,
    timeout=30,
    check=True,
)
root = Path(".ci-results")
root.mkdir(exist_ok=True)
(root / "protocol.stdout").write_text(result.stdout)
(root / "protocol.stderr").write_text(result.stderr)
responses = {}
for line in result.stdout.splitlines():
    if line.startswith("{"):
        value = json.loads(line)
        if "id" in value:
            responses[value["id"]] = value
if set(responses) != {1, 2, 3}:
    raise RuntimeError("Built server did not reply to all fixture requests")
if responses[1]["result"]["serverInfo"]["name"] != "gnome-mcp-server":
    raise RuntimeError("Unexpected server identity")
if not responses[2]["result"]["tools"] or not responses[3]["result"]["resources"]:
    raise RuntimeError("Built server did not advertise tools and resources")
print("MCP_PROTOCOL_SMOKE_PASS: initialize and read-only provider discovery")
