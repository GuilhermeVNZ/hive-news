from pathlib import Path
text = Path("news-backend/src/collectors/arxiv_collector.rs").read_text(encoding="utf-8")
start = text.index("            if line.contains(\"</entry>\")")
end = start
while not text[end:].startswith("                }") or text[end:start+2000].count("{") != text[end:start+2000].count("}"):
    end += 1
# Instead easier: print snippet to adjust manually
a = text[start:start+400]
print(repr(a))
