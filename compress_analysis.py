#!/usr/bin/env python3
"""
Análise Estatística dos Resultados do Compressor
Extrai dados do terminal e gera estatísticas detalhadas
"""

import re
from dataclasses import dataclass
from typing import List, Dict
from statistics import mean, median, stdev
from collections import defaultdict

@dataclass
class CompressionData:
    """Dados de uma compressão"""
    file_number: int
    original_tokens: int
    compressed_tokens: int
    savings_percent: float
    compression_ratio: float
    phase: str  # 'article' ou 'social'
    total_original: int = 0
    total_compressed: int = 0
    total_savings: float = 0.0

def extract_compression_data(terminal_text: str) -> List[CompressionData]:
    """Extrai dados de compressão do texto do terminal"""
    data = []
    
    # Padrões para encontrar dados
    # Exemplo: "🗜️  Compressing prompt (~22493 tokens)..."
    # Exemplo: "✅ Compressed to 14701 tokens (34.6% savings)"
    
    lines = terminal_text.split('\n')
    current_file = None
    current_phase = None
    current_original = None
    
    i = 0
    while i < len(lines):
        line = lines[i]
        
        # Detectar início de processamento de arquivo
        file_match = re.search(r'\[(\d+)/\d+\] Processing:', line)
        if file_match:
            current_file = int(file_match.group(1))
            continue
        
        # Detectar fase (article ou social)
        if 'Phase 1: Generating article' in line or 'Building article prompt' in line:
            current_phase = 'article'
        elif 'Building social media prompts' in line:
            current_phase = 'social'
        
        # Detectar compressão
        compress_match = re.search(r'Compressing prompt \(~(\d+) tokens\)', line)
        if compress_match:
            current_original = int(compress_match.group(1))
            i += 1
            # Procurar linha seguinte com resultado
            if i < len(lines):
                result_match = re.search(r'Compressed to (\d+) tokens \(([\d.]+)% savings\)', lines[i])
                if result_match and current_original and current_file:
                    compressed = int(result_match.group(1))
                    savings = float(result_match.group(2))
                    
                    data.append(CompressionData(
                        file_number=current_file,
                        original_tokens=current_original,
                        compressed_tokens=compressed,
                        savings_percent=savings,
                        compression_ratio=compressed / current_original,
                        phase=current_phase or 'unknown'
                    ))
        
        # Detectar totais (para alguns casos)
        total_match = re.search(r'Tokens: (\d+) → (\d+) \(([\d.]+)% savings\)', line)
        if total_match and current_file:
            total_orig = int(total_match.group(1))
            total_comp = int(total_match.group(2))
            total_sav = float(total_match.group(3))
            
            # Encontrar dados correspondentes e adicionar totais
            for d in data:
                if d.file_number == current_file and d.total_original == 0:
                    d.total_original = total_orig
                    d.total_compressed = total_comp
                    d.total_savings = total_sav
        
        i += 1
    
    return data

def analyze_statistics(data: List[CompressionData]) -> Dict:
    """Analisa estatísticas dos dados de compressão"""
    if not data:
        return {}
    
    # Separar por fase
    article_data = [d for d in data if d.phase == 'article']
    social_data = [d for d in data if d.phase == 'social']
    
    # Análise geral
    savings_all = [d.savings_percent for d in data]
    ratio_all = [d.compression_ratio for d in data]
    original_all = [d.original_tokens for d in data]
    
    # Análise por fase
    savings_article = [d.savings_percent for d in article_data]
    savings_social = [d.savings_percent for d in social_data]
    
    # Encontrar máximos e mínimos
    max_savings = max(data, key=lambda x: x.savings_percent)
    min_savings = min(data, key=lambda x: x.savings_percent)
    
    max_ratio = max(data, key=lambda x: x.compression_ratio)
    min_ratio = min(data, key=lambda x: x.compression_ratio)
    
    # Correlações
    # Correlação entre tamanho original e economia
    size_savings_corr = calculate_correlation(
        [d.original_tokens for d in data],
        [d.savings_percent for d in data]
    )
    
    # Análise por número do arquivo (ordem de processamento)
    by_file = defaultdict(list)
    for d in data:
        by_file[d.file_number].append(d.savings_percent)
    
    file_avg_savings = {f: mean(savings) for f, savings in by_file.items()}
    
    # Primeiros vs últimos arquivos
    first_half = [d.savings_percent for d in data if d.file_number <= len(by_file) // 2]
    second_half = [d.savings_percent for d in data if d.file_number > len(by_file) // 2]
    
    stats = {
        'total_samples': len(data),
        'article_samples': len(article_data),
        'social_samples': len(social_data),
        
        # Estatísticas gerais
        'savings': {
            'mean': mean(savings_all),
            'median': median(savings_all),
            'stdev': stdev(savings_all) if len(savings_all) > 1 else 0,
            'min': min(savings_all),
            'max': max(savings_all),
            'range': max(savings_all) - min(savings_all)
        },
        
        'compression_ratio': {
            'mean': mean(ratio_all),
            'median': median(ratio_all),
            'stdev': stdev(ratio_all) if len(ratio_all) > 1 else 0,
            'min': min(ratio_all),
            'max': max(ratio_all)
        },
        
        'original_tokens': {
            'mean': mean(original_all),
            'median': median(original_all),
            'stdev': stdev(original_all) if len(original_all) > 1 else 0,
            'min': min(original_all),
            'max': max(original_all)
        },
        
        # Por fase
        'by_phase': {
            'article': {
                'mean': mean(savings_article) if savings_article else 0,
                'samples': len(article_data)
            },
            'social': {
                'mean': mean(savings_social) if savings_social else 0,
                'samples': len(social_data)
            }
        },
        
        # Extremos
        'max_savings': {
            'file': max_savings.file_number,
            'phase': max_savings.phase,
            'original': max_savings.original_tokens,
            'compressed': max_savings.compressed_tokens,
            'savings': max_savings.savings_percent,
            'ratio': max_savings.compression_ratio
        },
        
        'min_savings': {
            'file': min_savings.file_number,
            'phase': min_savings.phase,
            'original': min_savings.original_tokens,
            'compressed': min_savings.compressed_tokens,
            'savings': min_savings.savings_percent,
            'ratio': min_savings.compression_ratio
        },
        
        # Correlações
        'correlations': {
            'size_vs_savings': size_savings_corr
        },
        
        # Por ordem de processamento
        'processing_order': {
            'first_half_mean': mean(first_half) if first_half else 0,
            'second_half_mean': mean(second_half) if second_half else 0,
            'difference': mean(second_half) - mean(first_half) if first_half and second_half else 0
        },
        
        # Detalhes por arquivo
        'file_details': file_avg_savings
    }
    
    return stats

def calculate_correlation(x: List[float], y: List[float]) -> float:
    """Calcula correlação de Pearson"""
    if len(x) != len(y) or len(x) < 2:
        return 0.0
    
    n = len(x)
    mean_x = mean(x)
    mean_y = mean(y)
    
    numerator = sum((x[i] - mean_x) * (y[i] - mean_y) for i in range(n))
    sum_sq_x = sum((x[i] - mean_x) ** 2 for i in range(n))
    sum_sq_y = sum((y[i] - mean_y) ** 2 for i in range(n))
    
    denominator = (sum_sq_x * sum_sq_y) ** 0.5
    
    if denominator == 0:
        return 0.0
    
    return numerator / denominator

def format_report(stats: Dict) -> str:
    """Formata relatório estatístico"""
    report = []
    report.append("=" * 80)
    report.append("ANÁLISE ESTATÍSTICA DOS RESULTADOS DO COMPRESSOR")
    report.append("=" * 80)
    report.append("")
    
    # Resumo geral
    report.append("📊 RESUMO GERAL")
    report.append("-" * 80)
    report.append(f"Total de amostras: {stats['total_samples']}")
    report.append(f"  - Compressões de artigo: {stats['article_samples']}")
    report.append(f"  - Compressões sociais: {stats['social_samples']}")
    report.append("")
    
    # Estatísticas de economia
    report.append("💾 ESTATÍSTICAS DE ECONOMIA (%)")
    report.append("-" * 80)
    s = stats['savings']
    report.append(f"Média: {s['mean']:.2f}%")
    report.append(f"Mediana: {s['median']:.2f}%")
    report.append(f"Desvio padrão: {s['stdev']:.2f}%")
    report.append(f"Mínimo: {s['min']:.2f}% (arquivo {stats['min_savings']['file']}, {stats['min_savings']['phase']})")
    report.append(f"Máximo: {s['max']:.2f}% (arquivo {stats['max_savings']['file']}, {stats['max_savings']['phase']})")
    report.append(f"Amplitude: {s['range']:.2f}%")
    report.append("")
    
    # Estatísticas de ratio de compressão
    report.append("📉 RATIO DE COMPRESSÃO (tokens comprimidos / tokens originais)")
    report.append("-" * 80)
    r = stats['compression_ratio']
    report.append(f"Média: {r['mean']:.4f} ({r['mean']*100:.2f}% do original)")
    report.append(f"Mediana: {r['median']:.4f} ({r['median']*100:.2f}% do original)")
    report.append(f"Desvio padrão: {r['stdev']:.4f}")
    report.append(f"Mínimo: {r['min']:.4f} ({r['min']*100:.2f}% do original)")
    report.append(f"Máximo: {r['max']:.4f} ({r['max']*100:.2f}% do original)")
    report.append("")
    
    # Tamanho original dos prompts
    report.append("📏 TAMANHO ORIGINAL DOS PROMPTS (tokens)")
    report.append("-" * 80)
    t = stats['original_tokens']
    report.append(f"Média: {t['mean']:.0f} tokens")
    report.append(f"Mediana: {t['median']:.0f} tokens")
    report.append(f"Desvio padrão: {t['stdev']:.0f} tokens")
    report.append(f"Mínimo: {t['min']:.0f} tokens")
    report.append(f"Máximo: {t['max']:.0f} tokens")
    report.append("")
    
    # Por fase
    report.append("🔀 COMPARAÇÃO POR FASE")
    report.append("-" * 80)
    bp = stats['by_phase']
    report.append(f"Artigos:")
    report.append(f"  - Média de economia: {bp['article']['mean']:.2f}%")
    report.append(f"  - Amostras: {bp['article']['samples']}")
    report.append(f"Mídias sociais:")
    report.append(f"  - Média de economia: {bp['social']['mean']:.2f}%")
    report.append(f"  - Amostras: {bp['social']['samples']}")
    report.append(f"Diferença: {bp['article']['mean'] - bp['social']['mean']:.2f}% (artigos vs sociais)")
    report.append("")
    
    # Casos extremos
    report.append("🔝 CASOS EXTREMOS")
    report.append("-" * 80)
    report.append("MAIOR ECONOMIA:")
    ms = stats['max_savings']
    report.append(f"  Arquivo #{ms['file']} ({ms['phase']})")
    report.append(f"  Original: {ms['original']:,} tokens")
    report.append(f"  Comprimido: {ms['compressed']:,} tokens")
    report.append(f"  Economia: {ms['savings']:.2f}%")
    report.append(f"  Ratio: {ms['ratio']:.4f}")
    report.append("")
    report.append("MENOR ECONOMIA:")
    mins = stats['min_savings']
    report.append(f"  Arquivo #{mins['file']} ({mins['phase']})")
    report.append(f"  Original: {mins['original']:,} tokens")
    report.append(f"  Comprimido: {mins['compressed']:,} tokens")
    report.append(f"  Economia: {mins['savings']:.2f}%")
    report.append(f"  Ratio: {mins['ratio']:.4f}")
    report.append("")
    
    # Correlações
    report.append("🔗 ANÁLISES DE CORRELAÇÃO")
    report.append("-" * 80)
    corr = stats['correlations']
    report.append(f"Tamanho original vs Economia: {corr['size_vs_savings']:.4f}")
    if abs(corr['size_vs_savings']) > 0.3:
        direction = "positiva" if corr['size_vs_savings'] > 0 else "negativa"
        strength = "forte" if abs(corr['size_vs_savings']) > 0.7 else "moderada"
        report.append(f"  → Correlação {strength} {direction}")
        report.append(f"  → {'Maiores' if corr['size_vs_savings'] > 0 else 'Menores'} prompts têm {'mais' if corr['size_vs_savings'] > 0 else 'menos'} economia")
    else:
        report.append(f"  → Correlação fraca ou inexistente")
    report.append("")
    
    # Ordem de processamento
    report.append("⏱️  ANÁLISE POR ORDEM DE PROCESSAMENTO")
    report.append("-" * 80)
    po = stats['processing_order']
    report.append(f"Primeira metade dos arquivos:")
    report.append(f"  - Média de economia: {po['first_half_mean']:.2f}%")
    report.append(f"Segunda metade dos arquivos:")
    report.append(f"  - Média de economia: {po['second_half_mean']:.2f}%")
    report.append(f"Diferença: {po['difference']:+.2f}%")
    if abs(po['difference']) > 1:
        report.append(f"  → {'Melhor compressão' if po['difference'] > 0 else 'Pior compressão'} na segunda metade")
    else:
        report.append(f"  → Compressão consistente ao longo do tempo")
    report.append("")
    
    # Top 5 e Bottom 5 arquivos
    report.append("🏆 TOP 5 ARQUIVOS COM MAIOR ECONOMIA")
    report.append("-" * 80)
    sorted_files = sorted(stats['file_details'].items(), key=lambda x: x[1], reverse=True)
    for i, (file_num, savings) in enumerate(sorted_files[:5], 1):
        report.append(f"{i}. Arquivo #{file_num}: {savings:.2f}%")
    report.append("")
    
    report.append("📉 TOP 5 ARQUIVOS COM MENOR ECONOMIA")
    report.append("-" * 80)
    for i, (file_num, savings) in enumerate(sorted_files[-5:], 1):
        report.append(f"{i}. Arquivo #{file_num}: {savings:.2f}%")
    report.append("")
    
    report.append("=" * 80)
    
    return "\n".join(report)

if __name__ == "__main__":
    # Ler dados do terminal (você pode passar o texto do terminal aqui)
    terminal_text = """
    [Cole aqui o texto do terminal]
    """
    
    # Para testar, vou usar um exemplo baseado no que vi no terminal
    # Você pode modificar isso para ler de um arquivo ou stdin
    print("Executando análise estatística...")
    print("Por favor, forneça os dados do terminal ou modifique o código para lê-los de um arquivo.")






