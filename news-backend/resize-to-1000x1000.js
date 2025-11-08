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
const targetSize = 1000;

async function resizeToSquare(imageName) {
  const inputPath = path.join(imagesDir, imageName);
  const tempPath = path.join(imagesDir, `_temp_${imageName}`);

  try {
    console.log(`Processando ${imageName}...`);

    // Verificar se o arquivo existe
    if (!fs.existsSync(inputPath)) {
      console.error(`Arquivo não encontrado: ${inputPath}`);
      return false;
    }

    // Obter metadados da imagem para verificar dimensões atuais
    const metadata = await sharp(inputPath).metadata();
    console.log(`  Dimensões originais: ${metadata.width}x${metadata.height}`);

    // Redimensionar para 1000x1000 usando 'contain' para manter aspect ratio
    // Isso vai adicionar padding transparente se necessário, sem recortar
    await sharp(inputPath)
      .resize(targetSize, targetSize, {
        fit: 'contain', // Mantém aspect ratio e adiciona padding
        background: { r: 0, g: 0, b: 0, alpha: 0 } // Fundo transparente
      })
      .toFile(tempPath);

    // Verificar dimensões finais
    const finalMetadata = await sharp(tempPath).metadata();
    console.log(`  Dimensões finais: ${finalMetadata.width}x${finalMetadata.height}`);

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
  console.log('Iniciando redimensionamento para 1000x1000...\n');
  console.log('Modo: contain (sem recortar, mantém aspect ratio)\n');

  const results = {
    success: [],
    failed: []
  };

  for (const image of images) {
    const success = await resizeToSquare(image);
    if (success) {
      results.success.push(image);
    } else {
      results.failed.push(image);
    }
    console.log(''); // Linha em branco entre processamentos
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



















