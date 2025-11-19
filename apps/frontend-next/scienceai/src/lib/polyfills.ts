/**
 * Polyfills e utilitários de compatibilidade
 * Garante que métodos modernos estejam disponíveis em todos os navegadores
 */

/**
 * Polyfill seguro para Element.closest()
 * Garante compatibilidade com navegadores mais antigos
 */
export function polyfillClosest() {
  if (!Element.prototype.closest) {
    Element.prototype.closest = function (selector: string) {
      let element: Element | null = this as Element;
      
      while (element) {
        if (element.matches && element.matches(selector)) {
          return element;
        }
        element = element.parentElement;
      }
      
      return null;
    };
  }
}

/**
 * Type guard para verificar se um EventTarget é um Element
 * Usado para evitar erros como "e.target.closest is not a function"
 */
export function isElement(target: EventTarget | null): target is Element {
  return target !== null && target instanceof Element;
}

/**
 * Type guard seguro para obter Element de um evento
 * Retorna null se não for um Element (evita erros)
 */
export function getElementFromEvent(e: Event): Element | null {
  if (isElement(e.target)) {
    return e.target;
  }
  if (isElement(e.currentTarget)) {
    return e.currentTarget as Element;
  }
  return null;
}

/**
 * Helper para usar closest() de forma segura em event handlers
 * Evita erros "closest is not a function"
 * 
 * @example
 * ```tsx
 * const handleClick = (e: MouseEvent) => {
 *   const element = safeClosest(e.target, 'a');
 *   if (element) {
 *     // element é um <a> válido
 *   }
 * };
 * ```
 */
export function safeClosest(
  target: EventTarget | null,
  selector: string
): Element | null {
  if (!isElement(target)) {
    return null;
  }
  
  // Garantir que closest existe
  if (typeof target.closest !== 'function') {
    // Usar polyfill inline se necessário
    let element: Element | null = target;
    while (element) {
      if (element.matches && element.matches(selector)) {
        return element;
      }
      element = element.parentElement;
    }
    return null;
  }
  
  return target.closest(selector);
}

/**
 * Inicializar polyfills na inicialização da aplicação
 */
export function initializePolyfills() {
  // Polyfill closest() se não existir
  polyfillClosest();
}









