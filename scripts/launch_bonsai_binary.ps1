$ErrorActionPreference = 'Continue'
$exe = 'C:\Users\Alexander\eyecandy\assets\bonsai-binary\Windows\BonsaiDiorama.exe'
$taskName = 'EyeCandyBonsaiOneShot'

# Clean prior task if any
Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue

# Build the action — launch the EXE
$action = New-ScheduledTaskAction -Execute $exe -WorkingDirectory (Split-Path $exe -Parent)

# Trigger: run now
$trigger = New-ScheduledTaskTrigger -At (Get-Date).AddSeconds(5) -Once

# Principal: run in the logged-in user's interactive session
$principal = New-ScheduledTaskPrincipal -UserId 'Alexander' -LogonType Interactive -RunLevel Limited

# Settings: short start window, allow demand
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable -ExecutionTimeLimit (New-TimeSpan -Hours 4)

$task = New-ScheduledTask -Action $action -Trigger $trigger -Principal $principal -Settings $settings

Register-ScheduledTask -TaskName $taskName -InputObject $task -Force | Out-Null
Write-Output "Task registered: $taskName"

# Start now
Start-ScheduledTask -TaskName $taskName
Write-Output "Task started"

# Wait 8s, check process
Start-Sleep -Seconds 8
$procs = Get-Process | Where-Object { $_.ProcessName -match 'Bonsai|Unreal' } | Select-Object ProcessName, Id, @{N='MB';E={[math]::Round($_.WorkingSet64/1MB,0)}}
if ($procs) {
    Write-Output "===PROCESSES==="
    $procs | Format-Table -AutoSize | Out-String
} else {
    Write-Output "No Bonsai/Unreal process running after 8s"
}

# Clean up the task entry after firing (the process keeps running)
Start-Sleep -Seconds 2
Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue
Write-Output "Task entry cleaned up (process keeps running independently)"
