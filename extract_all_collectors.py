#!/usr/bin/env python3
"""Extract all news collectors from system_config.json"""

import json
from pathlib import Path

config_path = Path("news-backend/system_config.json")
config = json.loads(config_path.read_text(encoding='utf-8'))

# Extract all collectors from all sites
all_collectors = []
sites = config.get("sites", {})
for site_name, site_config in sites.items():
    collectors_list = site_config.get("collectors", [])
    for c in collectors_list:
        c["_site"] = site_name
        all_collectors.append(c)

collectors = all_collectors

rss_collectors = []
html_collectors = []

for c in collectors:
    ctype = c.get("collector_type")
    name = c.get("name") or c.get("id", "Unknown")
    enabled = c.get("enabled", False)
    
    if ctype == "rss":
        feed_url = c.get("feed_url", "")
        rss_collectors.append((name, feed_url, c.get("id", ""), enabled))
    elif ctype == "html":
        base_url = c.get("base_url", "")
        html_collectors.append((name, base_url, c.get("id", ""), enabled))

print(f"Total RSS Collectors: {len(rss_collectors)}")
print(f"Total HTML Collectors: {len(html_collectors)}")
print(f"Total: {len(rss_collectors) + len(html_collectors)}\n")

print("=" * 80)
print("RSS COLLECTORS")
print("=" * 80)
for name, url, id, enabled in rss_collectors:
    status = "[ENABLED]" if enabled else "[DISABLED]"
    print(f'{status} {name:30} | {id:25} | {url}')

print("\n" + "=" * 80)
print("HTML COLLECTORS")
print("=" * 80)
for name, url, id, enabled in html_collectors:
    status = "[ENABLED]" if enabled else "[DISABLED]"
    print(f'{status} {name:30} | {id:25} | {url}')

# Generate Rust test code
print("\n\n" + "=" * 80)
print("RUST TEST CODE")
print("=" * 80)
print("\nlet rss_sites = vec![")
for name, url, id, enabled in rss_collectors:
    if enabled and url:
        print(f'    ("{name}", "{url}"),')
print("];\n")

print("let html_sites = vec![")
for name, url, id, enabled in html_collectors:
    if enabled and url:
        print(f'    ("{name}", "{url}", "{id}"),')
print("];")

