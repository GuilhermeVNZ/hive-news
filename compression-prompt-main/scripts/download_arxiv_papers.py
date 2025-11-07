#!/usr/bin/env python3
"""
Download arXiv papers from CSV list
"""

import csv
import os
import sys
import time
import urllib.request
from pathlib import Path

def download_arxiv_pdf(arxiv_id, output_dir):
    """Download PDF from arXiv using ID"""
    if not arxiv_id or arxiv_id.strip() == '':
        return False, "No arXiv ID"
    
    # Clean arxiv ID (remove version if present)
    arxiv_id = arxiv_id.strip()
    base_id = arxiv_id.split('v')[0]  # Remove version suffix
    
    # Construct URL
    pdf_url = f"https://arxiv.org/pdf/{base_id}.pdf"
    
    # Output filename
    output_file = output_dir / f"{base_id.replace('/', '_')}.pdf"
    
    # Skip if already exists
    if output_file.exists():
        return True, f"Already exists: {output_file.name}"
    
    try:
        print(f"  Downloading {arxiv_id}...", end=" ", flush=True)
        
        # Download with timeout
        headers = {'User-Agent': 'Mozilla/5.0 (compression-prompt research tool)'}
        req = urllib.request.Request(pdf_url, headers=headers)
        
        with urllib.request.urlopen(req, timeout=30) as response:
            content = response.read()
            
        # Write to file
        with open(output_file, 'wb') as f:
            f.write(content)
        
        file_size = len(content) / 1024 / 1024  # MB
        print(f"✓ ({file_size:.1f} MB)")
        return True, f"Downloaded: {output_file.name}"
        
    except urllib.error.HTTPError as e:
        print(f"✗ HTTP {e.code}")
        return False, f"HTTP Error {e.code}"
    except urllib.error.URLError as e:
        print(f"✗ {e.reason}")
        return False, f"URL Error: {e.reason}"
    except Exception as e:
        print(f"✗ {type(e).__name__}: {e}")
        return False, f"Error: {e}"

def main():
    # Paths
    script_dir = Path(__file__).parent
    project_dir = script_dir.parent
    csv_file = project_dir / "data" / "ai_papers.csv"
    output_dir = project_dir / "benchmarks" / "datasets" / "arxiv_pdfs"
    
    # Create output directory
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"📄 Reading papers from: {csv_file}")
    print(f"📁 Output directory: {output_dir}")
    print()
    
    if not csv_file.exists():
        print(f"❌ CSV file not found: {csv_file}")
        sys.exit(1)
    
    # Read CSV
    papers = []
    with open(csv_file, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        for row in reader:
            if row.get('arxiv'):  # Only papers with arXiv ID
                papers.append(row)
    
    total_papers = len(papers)
    print(f"Found {total_papers} papers with arXiv IDs")
    print("=" * 60)
    
    # Download
    success_count = 0
    skip_count = 0
    fail_count = 0
    
    for i, paper in enumerate(papers, 1):
        arxiv_id = paper['arxiv']
        title = paper['title'][:50]  # Truncate for display
        
        print(f"\n[{i}/{total_papers}] {title}")
        print(f"  arXiv: {arxiv_id}")
        
        success, message = download_arxiv_pdf(arxiv_id, output_dir)
        
        if success:
            if "Already exists" in message:
                skip_count += 1
            else:
                success_count += 1
        else:
            fail_count += 1
            print(f"  ⚠️  {message}")
        
        # Rate limiting: wait 3 seconds between downloads
        if i < total_papers and not "Already exists" in message:
            time.sleep(3)
    
    # Summary
    print()
    print("=" * 60)
    print("📊 Download Summary:")
    print(f"  ✅ Downloaded: {success_count}")
    print(f"  ⏭️  Skipped (existing): {skip_count}")
    print(f"  ❌ Failed: {fail_count}")
    print(f"  📦 Total: {total_papers}")
    print()
    
    # List files
    pdf_files = list(output_dir.glob("*.pdf"))
    total_size = sum(f.stat().st_size for f in pdf_files) / 1024 / 1024
    
    print(f"📁 Output directory contains {len(pdf_files)} PDFs ({total_size:.1f} MB)")
    print(f"   Location: {output_dir}")
    
    if fail_count > 0:
        print()
        print("⚠️  Some downloads failed. You can re-run this script to retry.")

if __name__ == "__main__":
    main()

