#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Script para redimensionar imagens para um tamanho específico, distorcendo se necessário.
Preserva todo o conteúdo da imagem, sem cortes.
"""

from PIL import Image
import os
import sys
from pathlib import Path

# Configurar encoding UTF-8 para Windows
if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8')


def resize_image_to_fixed_dimensions(image_path, target_width, target_height, output_path=None):
    """
    Redimensiona uma imagem para as dimensões especificadas, distorcendo se necessário.
    A imagem inteira é preservada, sem cortes.

    Args:
        image_path: Caminho da imagem original
        target_width: Largura desejada em pixels
        target_height: Altura desejada em pixels
        output_path: Caminho para salvar (None = sobrescrever original)
    
    Returns:
        tuple: (sucesso: bool, mensagem: str)
    """
    try:
        # Abrir imagem
        img = Image.open(image_path)
        original_size = img.size
        
        # Preservar modo RGBA se necessário
        if img.mode != 'RGBA':
            img = img.convert('RGBA')
        
        # Redimensionar a imagem para as dimensões exatas (distorce se necessário)
        resized_img = img.resize((target_width, target_height), Image.LANCZOS)
        
        # Salvar a imagem redimensionada
        if output_path is None:
            output_path = image_path
        
        # Garantir que o diretório existe
        os.makedirs(os.path.dirname(output_path) if os.path.dirname(output_path) else '.', exist_ok=True)
        
        resized_img.save(output_path, format='PNG', optimize=True)
        
        original_aspect = original_size[0] / original_size[1]
        target_aspect = target_width / target_height
        distortion = "sim" if abs(original_aspect - target_aspect) > 0.01 else "nao"
        
        return True, (
            f"[OK] '{os.path.basename(image_path)}' redimensionada: "
            f"{original_size[0]}x{original_size[1]} -> {target_width}x{target_height} "
            f"(distorcao: {distortion})"
        )
    
    except FileNotFoundError:
        return False, f"[ERRO] Arquivo nao encontrado: '{os.path.basename(image_path)}'"
    except Exception as e:
        return False, f"[ERRO] Erro ao processar '{os.path.basename(image_path)}': {str(e)}"


def main():
    """Processa as imagens especificadas para redimensionamento."""
    # Caminho base
    base_dir = Path(__file__).parent.parent
    images_dir = base_dir / 'images'
    
    # Imagens a serem processadas
    images_to_process = ['Lexum.png', 'Evolve.png', 'Display.png', 'Vision.png']
    
    target_width = 1000
    target_height = 1000
    
    print("=" * 60)
    print(f"Redimensionando imagens para {target_width}x{target_height} pixels")
    print("Preservando todo o conteudo (distorcendo se necessario)")
    print("=" * 60)
    print()
    
    results = []
    
    for image_name in images_to_process:
        image_path = images_dir / image_name
        
        if not image_path.exists():
            results.append((False, f"[ERRO] Arquivo nao encontrado: '{image_name}'"))
            continue
        
        success, message = resize_image_to_fixed_dimensions(
            str(image_path), 
            target_width, 
            target_height
        )
        results.append((success, message))
        print(message)
    
    print()
    print("=" * 60)
    successful = sum(1 for success, _ in results if success)
    total = len(results)
    print(f"Processadas: {successful}/{total} imagens redimensionadas com sucesso")
    print("=" * 60)
    
    return 0 if successful == total else 1


if __name__ == '__main__':
    exit(main())
















