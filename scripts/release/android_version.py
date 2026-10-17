#!/usr/bin/env python3
"""Print the Android versionName and versionCode for a release tag.

Google Play needs a versionCode that grows with every upload, so it is derived from the tag:
MAJOR * 1_000_000 + MINOR * 10_000 + PATCH * 100 + stage, where alpha.N is N, beta.N is 30 + N,
rc.N is 60 + N and a final release is 99. N stays below 30.
"""
import re
import sys

STAGE_BASE = {"alpha": 0, "beta": 30, "rc": 60}


def android_version(tag: str) -> tuple[str, int]:
    match = re.fullmatch(r"v(\d+)\.(\d+)\.(\d+)(?:-(alpha|beta|rc)\.(\d+))?", tag)
    if not match:
        raise SystemExit(f"Unsupported release tag: {tag}")
    major, minor, patch, stage, number = match.groups()
    if stage is None:
        stage_code = 99
    else:
        if int(number) >= 30:
            raise SystemExit(f"{tag}: pre-release number must stay below 30")
        stage_code = STAGE_BASE[stage] + int(number)
    if int(minor) >= 100 or int(patch) >= 100:
        raise SystemExit(f"{tag}: minor and patch must stay below 100")
    return tag[1:], int(major) * 1_000_000 + int(minor) * 10_000 + int(patch) * 100 + stage_code


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        order = ["v2.0.0-alpha.18", "v2.0.0-beta.1", "v2.0.0-rc.2", "v2.0.0", "v2.0.1-alpha.1", "v2.1.0"]
        codes = [android_version(tag)[1] for tag in order]
        assert codes == sorted(codes) and len(set(codes)) == len(codes), codes
        assert android_version("v2.0.0-alpha.18") == ("2.0.0-alpha.18", 2_000_018)
        print("ok")
    else:
        name, code = android_version(sys.argv[1])
        print(f"name={name}\ncode={code}")
