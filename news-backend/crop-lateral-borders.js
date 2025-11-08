const sharp = require('sharp');
const path = require('path');
const fs = require('fs');

// Lista de imagens a processar
const images = [
  'Classify.png',
  'Compressor.png',
  'Fusion.png',
  'Gov.png',
  'Nexus.png',
  'Rulebook.png',
  'Transmutation.png',
  'Voxa.png'
];

// Diretório das imagens (relativo ao script)
const imagesDir = path.join(__dirname, '..', 'images');

async function cropLateralBorders(imageName) {
  const inputPath = path.join(imagesDir, imageName);
  const tempPath = path.join(imagesDir, `_temp_${imageName}`);

  try {
    console.log(`Processando ${imageName}...`);

    // Verificar se o arquivo existe
    if (!fs.existsSync(inputPath)) {
      console.error(`Arquivo não encontrado: ${inputPath}`);
      return false;
    }

    // Obter metadados da imagem
    const metadata = await sharp(inputPath).metadata();
    const { width, height } = metadata;
    
    console.log(`  Dimensões originais: ${width}x${height}`);

    // Obter dados brutos dos pixels (RGBA)
    const { data, info } = await sharp(inputPath)
      .ensureAlpha()
      .raw()
      .toBuffer({ resolveWithObject: true });

    const channels = info.channels; // Deve ser 4 (RGBA)
    let leftCrop = 0;
    let rightCrop = 0;

    // Encontrar primeira coluna não transparente da esquerda
    columnLoop: for (let x = 0; x < width; x++) {
      for (let y = 0; y < height; y++) {
        const pixelIndex = (y * width + x) * channels;
        const alpha = data[pixelIndex + 3]; // Canal alfa é o 4º (índice 3)
        
        if (alpha !== 0) { // Pixel não é totalmente transparente
          leftCrop = x;
          break columnLoop;
        }
      }
    }

    // Encontrar primeira coluna não transparente da direita
    columnLoop: for (let x = width - 1; x >= 0; x--) {
      for (let y = 0; y < height; y++) {
        const pixelIndex = (y * width + x) * channels;
        const alpha = data[pixelIndex + 3]; // Canal alfa é o 4º (índice 3)
        
        if (alpha !== 0) { // Pixel não é totalmente transparente
          rightCrop = width - 1 - x;
          break columnLoop;
        }
      }
    }

    const newWidth = width - leftCrop - rightCrop;

    if (newWidth <= 0) {
      console.error(`  Erro: A imagem ${imageName} seria totalmente recortada. Pulando.`);
      return false;
    }

    if (leftCrop === 0 && rightCrop === 0) {
      console.log(`  Nenhuma borda lateral transparente encontrada. Mantendo dimensões.`);
      return true;
    }

    console.log(`  Bordas removidas: ${leftCrop}px (esquerda), ${rightCrop}px (direita)`);

    // Extrair a região sem as bordas laterais
    await sharp(inputPath)
      .extract({
        left: leftCrop,
        top: 0, // Manter toda a altura
        width: newWidth,
        height: height // Manter altura completa (1000px)
      })
      .toFile(tempPath);

    // Substituir o arquivo original pelo processado
    fs.unlinkSync(inputPath);
    fs.renameSync(tempPath, inputPath);

    console.log(`✅ ${imageName} processado com sucesso`);
    console.log(`  Dimensões finais: ${newWidth}x${height}\n`);
    return true;
  } catch (error) {
    // Limpar arquivo temporário em caso de erro
    if (fs.existsSync(tempPath)) {
      fs.unlinkSync(tempPath);
    }
    console.error(`❌ Erro ao processar ${imageName}:`, error.message);
    console.log('');
    return false;
  }
}

async function main() {
  console.log('Iniciando recorte de bordas laterais transparentes...\n');
  console.log('Mantendo altura de 1000px, removendo apenas bordas esquerda e direita\n');

  const results = {
    success: [],
    failed: []
  };

  for (const image of images) {
    const success = await cropLateralBorders(image);
    if (success) {
      results.success.push(image);
    } else {
      results.failed.push(image);
    }
  }

  console.log('=== Resumo ===');
  console.log(`✅ Processadas com sucesso: ${results.success.length}`);
  console.log(`❌ Falhas: ${results.failed.length}`);
  
  if (results.success.length > 0) {
    console.log('\nSucesso:');
    results.success.forEach(img => console.log(`  - ${img}`));
  }
  
  if (results.failed.length > 0) {
    console.log('\nFalhas:');
    results.failed.forEach(img => console.log(`  - ${img}`));
  }
}

main().catch(console.error);



















