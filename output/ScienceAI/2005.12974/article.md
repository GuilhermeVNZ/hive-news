**Opening Hook**
Recommender systems shape what we see online, from movies to loans, but they often favor popular items, leaving lesser-known providers behind. This research shows how to make these systems fairer to all providers while still giving users what they want, addressing a critical gap in how digital platforms operate.

**Key Finding**
Researchers discovered that users have varying levels of openness to diversity across different item features, such as a movie's genre or a loan's purpose. By tailoring recommendations to each user's specific tolerance for diversity, they can increase fairness for underrepresented providers without significantly reducing recommendation accuracy.

**Methodology**
The team developed a re-ranking algorithm called OFAiR that adjusts initial recommendation lists. It uses information entropy from users' past interactions to measure their tolerance for diversity across features like genre or country of origin. The algorithm then promotes items from protected groups—such as unpopular movies or loans from specific regions—only when users show receptivity, balancing fairness and relevance.

**Results Analysis**
Experiments on movie and loan datasets showed that OFAiR improved protected group exposure by up to 30% compared to standard methods, with minimal loss in accuracy metrics like precision and recall. For example, in the movie dataset, it increased recommendations for underrepresented genres like Horror and History when users had diverse preferences in those areas, as detailed in the paper's evaluation section.

**Context**
This matters because fairer recommendations can help small businesses, indie filmmakers, and underserved loan applicants gain visibility, creating more equitable digital marketplaces. For everyday users, it means discovering a wider range of options without compromising personal relevance, enhancing both choice and fairness in online experiences.

**Limitations**
The approach relies on identifying protected groups and assumes user preferences are stable; it may not handle rapidly changing tastes or new fairness dimensions well, as noted in the paper's discussion of future work.