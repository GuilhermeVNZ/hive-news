/**
 * Ranker Service
 * Calculates article ranking based on freshness, relevance, trend, and social signals
 * Now includes QA feedback loop and dynamic threshold
 */

export interface RankingFactors {
  freshness: number;
  relevance: number;
  trend: number;
  socialSignal: number;
  qaPenalty: number;
  finalRank: number;
}

export interface EngagementMetrics {
  views: number;
  clicks: number;
  timeOnPage: number;
  bounceRate: number;
}

export interface QAValidationInfo {
  isFactual: boolean;
  isNeutralTone: boolean;
  qualityScore: number;
  status: "pending" | "approved" | "rejected";
}

export interface ScientificValidationInfo {
  validation_score: number;
  flagged: boolean;
}

export class RankerService {
  // Weights are now defined inline in calculateRank() method
  // to match the enhanced formula with Scientific Validation

  /**
   * Calculate freshness score (0-1)
   */
  calculateFreshness(publishedAt: Date): number {
    const now = Date.now();
    const published = publishedAt.getTime();
    const ageInHours = (now - published) / (1000 * 60 * 60);

    // Exponential decay: 1.0 for <1h, 0.5 for 24h, 0.1 for 168h (7d)
    if (ageInHours < 1) return 1.0;
    if (ageInHours < 24) return Math.exp(-ageInHours / 24);
    if (ageInHours < 168) return Math.exp(-ageInHours / 168) * 0.5;
    return Math.max(0.1, Math.exp(-ageInHours / 720)); // >30 days
  }

  /**
   * Calculate relevance score (0-1)
   */
  calculateRelevance(
    vectorSimilarity: number,
    keywordScore: number,
    contentQuality: number
  ): number {
    return vectorSimilarity * 0.5 + keywordScore * 0.3 + contentQuality * 0.2;
  }

  /**
   * Calculate trend score (0-1)
   */
  calculateTrend(engagement: EngagementMetrics, trendHistory: number[]): number {
    // Current engagement rate
    const engagementRate = engagement.views > 0 ? engagement.clicks / engagement.views : 0;

    // Growth rate calculation
    if (trendHistory.length < 2) return 0.5;

    const recent = trendHistory.slice(-2);
    const growth = recent[1] - recent[0];
    const growthRate = growth / (recent[0] || 1);

    // Normalize to 0-1
    return Math.min(1.0, Math.max(0.0, engagementRate * 0.5 + growthRate * 0.5));
  }

  /**
   * Calculate social signal score (0-1)
   */
  calculateSocialSignal(engagement: EngagementMetrics): number {
    const ctr = engagement.views > 0 ? engagement.clicks / engagement.views : 0;
    const engagementScore = Math.min(1.0, ctr * 10); // 10% CTR = 1.0
    const bouncePenalty = engagement.bounceRate;

    return engagementScore * (1 - bouncePenalty);
  }

  /**
   * Calculate QA penalty (0-1)
   * Feedback loop with quality assurance to penalize articles that fail checks
   */
  calculateQAPenalty(qaValidation: QAValidationInfo): number {
    // Penalize based on validation status
    let penalty = 1.0;

    // Heavy penalty for rejected articles (50% reduction)
    if (qaValidation.status === "rejected") {
      penalty *= 0.5;
    }
    // Moderate penalty for pending approval (10% reduction)
    else if (qaValidation.status === "pending") {
      penalty *= 0.9;
    }

    // Additional penalty for factual errors (30% reduction)
    if (!qaValidation.isFactual) {
      penalty *= 0.7;
    }

    // Penalize for non-neutral tone (20% reduction)
    if (!qaValidation.isNeutralTone) {
      penalty *= 0.8;
    }

    // Apply quality score multiplier
    penalty *= qaValidation.qualityScore;

    return penalty;
  }

  /**
   * Calculate dynamic threshold based on average rank of last 24h
   * Adjusts threshold to prevent DeepSeek saturation
   */
  calculateDynamicThreshold(last24hRankMean: number): number {
    const baseThreshold = 0.7;

    // If average rank is high (>0.8), articles are generally good
    // Lower threshold to allow more diversity
    // If average rank is low (<0.5), articles need more curation
    // Raise threshold to maintain quality
    const adjustment = (last24hRankMean - 0.5) * 0.2;
    const adjustedThreshold = baseThreshold - adjustment;

    // Keep threshold within reasonable bounds (0.5 to 0.9)
    return Math.max(0.5, Math.min(0.9, adjustedThreshold));
  }

  /**
   * Calculate final rank combining all factors
   * Now includes QA feedback loop and Scientific Validation 🧬
   */
  calculateRank(
    publishedAt: Date,
    vectorSimilarity: number,
    keywordScore: number,
    contentQuality: number,
    engagement: EngagementMetrics,
    trendHistory: number[],
    qaValidation?: QAValidationInfo,
    scientificValidation?: ScientificValidationInfo
  ): RankingFactors {
    const freshness = this.calculateFreshness(publishedAt);
    const relevance = this.calculateRelevance(vectorSimilarity, keywordScore, contentQuality);
    const trend = this.calculateTrend(engagement, trendHistory);
    const socialSignal = this.calculateSocialSignal(engagement);

    // Base rank calculation with Scientific Validation 🧬
    let finalRank =
      freshness * 0.35 + // Reduced from 0.4 to accommodate validation
      relevance * 0.25 + // Reduced from 0.3 to accommodate validation
      trend * 0.2 + // Reduced from 0.2
      socialSignal * 0.1 +
      (scientificValidation?.validation_score ?? 1.0) * 0.1; // Scientific Validation (10%)

    // Apply QA penalty if validation info is provided
    let qaPenalty = 1.0;
    if (qaValidation) {
      qaPenalty = this.calculateQAPenalty(qaValidation);
      finalRank *= qaPenalty;
    }

    // Block flagged scientific papers
    if (scientificValidation?.flagged) {
      finalRank *= 0.1; // Heavy penalty for flagged papers
    }

    return {
      freshness,
      relevance,
      trend,
      socialSignal,
      qaPenalty,
      finalRank,
    };
  }

  /**
   * Schedule rank updates (every 15 minutes)
   */
  getUpdateSchedule(): string {
    return "*/15 * * * *"; // Every 15 minutes
  }
}
