import sys
import os
import re
from enum import Enum


class BumpType(Enum):
    MAJOR = "major"
    MINOR = "minor"
    PATCH = "patch"


def is_valid_semver(version: str) -> bool:
    semver_regex = r"^\d+\.\d+\.\d+$"
    return bool(re.match(semver_regex, version))


def raise_semver_exception():
    raise ValueError("Invalid version - must follow SEMVER convention (major.minor.patch)!")


def validate_semver_bump(previous_version: str, next_version: str) -> BumpType:
    if not (is_valid_semver(previous_version) and is_valid_semver(next_version)):
        raise_semver_exception()

    if previous_version == next_version:
        raise ValueError("Version was not bumped!")
    
    print(f"Comparing next version {next_version} with previous version {previous_version}.")
    
    previous_version = list(map(int, previous_version.split(".")))
    next_version = list(map(int, next_version.split(".")))

    semver_parts = [BumpType.MAJOR, BumpType.MINOR, BumpType.PATCH]
    for idx in range(len(semver_parts)):
        if next_version[idx] != previous_version[idx]:
            correct_bump = next_version[idx] - 1 == previous_version[idx]
            lower_levels_reset = all(next_version[jdx] == 0 for jdx in range(idx + 1, len(semver_parts)))

            if correct_bump and lower_levels_reset:
                print(f"{semver_parts[idx].value} version bump.")
                return semver_parts[idx]
            
            raise_semver_exception()
        

if __name__ == "__main__":
    previous_version = sys.argv[1].removeprefix("v")
    next_version = sys.argv[2].removeprefix("v")

    bump_type = validate_semver_bump(previous_version, next_version)
    with open(os.getenv("GITHUB_OUTPUT"), "a") as output:
        output.write(f"bump_type={bump_type.value}\n")