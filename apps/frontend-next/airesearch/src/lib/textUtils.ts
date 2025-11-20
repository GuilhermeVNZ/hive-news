/**
 * Utilitários para formatação de texto
 */

/**
 * Garante que cada parágrafo comece com letra maiúscula
 * @param text - Texto com parágrafos separados por quebras de linha duplas
 * @returns Texto com parágrafos capitalizados
 */
export function capitalizeParagraphs(text: string): string {
  if (!text) return text;
  
  // Dividir por quebras de linha duplas (parágrafos)
  const paragraphs = text.split(/\n\s*\n/);
  
  return paragraphs
    .map(paragraph => {
      const trimmed = paragraph.trim();
      if (!trimmed) return trimmed;
      
      // Encontrar a primeira letra alfabética e capitalizá-la
      let result = '';
      let foundLetter = false;
      
      for (let i = 0; i < trimmed.length; i++) {
        const char = trimmed[i];
        if (!foundLetter && /[a-zA-Z]/.test(char)) {
          result += char.toUpperCase();
          foundLetter = true;
        } else {
          result += char;
        }
      }
      
      return result;
    })
    .join('\n\n');
}

/**
 * Formatar texto para exibição, aplicando todas as correções necessárias
 * @param text - Texto original
 * @returns Texto formatado
 */
export function formatArticleText(text: string): string {
  if (!text) return text;
  
  // Aplicar capitalização de parágrafos
  let formatted = capitalizeParagraphs(text);
  
  // Remover espaços extras no início e fim
  formatted = formatted.trim();
  
  return formatted;
}
