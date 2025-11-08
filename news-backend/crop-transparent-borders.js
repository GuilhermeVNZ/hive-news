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

async function cropTransparentBorders(imageName) {
  const inputPath = path.join(imagesDir, imageName);
  const tempPath = path.join(imagesDir, `_temp_${imageName}`);

  try {
    console.log(`Processando ${imageName}...`);

    // Verificar se o arquivo existe
    if (!fs.existsSync(inputPath)) {
      console.error(`Arquivo não encontrado: ${inputPath}`);
      return false;
    }

    // Recortar bordas transparentes usando trim()
    // O trim() remove pixels de borda que são similares ao pixel do canto superior esquerdo
    // Para imagens com transparência, isso remove automaticamente as bordas transparentes
    await sharp(inputPath)
      .trim({
        threshold: 0 // Threshold 0 significa que só remove pixels exatamente transparentes
      })
      .toFile(tempPath);

    // Substituir o arquivo original pelo processado
    fs.unlinkSync(inputPath);
    fs.renameSync(tempPath, inputPath);

    console.log(`✅ ${imageName} processado com sucesso`);
    return true;
  } catch (error) {
    // Limpar arquivo temporário em caso de erro
    if (fs.existsSync(tempPath)) {
      fs.unlinkSync(tempPath);
    }
    console.error(`❌ Erro ao processar ${imageName}:`, error.message);
    return false;
  }
}

async function main() {
  console.log('Iniciando recorte de bordas transparentes...\n');

  const results = {
    success: [],
    failed: []
  };

  for (const image of images) {
    const success = await cropTransparentBorders(image);
    if (success) {
      results.success.push(image);
    } else {
      results.failed.push(image);
    }
  }

  console.log('\n=== Resumo ===');
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

