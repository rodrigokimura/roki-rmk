import zipfile
import os
import sys
import tempfile
import subprocess
import json

TARGETS = {
    "d": ("RoKi-central.uf2", "dongle"),
    "l": ("RoKi-peripheral.uf2", "left half"),
    "r": ("RoKi-peripheral2.uf2", "right half"),
}

if len(sys.argv) < 2 or sys.argv[1].lower() not in TARGETS:
    print("Usage: python unzip_and_copy_win.py <d|l|r> (d=dongle, l=left, r=right)")
    sys.exit(1)

arg = sys.argv[1].lower()
target_file, target_name = TARGETS[arg]
dest_path = "/mnt/d/"


def copy_to_windows(src: str, filename: str):
    """Copy file from WSL to Windows D: drive using PowerShell."""
    # Convert WSL path to Windows path for PowerShell
    result = subprocess.run(
        ["wslpath", "-w", src],
        capture_output=True, text=True, check=True
    )
    src_win = result.stdout.strip()
    dest_win = f"D:\\{filename}"
    cmd = [
        "powershell.exe",
        "-Command",
        f'Copy-Item -Path "{src_win}" -Destination "{dest_win}" -Force',
    ]
    subprocess.run(cmd, check=True)
    print(f"Copied to {dest_win}")


# Check gh CLI is available
try:
    subprocess.run(["gh", "--version"], check=True, capture_output=True)
except (subprocess.CalledProcessError, FileNotFoundError):
    print("ERROR: GitHub CLI (gh) not found. Install it from https://cli.github.com/")
    sys.exit(1)

# Get repo info from git remote
try:
    result = subprocess.run(
        ["git", "remote", "get-url", "origin"],
        capture_output=True,
        text=True,
        check=True,
    )
    remote_url = result.stdout.strip()
    # Parse github.com/owner/repo.git or https://github.com/owner/repo
    if "github.com" in remote_url:
        parts = remote_url.replace("://", "/").replace(":", "/").split("/")
        # Find github.com index
        for i, part in enumerate(parts):
            if "github.com" in part:
                owner = parts[i + 1]
                repo = parts[i + 2].replace(".git", "")
                break
        else:
            raise ValueError("Could not parse repo")
    else:
        raise ValueError("Not a GitHub remote")
except Exception as e:
    print(f"ERROR: Could not detect GitHub repo from git remote: {e}")
    print("Make sure you're running this from inside the git repo.")
    sys.exit(1)

print(f"Repo: {owner}/{repo}")
print(f"Target: {target_name} ({target_file})")

# Find latest workflow run
print("Finding latest workflow run...")
try:
    result = subprocess.run(
        [
            "gh",
            "run",
            "list",
            "--repo",
            f"{owner}/{repo}",
            "--workflow",
            "build.yml",
            "--limit",
            "1",
            "--json",
            "databaseId,status,conclusion",
        ],
        capture_output=True,
        text=True,
        check=True,
    )
    runs = json.loads(result.stdout)
    if not runs:
        print("ERROR: No workflow runs found.")
        sys.exit(1)
    run = runs[0]
    run_id = run["databaseId"]
    status = run["status"]
    conclusion = run.get("conclusion", "N/A")
    print(f"Latest run: {run_id} (status={status}, conclusion={conclusion})")
    if conclusion != "success":
        print(f"WARNING: Latest run did not succeed (conclusion={conclusion}).")
        reply = input("Download anyway? (y/N): ")
        if reply.lower() != "y":
            sys.exit(1)
except Exception as e:
    print(f"ERROR: Failed to list workflow runs: {e}")
    sys.exit(1)

# Download artifact to temp dir
tmp_dir = tempfile.mkdtemp(prefix="roki_")
artifact_name = "RoKi-firmware_uf2"  # default naming from RMK workflow

print(f"Downloading artifact '{artifact_name}'...")
try:
    subprocess.run(
        [
            "gh",
            "run",
            "download",
            str(run_id),
            "--repo",
            f"{owner}/{repo}",
            "--name",
            artifact_name,
            "--dir",
            tmp_dir,
        ],
        check=True,
        capture_output=False,
    )
except subprocess.CalledProcessError as e:
    print(f"ERROR: Failed to download artifact: {e}")
    # Try to list available artifacts for debugging
    print("Available artifacts in latest run:")
    try:
        result = subprocess.run(
            [
                "gh",
                "run",
                "view",
                str(run_id),
                "--repo",
                f"{owner}/{repo}",
                "--json",
                "artifacts",
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        data = json.loads(result.stdout)
        for art in data.get("artifacts", []):
            print(f"  - {art.get('name')}")
    except Exception:
        pass
    sys.exit(1)

# Find the downloaded zip
zip_candidates = [f for f in os.listdir(tmp_dir) if f.endswith(".zip")]
if zip_candidates:
    zip_path = os.path.join(tmp_dir, zip_candidates[0])
else:
    # Files may have been extracted directly
    extracted_files = [f for f in os.listdir(tmp_dir) if f.endswith(".uf2")]
    if extracted_files:
        zip_path = None  # files already extracted
    else:
        print("ERROR: No zip or uf2 files found in downloaded artifact.")
        sys.exit(1)

# Check destination exists
if not os.path.exists(dest_path):
    print(f"ERROR: Destination drive not found: {dest_path}")
    sys.exit(1)

try:
    if zip_path:
        print(f"Opening {zip_path}...")
        with zipfile.ZipFile(zip_path, "r") as z:
            if target_file in z.namelist():
                print(f"Extracting {target_file}...")
                extracted_path = z.extract(target_file, tmp_dir)
                copy_to_windows(extracted_path, target_file)
            else:
                print(f"{target_file} not found in the archive.")
                print("Files in archive:", z.namelist())
                sys.exit(1)
    else:
        # Direct file
        for f in extracted_files:
            if f == target_file:
                src = os.path.join(tmp_dir, f)
                copy_to_windows(src, f)
                break
        else:
            print(f"{target_file} not found in downloaded files.")
            print("Files:", extracted_files)
            sys.exit(1)

except subprocess.CalledProcessError as e:
    print(f"\nPowerShell copy failed: {e}")
    print("Make sure the D: drive is mounted and writable.")
    sys.exit(1)
except PermissionError as e:
    print(f"\nPermission denied: {e}")
    print("If copying to D:\\, make sure the drive is writable.")
    sys.exit(1)
except OSError as e:
    print(f"\nOS error: {e}")
    sys.exit(1)
finally:
    # Cleanup temp dir
    import shutil

    shutil.rmtree(tmp_dir, ignore_errors=True)
