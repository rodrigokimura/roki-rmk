import zipfile
import shutil
import os
import sys
import tempfile

if len(sys.argv) < 2 or sys.argv[1].lower() not in ("c", "p"):
    print("Usage: python unzip_and_copy_win.py <c|p> (c=central, p=peripheral)")
    sys.exit(1)

arg = sys.argv[1].lower()
target_file = "RoKi-central.uf2" if arg == "c" else "RoKi-peripheral.uf2"

zip_path = r"C:\Users\kimur\Downloads\RoKi-firmware_uf2.zip"
dest_path = r"D:\\"

# Check source exists
if not os.path.exists(zip_path):
    print(f"ERROR: Zip file not found: {zip_path}")
    sys.exit(1)

# Check destination exists
if not os.path.exists(dest_path):
    print(f"ERROR: Destination drive not found: {dest_path}")
    sys.exit(1)

try:
    with zipfile.ZipFile(zip_path, "r") as z:
        if target_file in z.namelist():
            print(f"Extracting {target_file}...")
            # Use a temp directory instead of /tmp
            tmp_dir = tempfile.gettempdir()
            extracted_path = z.extract(target_file, tmp_dir)
            dest_file = os.path.join(dest_path, target_file)
            shutil.copy2(extracted_path, dest_file)
            print(f"Copied to {dest_file}")
            os.remove(extracted_path)
        else:
            print(f"{target_file} not found in the archive.")
            print("Files in archive:", z.namelist())
except PermissionError as e:
    print(f"\nPermission denied: {e}")
    print("If copying to D:\\, make sure the drive is writable.")
    sys.exit(1)
except OSError as e:
    print(f"\nOS error: {e}")
    sys.exit(1)
