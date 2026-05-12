$ErrorActionPreference = 'Continue'
$exe = 'C:\Users\Alexander\eyecandy\assets\nvrtx-engine\Engine\Binaries\Win64\UnrealEditor.exe'
$proj = 'C:\Users\Alexander\eyecandy\zenparticles-ai\BonsaiDiorama\BonsaiDiorama.uproject'
$wd = 'C:\Users\Alexander\eyecandy\assets\nvrtx-engine\Engine\Binaries\Win64'
$taskName = 'EyeCandyEditorOneShot'

# Kill any prior Bonsai processes if they're still in the way (we don't want the binary running at the same time as the editor on the same project)
# (Skip: binary is on a different project copy actually -- bonsai-binary vs zenparticles-ai/BonsaiDiorama)

Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue

$action = New-ScheduledTaskAction -Execute $exe -Argument "`"$proj`"" -WorkingDirectory $wd
$trigger = New-ScheduledTaskTrigger -At (Get-Date).AddSeconds(5) -Once
$principal = New-ScheduledTaskPrincipal -UserId 'Alexander' -LogonType Interactive -RunLevel Limited
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable -ExecutionTimeLimit (New-TimeSpan -Hours 6)

$task = New-ScheduledTask -Action $action -Trigger $trigger -Principal $principal -Settings $settings
Register-ScheduledTask -TaskName $taskName -InputObject $task -Force | Out-Null

Write-Output "Editor task registered. Starting now..."
Start-ScheduledTask -TaskName $taskName

Start-Sleep -Seconds 10
$procs = Get-Process | Where-Object { $_.ProcessName -match 'Unreal|Shader|CrashReport' } | Select-Object ProcessName, Id, @{N='MB';E={[math]::Round($_.WorkingSet64/1MB,0)}}
if ($procs) {
    Write-Output "===PROCESSES (10s in)==="
    $procs | Format-Table -AutoSize | Out-String
} else {
    Write-Output "No editor process running at 10s"
}

Start-Sleep -Seconds 20
$procs = Get-Process | Where-Object { $_.ProcessName -match 'Unreal|Shader|CrashReport' } | Select-Object ProcessName, Id, @{N='MB';E={[math]::Round($_.WorkingSet64/1MB,0)}}
if ($procs) {
    Write-Output "===PROCESSES (30s in)==="
    $procs | Format-Table -AutoSize | Out-String
} else {
    Write-Output "No editor process running at 30s -- check logs"
}

Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue
