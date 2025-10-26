/**
 * Style System Service
 * Manages editorial style presets (scientific, tech, policy)
 */

export type EditorialStyle = "scientific" | "tech" | "policy";

export interface StylePreset {
  name: EditorialStyle;
  tone: string;
  format: string;
  structure: string[];
  keywords: string[];
}

export class StyleSystemService {
  private presets: Map<EditorialStyle, StylePreset>;

  constructor() {
    this.presets = new Map();
    this.initializePresets();
  }

  /**
   * Initialize style presets
   */
  private initializePresets(): void {
    this.presets.set("scientific", {
      name: "scientific",
      tone: "Academic, formal, data-driven, citation-heavy",
      format: "Research article format",
      structure: [
        "Abstract",
        "Introduction",
        "Methodology",
        "Findings",
        "Discussion",
        "References",
      ],
      keywords: ["research", "study", "analysis", "findings", "methodology", "hypothesis"],
    });

    this.presets.set("tech", {
      name: "tech",
      tone: "Technical, practical, solution-oriented",
      format: "Technical blog/article format",
      structure: ["Problem", "Solution", "Implementation", "Benefits", "Conclusion"],
      keywords: ["technology", "implementation", "solution", "development", "system", "platform"],
    });

    this.presets.set("policy", {
      name: "policy",
      tone: "Analytical, balanced, implications-focused",
      format: "Policy brief format",
      structure: ["Context", "Current State", "Implications", "Recommendations", "Conclusion"],
      keywords: ["policy", "regulation", "impact", "stakeholders", "recommendations", "analysis"],
    });
  }

  /**
   * Get style preset by name
   */
  getPreset(style: EditorialStyle): StylePreset | undefined {
    return this.presets.get(style);
  }

  /**
   * Validate style choice
   */
  isValidStyle(style: string): style is EditorialStyle {
    return this.presets.has(style as EditorialStyle);
  }

  /**
   * Get all available styles
   */
  getAllStyles(): EditorialStyle[] {
    return Array.from(this.presets.keys());
  }
}

