#!/bin/bash

# Ultra-Fast Nmap Flag Combinations Testing Script
# Tests 54 combinations (17 combos × 3 fast timings + blank)

TARGET="scanme.nmap.org"
BASE_URL="http://localhost:8081/scan-open-ports"
LOG_FILE="nmap_ultra_fast_results_$(date +%Y%m%d_%H%M%S).log"
RESULTS_DIR="test_results"

# Create results directory
mkdir -p "$RESULTS_DIR"

echo "🚀 Starting Ultra-Fast Nmap Flag Combinations Test" | tee "$LOG_FILE"
echo "Target: $TARGET" | tee -a "$LOG_FILE"
echo "Only T3, T4, T5 timings (fast speeds)" | tee -a "$LOG_FILE"
echo "Results will be saved to: $RESULTS_DIR/" | tee -a "$LOG_FILE"
echo "Log file: $LOG_FILE" | tee -a "$LOG_FILE"
echo "========================================" | tee -a "$LOG_FILE"

# Function to run a single test
run_test() {
    local test_name="$1"
    local json_payload="$2"
    local output_file="$RESULTS_DIR/${test_name}.json"
    
    echo "Testing: $test_name" | tee -a "$LOG_FILE"
    echo "Payload: $json_payload" | tee -a "$LOG_FILE"
    
    # Run the curl command and capture output
    response=$(curl -s -X POST "$BASE_URL" \
        -H "Content-Type: application/json" \
        -d "$json_payload" \
        -w "\nHTTP_CODE:%{http_code}\nTIME:%{time_total}")
    
    # Extract HTTP code and time
    http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    time_taken=$(echo "$response" | grep "TIME:" | cut -d: -f2)
    json_response=$(echo "$response" | sed '/HTTP_CODE:/d' | sed '/TIME:/d')
    
    # Save results
    echo "$json_response" > "$output_file"
    
    # Log results
    echo "HTTP Code: $http_code" | tee -a "$LOG_FILE"
    echo "Time: ${time_taken}s" | tee -a "$LOG_FILE"
    echo "Output saved to: $output_file" | tee -a "$LOG_FILE"
    echo "---" | tee -a "$LOG_FILE"
    
    # Small delay between tests
    sleep 1
}

# Function to test a combination with ultra-fast timing templates only
test_with_ultra_fast_timings() {
    local base_name="$1"
    local base_payload="$2"
    
    # Test with no timing (blank)
    run_test "${base_name}_no_timing" "$base_payload"
    
    # Test with ultra-fast timing templates only
    for timing in T3 T4 T5; do
        timed_payload="${base_payload%,*}, \"timing\": \"$timing\"}"
        run_test "${base_name}_${timing}" "$timed_payload"
    done
}

echo "📋 Starting Single Flag Tests..." | tee -a "$LOG_FILE"

# Single Flag Tests
test_with_ultra_fast_timings "single_O" "{\"target\": \"$TARGET\", \"flag_o\": true}"
test_with_ultra_fast_timings "single_sC" "{\"target\": \"$TARGET\", \"flag_sc\": true}"
test_with_ultra_fast_timings "single_sV" "{\"target\": \"$TARGET\", \"flag_sv\": true}"
test_with_ultra_fast_timings "single_traceroute" "{\"target\": \"$TARGET\", \"flag_traceroute\": true}"
test_with_ultra_fast_timings "single_A" "{\"target\": \"$TARGET\", \"flag_a\": true}"

echo "📋 Starting Two Flag Combination Tests..." | tee -a "$LOG_FILE"

# Two Flag Combinations
test_with_ultra_fast_timings "combo_O_sC" "{\"target\": \"$TARGET\", \"flag_o\": true, \"flag_sc\": true}"
test_with_ultra_fast_timings "combo_O_sV" "{\"target\": \"$TARGET\", \"flag_o\": true, \"flag_sv\": true}"
test_with_ultra_fast_timings "combo_O_traceroute" "{\"target\": \"$TARGET\", \"flag_o\": true, \"flag_traceroute\": true}"
test_with_ultra_fast_timings "combo_sC_sV" "{\"target\": \"$TARGET\", \"flag_sc\": true, \"flag_sv\": true}"
test_with_ultra_fast_timings "combo_sC_traceroute" "{\"target\": \"$TARGET\", \"flag_sc\": true, \"flag_traceroute\": true}"
test_with_ultra_fast_timings "combo_sV_traceroute" "{\"target\": \"$TARGET\", \"flag_sv\": true, \"flag_traceroute\": true}"
test_with_ultra_fast_timings "combo_A_traceroute" "{\"target\": \"$TARGET\", \"flag_a\": true, \"flag_traceroute\": true}"

echo "📋 Starting Three Flag Combination Tests..." | tee -a "$LOG_FILE"

# Three Flag Combinations
test_with_ultra_fast_timings "combo_O_sC_sV" "{\"target\": \"$TARGET\", \"flag_o\": true, \"flag_sc\": true, \"flag_sv\": true}"
test_with_ultra_fast_timings "combo_O_sC_traceroute" "{\"target\": \"$TARGET\", \"flag_o\": true, \"flag_sc\": true, \"flag_traceroute\": true}"
test_with_ultra_fast_timings "combo_O_sV_traceroute" "{\"target\": \"$TARGET\", \"flag_o\": true, \"flag_sv\": true, \"flag_traceroute\": true}"
test_with_ultra_fast_timings "combo_sC_sV_traceroute" "{\"target\": \"$TARGET\", \"flag_sc\": true, \"flag_sv\": true, \"flag_traceroute\": true}"

echo "📋 Starting Four Flag Combination Test..." | tee -a "$LOG_FILE"

# Four Flag Combination
test_with_ultra_fast_timings "combo_O_sC_sV_traceroute" "{\"target\": \"$TARGET\", \"flag_o\": true, \"flag_sc\": true, \"flag_sv\": true, \"flag_traceroute\": true}"

echo "📋 Starting Blank Scan Test..." | tee -a "$LOG_FILE"

# Blank Scan (no flags)
test_with_ultra_fast_timings "blank_scan" "{\"target\": \"$TARGET\"}"

echo "🎉 All Ultra-Fast Tests Completed!" | tee -a "$LOG_FILE"
echo "Total tests run: 54 (only T3, T4, T5 timings)" | tee -a "$LOG_FILE"
echo "Results saved in: $RESULTS_DIR/" | tee -a "$LOG_FILE"
echo "Log file: $LOG_FILE" | tee -a "$LOG_FILE"

# Generate summary report
echo "📊 Generating Summary Report..." | tee -a "$LOG_FILE"
summary_file="$RESULTS_DIR/ultra_fast_summary.txt"
echo "Ultra-Fast Nmap Flag Combinations Test Summary" > "$summary_file"
echo "============================================" >> "$summary_file"
echo "Target: $TARGET" >> "$summary_file"
echo "Test Date: $(date)" >> "$summary_file"
echo "Only T3 (Normal), T4 (Aggressive), T5 (Insane) timings" >> "$summary_file"
echo "" >> "$summary_file"
echo "Test Files Created:" >> "$summary_file"
ls -1 "$RESULTS_DIR"/*.json >> "$summary_file"
echo "" >> "$summary_file"
echo "Total JSON files: $(ls -1 "$RESULTS_DIR"/*.json | wc -l)" >> "$summary_file"

echo "Summary report saved to: $summary_file" | tee -a "$LOG_FILE"
echo "🚀 Ultra-Fast Testing Complete! Check $RESULTS_DIR/ for all results." | tee -a "$LOG_FILE"
