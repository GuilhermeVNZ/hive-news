#!/usr/bin/env python3
"""
Extrai dados de compressão do terminal e gera análise estatística
"""

import re
from dataclasses import dataclass
from typing import List
from statistics import mean, median, stdev

@dataclass
class CompressionEntry:
    file_num: int
    phase: str
    original: int
    compressed: int
    savings: float
    ratio: float

# Dados extraídos manualmente do terminal
data = [
    # Artigo prompts
    CompressionEntry(1, "article", 22493, 14701, 34.6, 0.654),
    CompressionEntry(2, "article", 16209, 10308, 36.4, 0.636),
    CompressionEntry(3, "article", 10750, 6885, 36.0, 0.640),
    CompressionEntry(4, "article", 23045, 13528, 41.3, 0.587),
    CompressionEntry(5, "article", 15694, 10139, 35.4, 0.646),
    CompressionEntry(6, "article", 36159, 22616, 37.5, 0.625),
    CompressionEntry(7, "article", 12486, 8452, 32.3, 0.677),
    CompressionEntry(8, "article", 35703, 18419, 48.4, 0.516),
    CompressionEntry(9, "article", 15516, 10154, 34.6, 0.654),
    CompressionEntry(10, "article", 27611, 18173, 34.2, 0.658),
    CompressionEntry(11, "article", 27073, 18016, 33.5, 0.665),
    CompressionEntry(12, "article", 14140, 9003, 36.3, 0.637),
    CompressionEntry(13, "article", 29642, 18877, 36.3, 0.637),
    CompressionEntry(14, "article", 18345, 11843, 35.4, 0.646),
    CompressionEntry(15, "article", 18228, 12097, 33.6, 0.664),
    CompressionEntry(16, "article", 18804, 11689, 37.8, 0.622),
    CompressionEntry(17, "article", 32794, 20867, 36.4, 0.636),
    CompressionEntry(18, "article", 25953, 16999, 34.5, 0.655),
    CompressionEntry(19, "article", 47983, 30336, 36.8, 0.632),
    CompressionEntry(20, "article", 23381, 15468, 33.8, 0.662),
    CompressionEntry(21, "article", 11928, 7741, 35.1, 0.649),
    CompressionEntry(22, "article", 9668, 6104, 36.9, 0.631),
    CompressionEntry(23, "article", 18990, 12066, 36.5, 0.635),
    CompressionEntry(24, "article", 20376, 12797, 37.2, 0.628),
    CompressionEntry(25, "article", 11022, 7266, 34.1, 0.659),
    CompressionEntry(26, "article", 16620, 10375, 37.6, 0.624),
    CompressionEntry(27, "article", 17085, 10664, 37.6, 0.624),
    CompressionEntry(28, "article", 12112, 7894, 34.8, 0.652),
    CompressionEntry(29, "article", 27805, 18229, 34.4, 0.656),
    CompressionEntry(30, "article", 11094, 7263, 34.5, 0.655),
    CompressionEntry(31, "article", 18096, 11033, 39.0, 0.610),
    CompressionEntry(32, "article", 14196, 9189, 35.3, 0.647),
    CompressionEntry(33, "article", 28887, 17344, 40.0, 0.600),
    CompressionEntry(34, "article", 11015, 6985, 36.6, 0.634),
    CompressionEntry(35, "article", 21328, 13258, 37.8, 0.622),
    CompressionEntry(36, "article", 15779, 9538, 39.6, 0.604),
    CompressionEntry(37, "article", 6329, 4016, 36.5, 0.635),
    CompressionEntry(38, "article", 17510, 11388, 35.0, 0.650),
    CompressionEntry(39, "article", 17249, 11795, 31.6, 0.684),
    CompressionEntry(40, "article", 21953, 14156, 35.5, 0.645),
    
    # Social prompts
    CompressionEntry(1, "social", 1415, 915, 35.3, 0.647),
    CompressionEntry(2, "social", 1477, 882, 40.3, 0.597),
    CompressionEntry(3, "social", 1393, 884, 36.6, 0.634),
    CompressionEntry(4, "social", 1378, 856, 37.9, 0.621),
    CompressionEntry(5, "social", 1620, 1045, 35.5, 0.645),
    CompressionEntry(6, "social", 1460, 870, 40.4, 0.596),
    CompressionEntry(7, "social", 1259, 767, 39.1, 0.609),
    CompressionEntry(8, "social", 1568, 1027, 34.5, 0.655),
    CompressionEntry(9, "social", 1471, 913, 38.0, 0.620),
    CompressionEntry(10, "social", 1507, 984, 34.7, 0.653),
    CompressionEntry(11, "social", 1331, 870, 34.6, 0.654),
    CompressionEntry(12, "social", 1426, 858, 39.9, 0.601),
    CompressionEntry(13, "social", 1297, 796, 38.6, 0.614),
    CompressionEntry(14, "social", 1250, 758, 39.4, 0.606),
    CompressionEntry(15, "social", 1496, 989, 33.9, 0.661),
    CompressionEntry(16, "social", 1417, 872, 38.5, 0.615),
    CompressionEntry(17, "social", 1521, 966, 36.5, 0.635),
    CompressionEntry(18, "social", 1395, 919, 34.2, 0.658),
    CompressionEntry(19, "social", 1490, 928, 37.8, 0.622),
    CompressionEntry(20, "social", 1503, 1029, 31.5, 0.685),
    CompressionEntry(21, "social", 1584, 1066, 32.7, 0.673),
    CompressionEntry(22, "social", 1390, 887, 36.2, 0.638),
    CompressionEntry(23, "social", 1461, 921, 37.0, 0.630),
    CompressionEntry(24, "social", 1330, 824, 38.0, 0.620),
    CompressionEntry(25, "social", 1578, 991, 37.2, 0.628),
    CompressionEntry(26, "social", 1421, 927, 34.8, 0.652),
    CompressionEntry(27, "social", 1465, 955, 34.9, 0.651),
    CompressionEntry(28, "social", 1369, 824, 39.9, 0.601),
    CompressionEntry(29, "social", 1617, 1101, 32.0, 0.680),
    CompressionEntry(30, "social", 1397, 836, 40.2, 0.598),
    CompressionEntry(31, "social", 1551, 967, 37.7, 0.623),
    CompressionEntry(32, "social", 1418, 909, 35.9, 0.641),
    CompressionEntry(33, "social", 1639, 1057, 35.5, 0.645),
    CompressionEntry(34, "social", 1397, 883, 36.8, 0.632),
    CompressionEntry(35, "social", 1394, 860, 38.4, 0.616),
    CompressionEntry(36, "social", 1522, 989, 35.1, 0.649),
    CompressionEntry(37, "social", 1342, 793, 40.9, 0.591),
    CompressionEntry(38, "social", 1499, 963, 35.8, 0.642),
    CompressionEntry(39, "social", 1516, 958, 36.8, 0.632),
    CompressionEntry(40, "social", 1406, 896, 36.3, 0.637),
]

def analyze_statistics(data: List[CompressionEntry]):
    """Gera análise estatística completa"""
    
    all_savings = [d.savings for d in data]
    all_ratios = [d.ratio for d in data]
    all_original = [d.original for d in data]
    
    article_data = [d for d in data if d.phase == "article"]
    social_data = [d for d in data if d.phase == "social"]
    
    article_savings = [d.savings for d in article_data]
    social_savings = [d.savings for d in social_data]
    
    # Correlação tamanho vs economia
    def correlation(x, y):
        n = len(x)
        if n < 2:
            return 0.0
        mean_x, mean_y = mean(x), mean(y)
        num = sum((x[i] - mean_x) * (y[i] - mean_y) for i in range(n))
        den = (sum((x[i] - mean_x)**2 for i in range(n)) * sum((y[i] - mean_y)**2 for i in range(n)))**0.5
        return num / den if den != 0 else 0.0
    
    size_savings_corr = correlation(all_original, all_savings)
    
    # Por ordem de processamento
    first_half = [d.savings for d in data[:len(data)//2]]
    second_half = [d.savings for d in data[len(data)//2:]]
    
    # Encontrar extremos
    max_savings = max(data, key=lambda x: x.savings)
    min_savings = min(data, key=lambda x: x.savings)
    
    # Por arquivo (média dos dois tipos)
    by_file = {}
    for d in data:
        if d.file_num not in by_file:
            by_file[d.file_num] = []
        by_file[d.file_num].append(d.savings)
    
    file_avgs = {f: mean(savings) for f, savings in by_file.items()}
    
    # Relatório
    report = []
    report.append("=" * 80)
    report.append("ANÁLISE ESTATÍSTICA DOS RESULTADOS DO COMPRESSOR")
    report.append("=" * 80)
    report.append("")
    
    report.append("📊 RESUMO GERAL")
    report.append("-" * 80)
    report.append(f"Total de amostras: {len(data)}")
    report.append(f"  - Compressões de artigo: {len(article_data)}")
    report.append(f"  - Compressões sociais: {len(social_data)}")
    report.append("")
    
    report.append("💾 ESTATÍSTICAS DE ECONOMIA (%)")
    report.append("-" * 80)
    report.append(f"Média: {mean(all_savings):.2f}%")
    report.append(f"Mediana: {median(all_savings):.2f}%")
    report.append(f"Desvio padrão: {stdev(all_savings):.2f}%")
    report.append(f"Mínimo: {min(all_savings):.2f}% (arquivo #{min_savings.file_num}, {min_savings.phase})")
    report.append(f"Máximo: {max(all_savings):.2f}% (arquivo #{max_savings.file_num}, {max_savings.phase})")
    report.append(f"Amplitude: {max(all_savings) - min(all_savings):.2f}%")
    report.append("")
    
    report.append("📉 RATIO DE COMPRESSÃO")
    report.append("-" * 80)
    report.append(f"Média: {mean(all_ratios):.4f} ({mean(all_ratios)*100:.2f}% do original)")
    report.append(f"Mediana: {median(all_ratios):.4f} ({median(all_ratios)*100:.2f}% do original)")
    report.append(f"Desvio padrão: {stdev(all_ratios):.4f}")
    report.append(f"Mínimo: {min(all_ratios):.4f} ({min(all_ratios)*100:.2f}% do original)")
    report.append(f"Máximo: {max(all_ratios):.4f} ({max(all_ratios)*100:.2f}% do original)")
    report.append("")
    
    report.append("📏 TAMANHO ORIGINAL DOS PROMPTS")
    report.append("-" * 80)
    report.append(f"Média: {mean(all_original):.0f} tokens")
    report.append(f"Mediana: {median(all_original):.0f} tokens")
    report.append(f"Desvio padrão: {stdev(all_original):.0f} tokens")
    report.append(f"Mínimo: {min(all_original):.0f} tokens")
    report.append(f"Máximo: {max(all_original):.0f} tokens")
    report.append("")
    
    report.append("🔀 COMPARAÇÃO POR FASE")
    report.append("-" * 80)
    report.append(f"Artigos:")
    report.append(f"  - Média de economia: {mean(article_savings):.2f}%")
    report.append(f"  - Mediana: {median(article_savings):.2f}%")
    report.append(f"  - Desvio padrão: {stdev(article_savings):.2f}%")
    report.append(f"  - Amostras: {len(article_data)}")
    report.append(f"Mídias sociais:")
    report.append(f"  - Média de economia: {mean(social_savings):.2f}%")
    report.append(f"  - Mediana: {median(social_savings):.2f}%")
    report.append(f"  - Desvio padrão: {stdev(social_savings):.2f}%")
    report.append(f"  - Amostras: {len(social_data)}")
    diff = mean(article_savings) - mean(social_savings)
    report.append(f"Diferença: {diff:+.2f}% (artigos {'mais' if diff > 0 else 'menos'} compressíveis que sociais)")
    report.append("")
    
    report.append("🔝 CASOS EXTREMOS")
    report.append("-" * 80)
    report.append("MAIOR ECONOMIA:")
    report.append(f"  Arquivo #{max_savings.file_num} ({max_savings.phase})")
    report.append(f"  Original: {max_savings.original:,} tokens")
    report.append(f"  Comprimido: {max_savings.compressed:,} tokens")
    report.append(f"  Economia: {max_savings.savings:.2f}%")
    report.append(f"  Ratio: {max_savings.ratio:.4f} ({max_savings.ratio*100:.2f}% do original)")
    report.append("")
    report.append("MENOR ECONOMIA:")
    report.append(f"  Arquivo #{min_savings.file_num} ({min_savings.phase})")
    report.append(f"  Original: {min_savings.original:,} tokens")
    report.append(f"  Comprimido: {min_savings.compressed:,} tokens")
    report.append(f"  Economia: {min_savings.savings:.2f}%")
    report.append(f"  Ratio: {min_savings.ratio:.4f} ({min_savings.ratio*100:.2f}% do original)")
    report.append("")
    
    report.append("🔗 ANÁLISES DE CORRELAÇÃO")
    report.append("-" * 80)
    report.append(f"Correlação Tamanho Original vs Economia: {size_savings_corr:.4f}")
    if abs(size_savings_corr) > 0.3:
        direction = "positiva" if size_savings_corr > 0 else "negativa"
        strength = "forte" if abs(size_savings_corr) > 0.7 else "moderada"
        report.append(f"  → Correlação {strength} {direction}")
        if size_savings_corr > 0:
            report.append(f"  → Prompts maiores tendem a ter MAIS economia")
        else:
            report.append(f"  → Prompts maiores tendem a ter MENOS economia")
    else:
        report.append(f"  → Correlação fraca ou inexistente")
        report.append(f"  → Tamanho do prompt NÃO está relacionado com economia")
    report.append("")
    
    report.append("⏱️  ANÁLISE POR ORDEM DE PROCESSAMENTO")
    report.append("-" * 80)
    report.append(f"Primeira metade dos arquivos ({len(first_half)} amostras):")
    report.append(f"  - Média de economia: {mean(first_half):.2f}%")
    report.append(f"Segunda metade dos arquivos ({len(second_half)} amostras):")
    report.append(f"  - Média de economia: {mean(second_half):.2f}%")
    diff_time = mean(second_half) - mean(first_half)
    report.append(f"Diferença: {diff_time:+.2f}%")
    if abs(diff_time) > 1:
        report.append(f"  → {'Melhor compressão' if diff_time > 0 else 'Pior compressão'} na segunda metade")
        report.append(f"  → Pode indicar padrão temporal")
    else:
        report.append(f"  → Compressão consistente ao longo do tempo")
    report.append("")
    
    report.append("🏆 TOP 5 ARQUIVOS COM MAIOR ECONOMIA (média artigo + social)")
    report.append("-" * 80)
    sorted_files = sorted(file_avgs.items(), key=lambda x: x[1], reverse=True)
    for i, (file_num, avg_savings) in enumerate(sorted_files[:5], 1):
        report.append(f"{i}. Arquivo #{file_num}: {avg_savings:.2f}%")
    report.append("")
    
    report.append("📉 TOP 5 ARQUIVOS COM MENOR ECONOMIA (média artigo + social)")
    report.append("-" * 80)
    for i, (file_num, avg_savings) in enumerate(sorted_files[-5:], 1):
        report.append(f"{i}. Arquivo #{file_num}: {avg_savings:.2f}%")
    report.append("")
    
    # Análise quando maior compressão ocorre
    report.append("🔍 ANÁLISE: QUANDO OCORREM MAIORES COMPRESSÕES?")
    report.append("-" * 80)
    high_compression = [d for d in data if d.savings > 40.0]
    report.append(f"Compressões acima de 40%: {len(high_compression)} amostras ({len(high_compression)/len(data)*100:.1f}% do total)")
    if high_compression:
        report.append(f"  Média de tokens original: {mean([d.original for d in high_compression]):.0f}")
        report.append(f"  Média de tokens original (geral): {mean(all_original):.0f}")
        report.append(f"  Distribuição por fase:")
        high_article = [d for d in high_compression if d.phase == "article"]
        high_social = [d for d in high_compression if d.phase == "social"]
        report.append(f"    - Artigos: {len(high_article)}")
        report.append(f"    - Sociais: {len(high_social)}")
    report.append("")
    
    low_compression = [d for d in data if d.savings < 32.5]
    report.append(f"Compressões abaixo de 32.5%: {len(low_compression)} amostras ({len(low_compression)/len(data)*100:.1f}% do total)")
    if low_compression:
        report.append(f"  Média de tokens original: {mean([d.original for d in low_compression]):.0f}")
        report.append(f"  Média de tokens original (geral): {mean(all_original):.0f}")
        report.append(f"  Distribuição por fase:")
        low_article = [d for d in low_compression if d.phase == "article"]
        low_social = [d for d in low_compression if d.phase == "social"]
        report.append(f"    - Artigos: {len(low_article)}")
        report.append(f"    - Sociais: {len(low_social)}")
    report.append("")
    
    report.append("=" * 80)
    
    return "\n".join(report)

if __name__ == "__main__":
    import sys
    import io
    
    # Configurar stdout para UTF-8
    if sys.platform == 'win32':
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    
    report = analyze_statistics(data)
    print(report)
    
    # Salvar em arquivo
    with open("compress_analysis_report.txt", "w", encoding="utf-8") as f:
        f.write(report)
    
    print("\nRelatorio salvo em: compress_analysis_report.txt")

