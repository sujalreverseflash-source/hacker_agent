# Automated Nmap Flag Combinations Testing Script (PowerShell)
# Tests all 108 combinations (17 combos × 6 timings + blank)

$TARGET = "scanme.nmap.org"
$BASE_URL = "http://localhost:8081/scan-open-ports"
$LOG_FILE = "nmap_test_results_$(Get-Date -Format 'yyyyMMdd_HHmmss').log"
$RESULTS_DIR = "test_results"

# Create results directory
New-Item -ItemType Directory -Force -Path $RESULTS_DIR | Out-Null

Write-Host "🚀 Starting Nmap Flag Combinations Test" -ForegroundColor Green
Write-Host "Target: $TARGET" -ForegroundColor Yellow
Write-Host "Results will be saved to: $RESULTS_DIR/" -ForegroundColor Yellow
Write-Host "Log file: $LOG_FILE" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan

# Initialize log
"🚀 Starting Nmap Flag Combinations Test" | Out-File $LOG_FILE
"Target: $TARGET" | Add-Content $LOG_FILE
"Results will be saved to: $RESULTS_DIR/" | Add-Content $LOG_FILE
"Log file: $LOG_FILE" | Add-Content $LOG_FILE
"========================================" | Add-Content $LOG_FILE

# Function to run a single test
function Run-Test {
    param(
        [string]$TestName,
        [string]$JsonPayload
    )
    
    $outputFile = "$RESULTS_DIR\$TestName.json"
    
    Write-Host "Testing: $TestName" -ForegroundColor Cyan
    Write-Host "Payload: $JsonPayload" -ForegroundColor Gray
    
    # Run the curl command and capture output
    $response = curl -s -X POST $BASE_URL `
        -H "Content-Type: application/json" `
        -d $JsonPayload `
        -w "`nHTTP_CODE:%{http_code}`nTIME:%{time_total}"
    
    # Extract HTTP code and time
    $lines = $response -split "`n"
    $httpCode = ($lines | Where-Object { $_ -match "HTTP_CODE:" }) -replace "HTTP_CODE:", ""
    $timeTaken = ($lines | Where-Object { $_ -match "TIME:" }) -replace "TIME:", ""
    $jsonResponse = ($lines | Where-Object { $_ -notmatch "HTTP_CODE:" -and $_ -notmatch "TIME:" }) -join "`n"
    
    # Save results
    $jsonResponse | Out-File $outputFile
    
    # Log results
    Write-Host "HTTP Code: $httpCode" -ForegroundColor Yellow
    Write-Host "Time: ${timeTaken}s" -ForegroundColor Yellow
    Write-Host "Output saved to: $outputFile" -ForegroundColor Green
    Write-Host "---" -ForegroundColor Gray
    
    # Log to file
    "Testing: $TestName" | Add-Content $LOG_FILE
    "Payload: $JsonPayload" | Add-Content $LOG_FILE
    "HTTP Code: $httpCode" | Add-Content $LOG_FILE
    "Time: ${timeTaken}s" | Add-Content $LOG_FILE
    "Output saved to: $outputFile" | Add-Content $LOG_FILE
    "---" | Add-Content $LOG_FILE
    
    # Small delay between tests
    Start-Sleep -Seconds 2
}

# Function to test a combination with all timing templates
function Test-WithTimings {
    param(
        [string]$BaseName,
        [string]$BasePayload
    )
    
    # Test with no timing (blank)
    Run-Test -TestName "${BaseName}_no_timing" -JsonPayload $BasePayload
    
    # Test with all timing templates
    foreach ($timing in @("T0", "T1", "T2", "T3", "T4", "T5")) {
        $timedPayload = $BasePayload -replace '}$', ", `"timing`": `"$timing`"}"
        Run-Test -TestName "${BaseName}_$timing" -JsonPayload $timedPayload
    }
}

Write-Host "📋 Starting Single Flag Tests..." -ForegroundColor Magenta

# Single Flag Tests
Test-WithTimings -BaseName "single_O" -BasePayload "{`"target`": `"$TARGET`", `"flag_o`": true}"
Test-WithTimings -BaseName "single_sC" -BasePayload "{`"target`": `"$TARGET`", `"flag_sc`": true}"
Test-WithTimings -BaseName "single_sV" -BasePayload "{`"target`": `"$TARGET`", `"flag_sv`": true}"
Test-WithTimings -BaseName "single_traceroute" -BasePayload "{`"target`": `"$TARGET`", `"flag_traceroute`": true}"
Test-WithTimings -BaseName "single_A" -BasePayload "{`"target`": `"$TARGET`", `"flag_a`": true}"

Write-Host "📋 Starting Two Flag Combination Tests..." -ForegroundColor Magenta

# Two Flag Combinations
Test-WithTimings -BaseName "combo_O_sC" -BasePayload "{`"target`": `"$TARGET`", `"flag_o`": true, `"flag_sc`": true}"
Test-WithTimings -BaseName "combo_O_sV" -BasePayload "{`"target`": `"$TARGET`", `"flag_o`": true, `"flag_sv`": true}"
Test-WithTimings -BaseName "combo_O_traceroute" -BasePayload "{`"target`": `"$TARGET`", `"flag_o`": true, `"flag_traceroute`": true}"
Test-WithTimings -BaseName "combo_sC_sV" -BasePayload "{`"target`": `"$TARGET`", `"flag_sc`": true, `"flag_sv`": true}"
Test-WithTimings -BaseName "combo_sC_traceroute" -BasePayload "{`"target`": `"$TARGET`", `"flag_sc`": true, `"flag_traceroute`": true}"
Test-WithTimings -BaseName "combo_sV_traceroute" -BasePayload "{`"target`": `"$TARGET`", `"flag_sv`": true, `"flag_traceroute`": true}"
Test-WithTimings -BaseName "combo_A_traceroute" -BasePayload "{`"target`": `"$TARGET`", `"flag_a`": true, `"flag_traceroute`": true}"

Write-Host "📋 Starting Three Flag Combination Tests..." -ForegroundColor Magenta

# Three Flag Combinations
Test-WithTimings -BaseName "combo_O_sC_sV" -BasePayload "{`"target`": `"$TARGET`", `"flag_o`": true, `"flag_sc`": true, `"flag_sv`": true}"
Test-WithTimings -BaseName "combo_O_sC_traceroute" -BasePayload "{`"target`": `"$TARGET`", `"flag_o`": true, `"flag_sc`": true, `"flag_traceroute`": true}"
Test-WithTimings -BaseName "combo_O_sV_traceroute" -BasePayload "{`"target`": `"$TARGET`", `"flag_o`": true, `"flag_sv`": true, `"flag_traceroute`": true}"
Test-WithTimings -BaseName "combo_sC_sV_traceroute" -BasePayload "{`"target`": `"$TARGET`", `"flag_sc`": true, `"flag_sv`": true, `"flag_traceroute`": true}"

Write-Host "📋 Starting Four Flag Combination Test..." -ForegroundColor Magenta

# Four Flag Combination
Test-WithTimings -BaseName "combo_O_sC_sV_traceroute" -BasePayload "{`"target`": `"$TARGET`", `"flag_o`": true, `"flag_sc`": true, `"flag_sv`": true, `"flag_traceroute`": true}"

Write-Host "📋 Starting Blank Scan Test..." -ForegroundColor Magenta

# Blank Scan (no flags)
Test-WithTimings -BaseName "blank_scan" -BasePayload "{`"target`": `"$TARGET`"}"

Write-Host "🎉 All Tests Completed!" -ForegroundColor Green
Write-Host "Total tests run: 108" -ForegroundColor Yellow
Write-Host "Results saved in: $RESULTS_DIR/" -ForegroundColor Yellow
Write-Host "Log file: $LOG_FILE" -ForegroundColor Yellow

# Generate summary report
Write-Host "📊 Generating Summary Report..." -ForegroundColor Cyan
$summaryFile = "$RESULTS_DIR\test_summary.txt"
"Nmap Flag Combinations Test Summary" | Out-File $summaryFile
"=================================" | Add-Content $summaryFile
"Target: $TARGET" | Add-Content $summaryFile
"Test Date: $(Get-Date)" | Add-Content $summaryFile
"" | Add-Content $summaryFile
"Test Files Created:" | Add-Content $summaryFile
Get-ChildItem "$RESULTS_DIR\*.json" | ForEach-Object { $_.Name } | Add-Content $summaryFile
"" | Add-Content $summaryFile
"Total JSON files: $((Get-ChildItem "$RESULTS_DIR\*.json").Count)" | Add-Content $summaryFile

Write-Host "Summary report saved to: $summaryFile" -ForegroundColor Green
Write-Host "🚀 Testing Complete! Check $RESULTS_DIR\ for all results." -ForegroundColor Green

# Final log entry
"🎉 All Tests Completed!" | Add-Content $LOG_FILE
"Total tests run: 108" | Add-Content $LOG_FILE
"Results saved in: $RESULTS_DIR/" | Add-Content $LOG_FILE
"Summary report saved to: $summaryFile" | Add-Content $LOG_FILE
