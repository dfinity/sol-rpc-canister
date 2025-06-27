#!/usr/bin/env python3
"""
Validate that ninja/Cargo.toml is properly standalone.
"""

import sys
import tomllib
from pathlib import Path


def load_toml(path):
    with open(path, 'rb') as f:
        return tomllib.load(f)


def resolve_spec(source_spec, workspace_deps, name):
    """Resolve a source dependency spec to what it should be in ninja."""
    if isinstance(source_spec, dict) and source_spec.get('workspace'):
        # Merge workspace version with source flags
        workspace_spec = workspace_deps[name]
        resolved = {'version': workspace_spec} if isinstance(workspace_spec, str) else dict(workspace_spec)
        resolved.update({k: v for k, v in source_spec.items() if k != 'workspace'})
        return resolved['version'] if list(resolved.keys()) == ['version'] else resolved
    
    elif isinstance(source_spec, dict) and 'path' in source_spec:
        # Get version from path and merge with source flags
        path_toml = load_toml(Path("examples/basic_solana") / source_spec['path'] / "Cargo.toml")
        version = path_toml['package']['version']
        resolved = {'version': version}
        resolved.update({k: v for k, v in source_spec.items() if k != 'path'})
        return resolved['version'] if list(resolved.keys()) == ['version'] else resolved
    
    return source_spec


def main():
    source = load_toml(Path("examples/basic_solana/Cargo.toml"))
    ninja = load_toml(Path("examples/basic_solana/ninja/Cargo.toml"))
    root = load_toml(Path("Cargo.toml"))
    
    source_deps = source.get('dependencies', {})
    ninja_deps = ninja.get('dependencies', {})
    workspace_deps = root.get('workspace', {}).get('dependencies', {})
    
    # Check dependency names match
    if set(source_deps.keys()) != set(ninja_deps.keys()):
        print(f"ERROR: Dependencies don't match")
        sys.exit(1)
    
    # Check no workspace/path deps in ninja
    for name, spec in ninja_deps.items():
        if isinstance(spec, dict) and (spec.get('workspace') or 'path' in spec):
            print(f"ERROR: {name} not resolved in ninja")
            sys.exit(1)
    
    # Check all dependencies are correctly resolved
    for name, source_spec in source_deps.items():
        expected = resolve_spec(source_spec, workspace_deps, name)
        actual = ninja_deps[name]
        if expected != actual:
            print(f"ERROR: {name} mismatch - expected: {expected}, actual: {actual}")
            sys.exit(1)
    
    # Check patches match
    root_patches = set(root.get('patch', {}).get('crates-io', {}).keys())
    ninja_patches = set(ninja.get('patch', {}).get('crates-io', {}).keys())
    if not root_patches.issubset(ninja_patches):
        print(f"ERROR: Missing patches: {root_patches - ninja_patches}")
        sys.exit(1)
    
    print("OK: ninja/Cargo.toml is properly standalone")


if __name__ == "__main__":
    main() 