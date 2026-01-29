#!/usr/bin/env python3
"""
Quick Nmap Flag Combinations Test Runner
Tests all 108 combinations with Python
"""

import requests
import json
import time
import os
from datetime import datetime

TARGET = "scanme.nmap.org"
BASE_URL = "http://localhost:8081/scan-open-ports"
RESULTS_DIR = "test_results"

# Create results directory
os.makedirs(RESULTS_DIR, exist_ok=True)

# Initialize log
log_file = f"nmap_test_results_{datetime.now().strftime('%Y%m%d_%H%M%S')}.log"
with open(log_file, 'w') as f:
    f.write("🚀 Starting Nmap Flag Combinations Test\n")
    f.write(f"Target: {TARGET}\n")
    f.write(f"Results will be saved to: {RESULTS_DIR}/\n")
    f.write(f"Log file: {log_file}\n")
    f.write("========================================\n")

def run_test(test_name, json_payload):
    """Run a single test and save results"""
    output_file = f"{RESULTS_DIR}/{test_name}.json"
    
    print(f"Testing: {test_name}")
    print(f"Payload: {json_payload}")
    
    try:
        # Make the request
        start_time = time.time()
        response = requests.post(BASE_URL, 
                               headers={"Content-Type": "application/json"},
                               json=json.loads(json_payload))
        end_time = time.time()
        
        # Calculate metrics
        time_taken = end_time - start_time
        http_code = response.status_code
        
        # Save results
        result = {
            "test_name": test_name,
            "payload": json.loads(json_payload),
            "response": response.json(),
            "http_code": http_code,
            "time_taken": time_taken,
            "timestamp": datetime.now().isoformat()
        }
        
        with open(output_file, 'w') as f:
            json.dump(result, f, indent=2)
        
        # Log results
        print(f"HTTP Code: {http_code}")
        print(f"Time: {time_taken:.2f}s")
        print(f"Output saved to: {output_file}")
        print("---")
        
        # Log to file
        with open(log_file, 'a') as f:
            f.write(f"Testing: {test_name}\n")
            f.write(f"Payload: {json_payload}\n")
            f.write(f"HTTP Code: {http_code}\n")
            f.write(f"Time: {time_taken:.2f}s\n")
            f.write(f"Output saved to: {output_file}\n")
            f.write("---\n")
        
        # Small delay between tests
        time.sleep(2)
        
    except Exception as e:
        print(f"Error in {test_name}: {str(e)}")
        with open(log_file, 'a') as f:
            f.write(f"Error in {test_name}: {str(e)}\n")

def test_with_timings(base_name, base_payload):
    """Test a combination with all timing templates"""
    # Test with no timing (blank)
    run_test(f"{base_name}_no_timing", base_payload)
    
    # Test with all timing templates
    for timing in ["T0", "T1", "T2", "T3", "T4", "T5"]:
        timed_payload = base_payload.rstrip('}') + f', "timing": "{timing}"}}'
        run_test(f"{base_name}_{timing}", timed_payload)

def main():
    print("🚀 Starting Nmap Flag Combinations Test")
    print(f"Target: {TARGET}")
    print(f"Results will be saved to: {RESULTS_DIR}/")
    print(f"Log file: {log_file}")
    print("========================================")
    
    print("📋 Starting Single Flag Tests...")
    
    # Single Flag Tests
    test_with_timings("single_O", f'{{"target": "{TARGET}", "flag_o": true}}')
    test_with_timings("single_sC", f'{{"target": "{TARGET}", "flag_sc": true}}')
    test_with_timings("single_sV", f'{{"target": "{TARGET}", "flag_sv": true}}')
    test_with_timings("single_traceroute", f'{{"target": "{TARGET}", "flag_traceroute": true}}')
    test_with_timings("single_A", f'{{"target": "{TARGET}", "flag_a": true}}')
    
    print("📋 Starting Two Flag Combination Tests...")
    
    # Two Flag Combinations
    test_with_timings("combo_O_sC", f'{{"target": "{TARGET}", "flag_o": true, "flag_sc": true}}')
    test_with_timings("combo_O_sV", f'{{"target": "{TARGET}", "flag_o": true, "flag_sv": true}}')
    test_with_timings("combo_O_traceroute", f'{{"target": "{TARGET}", "flag_o": true, "flag_traceroute": true}}')
    test_with_timings("combo_sC_sV", f'{{"target": "{TARGET}", "flag_sc": true, "flag_sv": true}}')
    test_with_timings("combo_sC_traceroute", f'{{"target": "{TARGET}", "flag_sc": true, "flag_traceroute": true}}')
    test_with_timings("combo_sV_traceroute", f'{{"target": "{TARGET}", "flag_sv": true, "flag_traceroute": true}}')
    test_with_timings("combo_A_traceroute", f'{{"target": "{TARGET}", "flag_a": true, "flag_traceroute": true}}')
    
    print("📋 Starting Three Flag Combination Tests...")
    
    # Three Flag Combinations
    test_with_timings("combo_O_sC_sV", f'{{"target": "{TARGET}", "flag_o": true, "flag_sc": true, "flag_sv": true}}')
    test_with_timings("combo_O_sC_traceroute", f'{{"target": "{TARGET}", "flag_o": true, "flag_sc": true, "flag_traceroute": true}}')
    test_with_timings("combo_O_sV_traceroute", f'{{"target": "{TARGET}", "flag_o": true, "flag_sv": true, "flag_traceroute": true}}')
    test_with_timings("combo_sC_sV_traceroute", f'{{"target": "{TARGET}", "flag_sc": true, "flag_sv": true, "flag_traceroute": true}}')
    
    print("📋 Starting Four Flag Combination Test...")
    
    # Four Flag Combination
    test_with_timings("combo_O_sC_sV_traceroute", f'{{"target": "{TARGET}", "flag_o": true, "flag_sc": true, "flag_sv": true, "flag_traceroute": true}}')
    
    print("📋 Starting Blank Scan Test...")
    
    # Blank Scan (no flags)
    test_with_timings("blank_scan", f'{{"target": "{TARGET}"}}')
    
    print("🎉 All Tests Completed!")
    print("Total tests run: 108")
    print(f"Results saved in: {RESULTS_DIR}/")
    print(f"Log file: {log_file}")
    
    # Generate summary report
    print("📊 Generating Summary Report...")
    summary_file = f"{RESULTS_DIR}/test_summary.txt"
    with open(summary_file, 'w') as f:
        f.write("Nmap Flag Combinations Test Summary\n")
        f.write("=================================\n")
        f.write(f"Target: {TARGET}\n")
        f.write(f"Test Date: {datetime.now()}\n")
        f.write("\nTest Files Created:\n")
        for file in os.listdir(RESULTS_DIR):
            if file.endswith('.json'):
                f.write(f"{file}\n")
        f.write(f"\nTotal JSON files: {len([f for f in os.listdir(RESULTS_DIR) if f.endswith('.json')])}\n")
    
    print(f"Summary report saved to: {summary_file}")
    print("🚀 Testing Complete! Check test_results/ for all results.")

if __name__ == "__main__":
    main()
