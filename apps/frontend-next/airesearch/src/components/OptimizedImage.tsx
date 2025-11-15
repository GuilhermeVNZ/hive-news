import Image, { ImageProps } from "next/image";

/**
 * Componente de imagem otimizado com placeholder inteligente
 * Usa blur ou dominant color para melhorar percepção de performance
 */
interface OptimizedImageProps extends Omit<ImageProps, "placeholder"> {
  blurPlaceholder?: string;
  priority?: boolean;
}

export function OptimizedImage({
  src,
  alt,
  priority = false,
  blurPlaceholder = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/2wBDAQkJCQwLDBgNDRgyIRwhMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjL/wAARCAAIAAoDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAv/xAAhEAACAQMDBQAAAAAAAAAAAAABAgMABAUGIWGRkqGx0f/EABUBAQEAAAAAAAAAAAAAAAAAAAMF/8QAGhEAAgIDAAAAAAAAAAAAAAAAAAECEgMRkf/aAAwDAQACEQMRAD8AltJagyeH0AthI5xdrLcNM91BF5pX2HaH9bcfaSXWGaRmknyJckliyjqTzSlT54b6bk+h0R//9k=",
  ...props
}: OptimizedImageProps) {
  return (
    <Image
      src={src}
      alt={alt}
      priority={priority}
      placeholder="blur"
      blurDataURL={blurPlaceholder}
      // Otimizações automáticas do Next.js:
      // - AVIF/WebP conversão
      // - Responsive srcset
      // - Lazy loading quando não é priority
      {...props}
    />
  );
}
