#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Script para cortar imagens removendo espaço transparente das margens.
Corta da periferia ao centro até 1 pixel antes do primeiro pixel não transparente.
"""

from PIL import Image
import os
import sys
from pathlib import Path

# Configurar encoding UTF-8 para Windows
if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8')


def find_first_non_transparent_pixel(image, direction='top'):
    """
    Encontra o primeiro pixel não transparente na direção especificada.
    
    Args:
        image: PIL Image com canal alpha
        direction: 'top', 'bottom', 'left', 'right'
    
    Returns:
        int: Coordenada do primeiro pixel não transparente
    """
    width, height = image.size
    rgba = image.convert('RGBA')
    pixels = rgba.load()
    
    if direction == 'top':
        for y in range(height):
            for x in range(width):
                if pixels[x, y][3] > 0:  # Alpha > 0 (não transparente)
                    return y
        return height
    
    elif direction == 'bottom':
        for y in range(height - 1, -1, -1):
            for x in range(width):
                if pixels[x, y][3] > 0:
                    return y
        return -1
    
    elif direction == 'left':
        for x in range(width):
            for y in range(height):
                if pixels[x, y][3] > 0:
                    return x
        return width
    
    elif direction == 'right':
        for x in range(width - 1, -1, -1):
            for y in range(height):
                if pixels[x, y][3] > 0:
                    return x
        return -1


def crop_bottom_margin(image_path, output_path=None):
    """
    Corta apenas a margem inferior da imagem, removendo espaço transparente
    de baixo para cima até encontrar o primeiro pixel não transparente.
    
    Args:
        image_path: Caminho da imagem original
        output_path: Caminho para salvar (None = sobrescrever original)
    
    Returns:
        tuple: (sucesso: bool, mensagem: str)
    """
    try:
        # Abrir imagem
        img = Image.open(image_path)
        
        # Converter para RGBA se necessário
        if img.mode != 'RGBA':
            img = img.convert('RGBA')
        
        width, height = img.size
        
        # Encontrar primeiro pixel não transparente vindo de baixo para cima
        bottom = find_first_non_transparent_pixel(img, 'bottom')
        
        # Verificar se encontrou algum conteúdo
        if bottom == -1:
            return False, f"Imagem '{os.path.basename(image_path)}' nao tem conteudo nao transparente"
        
        # Cortar apenas a margem inferior (remover todo espaço transparente abaixo)
        # Manter topo, esquerda e direita inalterados
        crop_left = 0
        crop_top = 0
        crop_right = width
        crop_bottom = bottom + 1  # +1 porque bottom é o índice do último pixel não transparente
        
        # Cortar imagem
        cropped = img.crop((crop_left, crop_top, crop_right, crop_bottom))
        
        # Salvar imagem cortada
        if output_path is None:
            output_path = image_path
        
        # Garantir que o diretório existe
        os.makedirs(os.path.dirname(output_path) if os.path.dirname(output_path) else '.', exist_ok=True)
        
        cropped.save(output_path, format='PNG', optimize=True)
        
        original_size = img.size
        cropped_size = cropped.size
        
        reduction = height - cropped_size[1]
        
        return True, (
            f"[OK] '{os.path.basename(image_path)}' margem inferior cortada: "
            f"{original_size[0]}x{original_size[1]} -> {cropped_size[0]}x{cropped_size[1]} "
            f"(removidos {reduction} pixels da parte inferior)"
        )
    
    except Exception as e:
        return False, f"[ERRO] Erro ao processar '{os.path.basename(image_path)}': {str(e)}"


def crop_fixed_bottom(image_path, pixels_to_crop, output_path=None):
    """
    Corta um número fixo de pixels da margem inferior da imagem.
    
    Args:
        image_path: Caminho da imagem original
        pixels_to_crop: Número de pixels a remover da parte inferior
        output_path: Caminho para salvar (None = sobrescrever original)
    
    Returns:
        tuple: (sucesso: bool, mensagem: str)
    """
    try:
        # Abrir imagem
        img = Image.open(image_path)
        
        # Converter para RGBA se necessário
        if img.mode != 'RGBA':
            img = img.convert('RGBA')
        
        width, height = img.size
        
        # Verificar se há pixels suficientes para cortar
        if pixels_to_crop >= height:
            return False, (
                f"Imagem '{os.path.basename(image_path)}' tem apenas {height} pixels de altura, "
                f"nao e possivel cortar {pixels_to_crop} pixels"
            )
        
        # Cortar pixels da margem inferior
        # Manter topo, esquerda e direita inalterados
        crop_left = 0
        crop_top = 0
        crop_right = width
        crop_bottom = height - pixels_to_crop
        
        # Cortar imagem
        cropped = img.crop((crop_left, crop_top, crop_right, crop_bottom))
        
        # Salvar imagem cortada
        if output_path is None:
            output_path = image_path
        
        # Garantir que o diretório existe
        os.makedirs(os.path.dirname(output_path) if os.path.dirname(output_path) else '.', exist_ok=True)
        
        cropped.save(output_path, format='PNG', optimize=True)
        
        original_size = img.size
        cropped_size = cropped.size
        
        return True, (
            f"[OK] '{os.path.basename(image_path)}' margem inferior cortada ({pixels_to_crop}px): "
            f"{original_size[0]}x{original_size[1]} -> {cropped_size[0]}x{cropped_size[1]} "
            f"(removidos {pixels_to_crop} pixels da parte inferior)"
        )
    
    except Exception as e:
        return False, f"[ERRO] Erro ao processar '{os.path.basename(image_path)}': {str(e)}"


def crop_image_with_margin(image_path, output_path=None, margin=1):
    """
    Corta uma imagem removendo espaço transparente das margens,
    deixando 'margin' pixels de margem transparente.
    
    Args:
        image_path: Caminho da imagem original
        output_path: Caminho para salvar (None = sobrescrever original)
        margin: Número de pixels de margem transparente a manter
    
    Returns:
        tuple: (sucesso: bool, mensagem: str)
    """
    try:
        # Abrir imagem
        img = Image.open(image_path)
        
        # Converter para RGBA se necessário
        if img.mode != 'RGBA':
            img = img.convert('RGBA')
        
        # Encontrar bordas do conteúdo não transparente
        top = find_first_non_transparent_pixel(img, 'top')
        bottom = find_first_non_transparent_pixel(img, 'bottom')
        left = find_first_non_transparent_pixel(img, 'left')
        right = find_first_non_transparent_pixel(img, 'right')
        
        # Verificar se encontrou algum conteúdo
        if top >= bottom or left >= right:
            return False, f"Imagem '{os.path.basename(image_path)}' não tem conteúdo não transparente"
        
        # Calcular coordenadas de corte com margem
        # Subtrair 1 pixel de margem de cada lado
        crop_left = max(0, left - margin)
        crop_top = max(0, top - margin)
        crop_right = min(img.size[0], right + margin + 1)
        crop_bottom = min(img.size[1], bottom + margin + 1)
        
        # Cortar imagem
        cropped = img.crop((crop_left, crop_top, crop_right, crop_bottom))
        
        # Salvar imagem cortada
        if output_path is None:
            output_path = image_path
        
        # Garantir que o diretório existe
        os.makedirs(os.path.dirname(output_path) if os.path.dirname(output_path) else '.', exist_ok=True)
        
        cropped.save(output_path, format='PNG', optimize=True)
        
        original_size = img.size
        cropped_size = cropped.size
        
        return True, (
            f"[OK] '{os.path.basename(image_path)}' cortada: "
            f"{original_size[0]}x{original_size[1]} -> {cropped_size[0]}x{cropped_size[1]} "
            f"(corte: [{crop_left}, {crop_top}, {crop_right}, {crop_bottom}])"
        )
    
    except Exception as e:
        return False, f"[ERRO] Erro ao processar '{os.path.basename(image_path)}': {str(e)}"


def main():
    """Processa as imagens especificadas."""
    import sys
    
    # Caminho base
    base_dir = Path(__file__).parent.parent
    images_dir = base_dir / 'images'
    
    # Verificar argumentos da linha de comando
    if len(sys.argv) > 1 and sys.argv[1] == '--crop-bottom-fixed':
        # Modo: cortar número fixo de pixels da margem inferior
        try:
            pixels_to_crop = int(sys.argv[2]) if len(sys.argv) > 2 else 10
        except ValueError:
            pixels_to_crop = 10
        
        # Se imagens específicas forem fornecidas após o número de pixels, usar essas
        # Caso contrário, usar as padrão
        if len(sys.argv) > 3:
            images = sys.argv[3:]
        else:
            images = ['Lexum.png', 'Vision.png']
        
        print("=" * 60)
        print(f"Cortando margem inferior - Removendo {pixels_to_crop} pixels fixos da parte inferior")
        print("=" * 60)
        print()
        
        results = []
        
        for image_name in images:
            image_path = images_dir / image_name
            
            if not image_path.exists():
                results.append((False, f"[ERRO] Arquivo nao encontrado: '{image_name}'"))
                continue
            
            success, message = crop_fixed_bottom(str(image_path), pixels_to_crop)
            results.append((success, message))
            print(message)
        
        print()
        print("=" * 60)
        successful = sum(1 for success, _ in results if success)
        print(f"Processadas: {successful}/{len(images)} imagens")
        print("=" * 60)
        
        return 0 if successful == len(images) else 1
    
    elif len(sys.argv) > 1 and sys.argv[1] == '--bottom-only':
        # Modo: cortar apenas margem inferior (transparente)
        # Se imagens específicas forem fornecidas após o argumento, usar essas
        # Caso contrário, usar as padrão
        if len(sys.argv) > 2:
            images = sys.argv[2:]
        else:
            images = ['Lexum.png', 'Vision.png']
        
        print("=" * 60)
        print("Cortando margem inferior - Removendo espaço transparente de baixo para cima")
        print("=" * 60)
        print()
        
        results = []
        
        for image_name in images:
            image_path = images_dir / image_name
            
            if not image_path.exists():
                results.append((False, f"[ERRO] Arquivo nao encontrado: '{image_name}'"))
                continue
            
            success, message = crop_bottom_margin(str(image_path))
            results.append((success, message))
            print(message)
        
        print()
        print("=" * 60)
        successful = sum(1 for success, _ in results if success)
        print(f"Processadas: {successful}/{len(images)} imagens")
        print("=" * 60)
        
        return 0 if successful == len(images) else 1
    
    else:
        # Modo padrão: cortar todas as margens
        images = ['Display.png', 'Evolve.png', 'Lexum.png', 'Vision.png']
        
        print("=" * 60)
        print("Cortando imagens - Removendo espaço transparente das margens")
        print("=" * 60)
        print()
        
        results = []
        
        for image_name in images:
            image_path = images_dir / image_name
            
            if not image_path.exists():
                results.append((False, f"[ERRO] Arquivo nao encontrado: '{image_name}'"))
                continue
            
            success, message = crop_image_with_margin(str(image_path))
            results.append((success, message))
            print(message)
        
        print()
        print("=" * 60)
        successful = sum(1 for success, _ in results if success)
        print(f"Processadas: {successful}/{len(images)} imagens")
        print("=" * 60)
        
        return 0 if successful == len(images) else 1


if __name__ == '__main__':
    exit(main())

