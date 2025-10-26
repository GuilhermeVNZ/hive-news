import fs from "fs";
import path from "path";
import yaml from "js-yaml";
// import { EditorialContract } from '../../contracts/editorial.contract';

/**
 * Profile Loader Service
 * Loads and validates portal profiles from YAML files
 */

interface PortalProfile {
  portal: {
    id: string;
    name: string;
    domain: string;
    description?: string;
  };
  editorial: {
    style: "scientific" | "tech" | "policy";
    cadence: string;
    language: {
      base: string;
      translate_to: string[];
    };
    image: {
      style: string;
      cover_aspect_ratio: string;
      thumbnail_aspect_ratio: string;
    };
    seo: {
      priority: "high" | "medium" | "low";
      keywords: string[];
    };
  };
  sources: Array<{
    url: string;
    type: "rss" | "api" | "html";
    enabled: boolean;
    refresh: string;
  }>;
  publishing: {
    schedule: {
      frequency: string;
      start_time: string;
      timezone: string;
    };
    channels: string[];
    post_format: {
      include_cover_image: boolean;
      include_schema_markup: boolean;
      include_hreflang: boolean;
    };
  };
  quality: {
    min_score: number;
    fact_check: boolean;
    sources_required: number;
    expert_review: boolean;
  };
  cmmv: {
    contract_namespace: string;
    auto_generate: string[];
  };
}

export class ProfileLoaderService {
  private profilesDir: string;

  constructor(profilesDir = "./configs/portal-profiles") {
    this.profilesDir = profilesDir;
  }

  /**
   * Load a portal profile from YAML file
   */
  async loadProfile(portalId: string): Promise<PortalProfile | null> {
    const filePath = path.join(this.profilesDir, `${portalId}.yaml`);

    try {
      const fileContents = fs.readFileSync(filePath, "utf8");
      const profile = yaml.load(fileContents) as PortalProfile;

      // Validate profile
      this.validateProfile(profile);

      return profile;
    } catch (error) {
      console.error(`Failed to load profile for ${portalId}:`, error);
      return null;
    }
  }

  /**
   * Validate portal profile schema
   */
  private validateProfile(profile: any): void {
    if (!profile.portal || !profile.portal.id) {
      throw new Error("Invalid profile: missing portal.id");
    }

    if (!profile.editorial || !profile.editorial.style) {
      throw new Error("Invalid profile: missing editorial.style");
    }

    if (!profile.sources || !Array.isArray(profile.sources)) {
      throw new Error("Invalid profile: sources must be an array");
    }
  }

  /**
   * Load all portal profiles
   */
  async loadAllProfiles(): Promise<Map<string, PortalProfile>> {
    const profiles = new Map<string, PortalProfile>();
    const files = fs.readdirSync(this.profilesDir);

    for (const file of files) {
      if (file.endsWith(".yaml")) {
        const portalId = file.replace(".yaml", "");
        const profile = await this.loadProfile(portalId);

        if (profile) {
          profiles.set(portalId, profile);
        }
      }
    }

    return profiles;
  }

  /**
   * Watch for profile changes and hot-reload
   */
  watchProfiles(callback: (portalId: string, profile: PortalProfile) => void): void {
    fs.watch(this.profilesDir, (_eventType, filename) => {
      if (filename && filename.endsWith(".yaml")) {
        const portalId = filename.replace(".yaml", "");

        setTimeout(async () => {
          const profile = await this.loadProfile(portalId);
          if (profile) {
            callback(portalId, profile);
          }
        }, 1000);
      }
    });
  }

  /**
   * Convert profile to EditorialContract data
   */
  toEditorialContract(profile: PortalProfile): Record<string, unknown> {
    return {
      name: profile.portal.name,
      sources: profile.sources.map((s) => s.url),
      style: profile.editorial.style,
      langs: {
        base: profile.editorial.language.base,
        translate_to: profile.editorial.language.translate_to,
      },
      cadence: profile.editorial.cadence,
      image_style: profile.editorial.image.style,
      min_rank_to_translate: 0.7,
      seo_priority: profile.editorial.seo.priority,
    };
  }
}
