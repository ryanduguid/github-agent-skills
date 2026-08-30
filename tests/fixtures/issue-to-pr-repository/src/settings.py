"""Read the small JSON settings document used by this fixture."""

import json


def load_settings(source: str) -> dict[str, object]:
    return json.loads(source)
