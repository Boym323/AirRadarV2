#!/usr/bin/env python3
"""G-03: samostatná formální runtime OpenAPI validace bez komunikace s LXC."""
import json
from pathlib import Path
import yaml
from jsonschema import Draft202012Validator, FormatChecker
from openapi_spec_validator import validate_spec

ROOT = Path(__file__).resolve().parent.parent
class UniqueKeyLoader(yaml.SafeLoader):
    pass

def unique_mapping(loader, node):
    result = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node, deep=True)
        if key in result:
            raise ValueError(f"Duplikátní YAML klíč: {key}")
        result[key] = loader.construct_object(value_node, deep=True)
    return result

UniqueKeyLoader.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, unique_mapping)
api = yaml.load((ROOT / "contracts/openapi.g03.yaml").read_text(encoding="utf-8"), Loader=UniqueKeyLoader)
validate_spec(api)
schemas = api["components"]["schemas"]
for name in ("LiveHealthV1", "LiveAircraftResponseV1", "SnapshotV1", "DeltaV1", "ErrorV1"):
    assert name in schemas

def check(name, data):
    schema = {
        "$ref": f"#/components/schemas/{name}",
        "components": {"schemas": schemas},
    }
    Draft202012Validator(schema, format_checker=FormatChecker()).validate(data)

sequence = json.loads((ROOT / "contracts/fixtures/sse/sequence.json").read_text(encoding="utf-8"))
check("LiveAircraftResponseV1", {"status": "live", "data": sequence["snapshot"]})
check("LiveAircraftResponseV1", {"status": "waiting_for_observations", "data": sequence["snapshot"]})
check("LiveHealthV1", {
    "status": "waiting_for_observations",
    "stage": "G-03",
    "live_ingest": False,
    "database_connected": False,
})
check("ErrorV1", {"code": "UNAVAILABLE", "message": "Limit SSE spojení byl vyčerpán."})
check("ErrorV1", {"code": "CONTRACT_INVALID", "message": "Nepodporované parametry."})
check("DeltaV1", sequence["delta"])
print("PASS G-03: oddělený OpenAPI 3.1, public snapshot/delta, health, typované chyby.")
