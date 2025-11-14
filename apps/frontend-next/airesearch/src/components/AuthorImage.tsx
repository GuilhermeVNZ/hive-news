"use client";

import Image from "next/image";
import { useState } from "react";

export function AuthorImage() {
  const [imageError, setImageError] = useState(false);

  if (imageError) {
    return (
      <div className="w-16 h-16 bg-primary rounded-full flex items-center justify-center text-primary-foreground text-xl font-bold">
        G
      </div>
    );
  }

  return (
    <div className="w-16 h-16 rounded-full overflow-hidden flex-shrink-0">
      <Image
        src="/images/Author.jpeg"
        alt="Guilherme A."
        width={64}
        height={64}
        className="w-full h-full object-cover"
        onError={() => setImageError(true)}
      />
    </div>
  );
}


