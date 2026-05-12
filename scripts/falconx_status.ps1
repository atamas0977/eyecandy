$p = Get-Process -Name UnrealEditor -ErrorAction SilentlyContinue
if ($p) {
    $rss_mb = [math]::Round($p.WorkingSet64 / 1MB, 0)
    $cpu = [math]::Round($p.TotalProcessorTime.TotalSeconds, 1)
    Write-Output "UnrealEditor PID=$($p.Id)  RSS=${rss_mb}MB  CPU=${cpu}s"
} else {
    Write-Output "UnrealEditor NOT RUNNING"
}
$scw = (Get-Process -Name ShaderCompileWorker -ErrorAction SilentlyContinue | Measure-Object).Count
Write-Output "ShaderCompileWorker processes: $scw"
$log = 'C:\Users\Alexander\eyecandy\zenparticles-ai\BonsaiDiorama\Saved\Logs\BonsaiDiorama.log'
if (Test-Path $log) {
    $li = Get-Item $log
    Write-Output "Log size: $([math]::Round($li.Length/1KB,1)) KB  last-write: $($li.LastWriteTime)"
    Write-Output "---Last 12 log lines---"
    Get-Content $log -Tail 12
}
