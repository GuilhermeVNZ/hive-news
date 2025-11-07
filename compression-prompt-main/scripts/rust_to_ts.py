#!/usr/bin/env python3
"""
Converte statistical_filter.rs para StatisticalFilter.ts
MANTENDO TUDO IGUAL - comentários, stopwords, lógica, TUDO!
"""

import re

def rust_to_typescript(rust_code):
    """Converte código Rust para TypeScript mantendo estrutura"""
    
    ts_code = rust_code
    
    # Converter tipos
    ts_code = ts_code.replace('f64', 'number')
    ts_code = ts_code.replace('f32', 'number')
    ts_code = ts_code.replace('usize', 'number')
    ts_code = ts_code.replace('&str', 'string')
    ts_code = ts_code.replace('String', 'string')
    ts_code = ts_code.replace('Vec<', 'Array<')
    ts_code = ts_code.replace('HashMap<', 'Map<')
    ts_code = ts_code.replace('HashSet<', 'Set<')
    
    # Converter sintaxe de função
    ts_code = re.sub(r'fn (\w+)', r'function \1', ts_code)
    ts_code = re.sub(r'pub fn (\w+)', r'public \1', ts_code)
    ts_code = re.sub(r'const (\w+): &\[&str\]', r'const \1: string[]', ts_code)
    
    # Converter comentários de doc
    ts_code = re.sub(r'///(.+)', r'/**\1 */', ts_code)
    ts_code = re.sub(r'//!(.+)', r'/**\1 */', ts_code)
    
    # Converter arrays Rust para TS
    ts_code = ts_code.replace('&[', '[')
    ts_code = re.sub(r'\[(.+?)\]', r'[\1]', ts_code)
    
    return ts_code

# Ler arquivo Rust
with open('../rust/src/statistical_filter.rs', 'r', encoding='utf-8') as f:
    rust_content = f.read()

# Converter
ts_content = rust_to_typescript(rust_content)

# Salvar
with open('../typescript/src/StatisticalFilter.ts', 'w', encoding='utf-8') as f:
    f.write(ts_content)

print("✅ Conversão completa!")
print(f"Linhas convertidas: {len(ts_content.splitlines())}")

