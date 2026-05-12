#!/usr/bin/env bash
# Helper shell to run commands on FalconX via SSH.
# Usage:
#   ./scripts/falconx.sh "<powershell-command>"           # run via powershell -Command
#   ./scripts/falconx.sh --file local-script.ps1          # upload + run script
#   ./scripts/falconx.sh --raw "<cmd>"                    # run raw (cmd.exe)
#   ./scripts/falconx.sh --sync                           # rsync this dir → FalconX C:\Users\Alexander\eyecandy\
set -euo pipefail

HOST="Alexander@192.168.0.145"
KEY="$HOME/.ssh/falconx_ed25519"
SSH_FLAGS=(-i "$KEY" -o BatchMode=yes -o StrictHostKeyChecking=no -o ConnectTimeout=15)
REMOTE_ROOT='C:\Users\Alexander\eyecandy'
HERE="$(cd "$(dirname "$0")/.." && pwd)"

case "${1:-}" in
  --file)
    shift
    src="$1"
    name="$(basename "$src")"
    scp "${SSH_FLAGS[@]}" "$src" "$HOST:/Users/Alexander/Documents/$name" >/dev/null
    ssh "${SSH_FLAGS[@]}" "$HOST" "powershell -ExecutionPolicy Bypass -File C:\\Users\\Alexander\\Documents\\$name"
    ;;
  --raw)
    shift
    ssh "${SSH_FLAGS[@]}" "$HOST" "$@"
    ;;
  --sync)
    # rsync via SSH; excludes match .gitignore intent
    rsync -avh --delete \
      --exclude='.git/' \
      --exclude='target/' \
      --exclude='Binaries/' \
      --exclude='Intermediate/' \
      --exclude='Saved/' \
      --exclude='DerivedDataCache/' \
      --exclude='audio/*.wav' \
      --exclude='audio/*.flac' \
      --exclude='node_modules/' \
      -e "ssh ${SSH_FLAGS[*]}" \
      "$HERE/" \
      "$HOST:/Users/Alexander/eyecandy-src/"
    echo "Synced $HERE → FalconX:C:\\Users\\Alexander\\eyecandy-src\\"
    ;;
  --launch-editor)
    # Open BonsaiDiorama in the source-built editor
    ssh "${SSH_FLAGS[@]}" "$HOST" "powershell -Command \"Start-Process 'C:\\Users\\Alexander\\eyecandy\\assets\\nvrtx-engine\\Engine\\Binaries\\Win64\\UnrealEditor.exe' -ArgumentList '\\\"C:\\Users\\Alexander\\eyecandy\\zenparticles-ai\\BonsaiDiorama\\BonsaiDiorama.uproject\\\"' -WorkingDirectory 'C:\\Users\\Alexander\\eyecandy\\assets\\nvrtx-engine\\Engine\\Binaries\\Win64' -PassThru | Select-Object Id\""
    ;;
  --launch-binary)
    # Once binary is extracted, launch BonsaiDiorama.exe
    ssh "${SSH_FLAGS[@]}" "$HOST" "powershell -Command \"Get-ChildItem -Path 'C:\\Users\\Alexander\\eyecandy\\assets\\bonsai-binary' -Filter 'BonsaiDiorama.exe' -Recurse | Select-Object -First 1 -ExpandProperty FullName | ForEach-Object { Start-Process \$_ -PassThru } | Select-Object Id\""
    ;;
  --download-status)
    ssh "${SSH_FLAGS[@]}" "$HOST" 'powershell -Command "Get-BitsTransfer | Select-Object DisplayName, JobState, @{N=\"GB\";E={[math]::Round($_.BytesTotal/1GB,2)}}, @{N=\"PctDone\";E={if($_.BytesTotal -gt 0){[math]::Round(($_.BytesTransferred*100)/$_.BytesTotal,1)}else{0}}} | Format-Table -AutoSize | Out-String"'
    ;;
  --download-complete)
    ssh "${SSH_FLAGS[@]}" "$HOST" 'powershell -Command "Get-BitsTransfer -Name BonsaiBinary | Complete-BitsTransfer"'
    ;;
  '')
    echo "Usage: $0 \"<powershell-cmd>\" | --file <ps1> | --raw <cmd> | --sync | --launch-editor | --launch-binary | --download-status | --download-complete"
    exit 1
    ;;
  *)
    ssh "${SSH_FLAGS[@]}" "$HOST" "powershell -Command \"$*\""
    ;;
esac
