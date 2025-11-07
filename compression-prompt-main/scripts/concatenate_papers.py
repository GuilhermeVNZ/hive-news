#!/usr/bin/env python3
"""
Concatenate Markdown papers into benchmark prompts
"""

import os
import sys
import json
from pathlib import Path
from typing import List, Dict

def create_metadata(papers: List[Path], output_file: Path) -> Dict:
    """Create metadata JSON for the dataset"""
    metadata = {
        "total_papers": len(papers),
        "output_file": str(output_file),
        "papers": []
    }
    
    total_size = 0
    for paper in papers:
        if paper.exists():
            size = paper.stat().st_size
            total_size += size
            metadata["papers"].append({
                "filename": paper.name,
                "size_bytes": size,
                "arxiv_id": paper.stem  # Filename without extension
            })
    
    metadata["total_size_bytes"] = total_size
    metadata["total_size_mb"] = round(total_size / 1024 / 1024, 2)
    
    return metadata

def concatenate_papers(markdown_dir: Path, output_file: Path, count: int, separator: str = "\n\n---PAPER_SEPARATOR---\n\n"):
    """Concatenate N markdown files into a single prompt"""
    
    # Get all markdown files
    md_files = sorted(markdown_dir.glob("*.md"))
    
    if not md_files:
        print(f"❌ No markdown files found in {markdown_dir}")
        return False
    
    # Take first N files
    selected_files = md_files[:count]
    actual_count = len(selected_files)
    
    print(f"📊 Concatenating {actual_count} papers...")
    print(f"   Output: {output_file}")
    
    # Ensure output directory exists
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    # Concatenate
    total_chars = 0
    with open(output_file, 'w', encoding='utf-8') as out:
        for i, md_file in enumerate(selected_files, 1):
            print(f"   [{i}/{actual_count}] {md_file.name}", end="")
            
            try:
                content = md_file.read_text(encoding='utf-8')
                out.write(content)
                out.write(separator)
                
                total_chars += len(content)
                print(f" ✓ ({len(content):,} chars)")
                
            except Exception as e:
                print(f" ✗ Error: {e}")
    
    # Create metadata
    metadata_file = output_file.with_suffix('.json')
    metadata = create_metadata(selected_files, output_file)
    
    with open(metadata_file, 'w', encoding='utf-8') as f:
        json.dump(metadata, f, indent=2)
    
    print()
    print(f"✅ Concatenation complete!")
    print(f"   Total characters: {total_chars:,}")
    print(f"   File size: {output_file.stat().st_size / 1024 / 1024:.2f} MB")
    print(f"   Metadata: {metadata_file}")
    
    return True

def main():
    # Paths
    script_dir = Path(__file__).parent
    project_dir = script_dir.parent
    md_dir = project_dir / "benchmarks" / "datasets" / "arxiv_markdown"
    prompts_dir = project_dir / "benchmarks" / "datasets" / "prompts"
    
    # Check if markdown directory exists
    if not md_dir.exists():
        print(f"❌ Markdown directory not found: {md_dir}")
        print("   Run convert_pdfs_to_markdown.sh first")
        sys.exit(1)
    
    # Count available files
    md_files = list(md_dir.glob("*.md"))
    total_available = len(md_files)
    
    print(f"📁 Markdown directory: {md_dir}")
    print(f"📊 Available papers: {total_available}")
    print()
    
    if total_available == 0:
        print("❌ No markdown files found")
        sys.exit(1)
    
    # Create benchmark prompts for different sizes
    sizes = [100, 200, 500, 1000]
    
    for size in sizes:
        if size > total_available:
            print(f"⏭️  Skipping {size}-paper benchmark (only {total_available} available)")
            continue
        
        output_file = prompts_dir / f"benchmark_{size}_papers.txt"
        
        print(f"{'='*60}")
        print(f"Creating {size}-paper benchmark")
        print(f"{'='*60}")
        
        success = concatenate_papers(md_dir, output_file, size)
        
        if success:
            print()
    
    # Summary
    print(f"{'='*60}")
    print("📊 Benchmark Prompts Created:")
    print(f"{'='*60}")
    
    for prompt_file in sorted(prompts_dir.glob("benchmark_*.txt")):
        size_mb = prompt_file.stat().st_size / 1024 / 1024
        metadata_file = prompt_file.with_suffix('.json')
        
        if metadata_file.exists():
            with open(metadata_file) as f:
                meta = json.load(f)
                paper_count = meta['total_papers']
        else:
            paper_count = "?"
        
        print(f"  📄 {prompt_file.name}")
        print(f"     Papers: {paper_count}, Size: {size_mb:.2f} MB")
    
    print()
    print("✨ Done!")

if __name__ == "__main__":
    main()

