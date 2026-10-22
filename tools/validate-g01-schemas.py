#!/usr/bin/env python3
"""G-01 — formální OpenAPI 3.1 / JSON Schema validace na syntetických fixtures."""
import copy
import json
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator, FormatChecker
from openapi_spec_validator import validate_spec

ROOT = Path(__file__).resolve().parent.parent
CONTRACTS = ROOT / "contracts"


class NoDuplicateKeysLoader(yaml.SafeLoader):
    """Duplikátní YAML klíč musí skončit chybou, nikoli tichým přepsáním."""


def construct_mapping(loader, node):
    mapping = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node, deep=True)
        if key in mapping:
            raise ValueError(f"Duplicitní YAML klíč: {key!r}")
        mapping[key] = loader.construct_object(value_node, deep=True)
    return mapping


NoDuplicateKeysLoader.add_constructor(
    yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, construct_mapping
)


def read_json(path):
    return json.loads((CONTRACTS / path).read_text(encoding="utf-8"))


def assert_invalid(validator, data, context):
    if validator.is_valid(data):
        raise AssertionError(f"Neplatná fixture prošla validací: {context}")


api = yaml.load(
    (CONTRACTS / "openapi.yaml").read_text(encoding="utf-8"),
    Loader=NoDuplicateKeysLoader,
)
validate_spec(api)
components = api["components"]["schemas"]
for name in ("PositionV1", "AircraftPublicV1", "SnapshotV1", "DeltaV1", "ErrorV1"):
    if name not in components:
        raise AssertionError(f"Chybí OpenAPI komponenta {name}")
assert api["paths"]["/api/v1/aircraft"]["get"]["responses"]["200"]["content"]["application/json"]["schema"]["properties"]["status"]["const"] == "NOT_IMPLEMENTED"

schemas = {
    name: read_json(name + ".schema.json")
    for name in ("readsb-json", "aircraft-observation", "aircraft-public", "sse-v1")
}
for name, schema in schemas.items():
    Draft202012Validator.check_schema(schema)

# Lokální relativní $ref ve sse-v1.schema.json ukazují na aircraft-public.
# Převod na in-memory schéma nepotřebuje síť ani jiné URI schéma.
sse_schema = copy.deepcopy(schemas["sse-v1"])
for choice in sse_schema["oneOf"]:
    properties = choice["properties"]
    array_field = "aircraft" if choice["properties"]["kind"]["const"] == "snapshot" else "upsert"
    assert properties[array_field]["items"]["$ref"] == "aircraft-public.schema.json"
    properties[array_field]["items"] = schemas["aircraft-public"]

def validator(name, schema=None):
    return Draft202012Validator(schema or schemas[name], format_checker=FormatChecker())

readsb = validator("readsb-json")
for name in ("initial", "repeated", "changed", "empty"):
    readsb.validate(read_json(f"fixtures/readsb/{name}.json"))
assert_invalid(readsb, read_json("fixtures/readsb/invalid.json"), "neplatné readsb vstupy")
invalid_hex = read_json("fixtures/readsb/initial.json")
invalid_hex["aircraft"][0]["hex"] = "~neplatny-hex"
assert_invalid(readsb, invalid_hex, "identita readsb")

observation = validator("aircraft-observation")
expected = read_json("fixtures/readsb/expected.json")
for idx, value in enumerate(expected["initial"]):
    observation.validate(value)
    broken = {**value, "receiverId": "ZPRAVA SOUKROMA"}
    assert_invalid(observation, broken, f"neplatný přijímač {idx}")
    broken = {**value, "squawk": "8899"}
    assert_invalid(observation, broken, f"neplatný squawk {idx}")

public = validator("aircraft-public")
stream = validator("sse-v1", sse_schema)
sequence = read_json("fixtures/sse/sequence.json")
for name in ("snapshot", "delta", "removed", "gap", "restart"):
    stream.validate(sequence[name])
public_aircraft = sequence["snapshot"]["aircraft"][0]
public.validate(public_aircraft)
assert_invalid(public, {**public_aircraft, "receiverId": "rx-primary"}, "veřejný receiverId")
assert_invalid(public, {**public_aircraft, "position": {"lat": 91, "lon": 15}}, "GPS rozsah")
bad_delta = copy.deepcopy(sequence["delta"])
bad_delta["removed"] = ["icao:abc123", "icao:abc123"]
assert_invalid(stream, bad_delta, "duplicita removed")
bad_delta = copy.deepcopy(sequence["delta"])
bad_delta["upsert"][0]["squawk"] = "8899"
assert_invalid(stream, bad_delta, "neplatný squawk v delte")

print("PASS G-01: OpenAPI 3.1 bez duplicit, 4 JSON Schema a pozitivní/negativní golden fixtures.")
