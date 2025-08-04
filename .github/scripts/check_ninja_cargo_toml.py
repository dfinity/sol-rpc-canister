#!/usr/bin/env python3
"""
Validate that 'examples/basic_solana/ninja/Cargo.toml' is standalone (i.e. no workspace and path dependencies) and has
correctly resolved dependency versions matching those of 'examples/basic_solana/Cargo.toml'.
"""

import sys
import tomllib
from pathlib import Path


def load_toml(path):
    with open(path, 'rb') as f:
        return tomllib.load(f)


def resolve_spec(basic_solana_spec, workspace_deps, name):
    """Resolve a `basic_solana` dependency spec to what it should be in the ICP Ninja `Cargo.toml`."""
    if isinstance(basic_solana_spec, dict) and 'workspace' in basic_solana_spec:
        # Get workspace dependency version and merge with `basic_solana` spec (`features`, `optional`, etc.)
        workspace_spec = workspace_deps[name]
        resolved = {'version': workspace_spec} if isinstance(workspace_spec, str) else dict(workspace_spec)
        resolved.update({k: v for k, v in basic_solana_spec.items() if k != 'workspace'})
        return resolved['version'] if list(resolved.keys()) == ['version'] else resolved

    elif isinstance(basic_solana_spec, dict) and 'path' in basic_solana_spec:
        # Get path dependency version and merge with `basic_solana` spec (`features`, `optional`, etc.)
        path_toml = load_toml(Path("examples/basic_solana") / basic_solana_spec['path'] / "Cargo.toml")
        version = path_toml['package']['version']
        resolved = {'version': version}
        resolved.update({k: v for k, v in basic_solana_spec.items() if k != 'path'})
        return resolved['version'] if list(resolved.keys()) == ['version'] else resolved

    return basic_solana_spec


def main():
    basic_solana = load_toml(Path("examples/basic_solana/Cargo.toml"))
    ninja = load_toml(Path("examples/basic_solana/ninja/Cargo.toml"))
    workspace = load_toml(Path("Cargo.toml"))

    basic_solana_deps = basic_solana.get('dependencies', {})
    ninja_deps = ninja.get('dependencies', {})
    workspace_deps = workspace.get('workspace', {}).get('dependencies', {})

    error_messages = set()

    # Check dependency lists match
    if set(basic_solana_deps.keys()) != set(ninja_deps.keys()):
        if missing := set(basic_solana_deps.keys()) - set(ninja_deps.keys()):
            error_messages.add(f"Missing dependencies: {sorted(missing)}")
        if extra := set(ninja_deps.keys()) - set(basic_solana_deps.keys()):
            error_messages.add(f"Unknown dependencies: {sorted(extra)}")

    # Check no workspace/path deps in ninja
    for name, spec in ninja_deps.items():
        if isinstance(spec, dict) and ('workspace' in spec or 'path' in spec):
            error_messages.add(f"'{name}' version is not resolved")

    # Check all dependencies are correctly resolved
    for name, basic_solana_spec in basic_solana_deps.items():
        expected = resolve_spec(basic_solana_spec, workspace_deps, name)
        actual = ninja_deps[name]
        if expected != actual:
            error_messages.add(f"'{name}' version mismatch - expected: '{expected}', actual: '{actual}'")

    # Check patches match
    root_patches = set(workspace.get('patch', {}).get('crates-io', {}).keys())
    ninja_patches = set(ninja.get('patch', {}).get('crates-io', {}).keys())
    if not root_patches.issubset(ninja_patches):
        error_messages.add(f"Missing patches: {root_patches - ninja_patches}")

    if error_messages:
        print("ERROR: Some errors were found in 'examples/basic_solana/ninja/Cargo.toml':")
        for error_message in error_messages:
            print(f"\t* {error_message}")
        sys.exit(1)

    print("OK: 'examples/basic_solana/ninja/Cargo.toml' is valid")


if __name__ == "__main__":
    main()
