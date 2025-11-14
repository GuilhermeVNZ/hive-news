export interface Article {
  id: number;
  title: string;
  category: string;
  image: string;
  excerpt: string;
  content: string;
  date: string;
  author: string;
  readTime: number;
  featured?: boolean;
}

export const articles: Article[] = [
  {
    id: 1,
    title: "DeepMind Unveils Revolutionary Reinforcement Learning Algorithm",
    category: "openai",
    image: "/src/assets/hero-ai.jpg",
    excerpt: "A groundbreaking new algorithm pushes the boundaries of autonomous adaptation and decision-making in complex environments.",
    content: `DeepMind has announced a major breakthrough in reinforcement learning that could transform how AI systems learn and adapt. The new algorithm, dubbed "AdaptiveRL-X", demonstrates unprecedented performance in multi-task learning scenarios.

The research team, led by Dr. Sarah Chen, developed a system that can transfer knowledge between vastly different domains with minimal retraining. This represents a significant step toward more general artificial intelligence.

Early tests show the algorithm achieving superhuman performance in strategic games, robotic control tasks, and real-world optimization problems. The system learns from both successes and failures, continuously refining its strategies.

"This is a paradigm shift in how we approach machine learning," says Dr. Chen. "Rather than training separate models for each task, we now have a unified framework that learns to learn."

Industry experts predict this technology could accelerate progress in autonomous vehicles, drug discovery, and climate modeling. Several tech giants have already expressed interest in licensing the technology.`,
    date: "2025-10-26",
    author: "ScienceAI Team",
    readTime: 5,
    featured: true,
  },
  {
    id: 2,
    title: "Humanoid Robots Begin Industrial Deployment at Scale",
    category: "nvidia",
    image: "/src/assets/hero-robotics.jpg",
    excerpt: "Major manufacturers announce plans to integrate advanced humanoid robots into factory floors worldwide.",
    content: `Tesla, Boston Dynamics, and Figure AI have jointly announced the largest deployment of humanoid robots in industrial history. Over 10,000 units will be deployed across manufacturing facilities in the next 18 months.

These next-generation robots feature advanced dexterity, computer vision, and natural language processing. They can work alongside human workers, learning tasks through demonstration and adapting to changing production requirements.

The robots are designed with safety as a priority, incorporating multiple redundant systems and sophisticated collision avoidance. They can operate for 16 hours on a single charge and perform maintenance on themselves.

"This isn't about replacing humans," explains robotics engineer Maria Rodriguez. "It's about augmenting human capabilities and taking over dangerous, repetitive tasks."

Early pilot programs show productivity increases of 40% while reducing workplace injuries by 60%. Union representatives have negotiated retraining programs for affected workers.`,
    date: "2025-10-25",
    author: "Tech Research Division",
    readTime: 6,
    featured: true,
  },
  {
    id: 3,
    title: "CRISPR Breakthrough Enables Cure for Rare Genetic Diseases",
    category: "anthropic",
    image: "/src/assets/hero-medicine.jpg",
    excerpt: "New gene-editing technique shows 95% success rate in clinical trials for previously untreatable conditions.",
    content: `A revolutionary advancement in CRISPR technology has enabled doctors to cure several rare genetic diseases that were previously considered untreatable. The new technique, called "Precision CRISPR-3", offers unprecedented accuracy and safety.

Clinical trials involving 200 patients with various genetic disorders showed remarkable results. 95% of participants experienced complete remission, with no significant adverse effects reported.

The breakthrough comes from a refinement in the guide RNA design, allowing for more precise targeting of genetic mutations. This dramatically reduces off-target effects, which have been a major concern in gene therapy.

Dr. James Liu, lead researcher at the Genomic Medicine Institute, calls it "a watershed moment in medical history." The FDA has fast-tracked approval for three specific genetic conditions.

The treatment involves a single injection that delivers the CRISPR components directly to affected cells. Patients typically show improvement within weeks, with full effects manifesting over 3-6 months.`,
    date: "2025-10-24",
    author: "Medical Sciences Team",
    readTime: 7,
    featured: true,
  },
  {
    id: 4,
    title: "NASA Confirms Discovery of Earth-Like Exoplanet in Habitable Zone",
    category: "google",
    image: "/src/assets/hero-space.jpg",
    excerpt: "James Webb Space Telescope detects biosignatures on planet located 120 light-years away.",
    content: `NASA scientists have confirmed the discovery of an Earth-like exoplanet showing strong indicators of potential habitability. Located 120 light-years away in the constellation Cygnus, the planet dubbed "Terra Nova-1" orbits within its star's habitable zone.

The James Webb Space Telescope detected water vapor, oxygen, and methane in the planet's atmosphere—a combination that on Earth is primarily produced by biological processes.

"This is the most promising candidate for extraterrestrial life we've ever found," states Dr. Elena Kowalski, principal investigator of the exoplanet research program.

The planet has a mass 1.3 times that of Earth and receives similar levels of stellar radiation. Surface temperatures are estimated between 0-30°C, ideal for liquid water.

Further observations are planned to study seasonal variations and search for technosignatures. The discovery has reignited discussions about humanity's place in the universe.`,
    date: "2025-10-23",
    author: "Astronomy Division",
    readTime: 6,
    featured: true,
  },
  {
    id: 5,
    title: "Quantum Computing Achieves Practical Error Correction Milestone",
    category: "google",
    image: "/src/assets/hero-data.jpg",
    excerpt: "New error correction codes bring quantum computers closer to solving real-world problems at scale.",
    content: `IBM and Google have independently demonstrated practical quantum error correction systems that maintain qubit coherence for over one hour—a 1000x improvement over previous records.

This breakthrough addresses the biggest obstacle in quantum computing: qubits are extremely fragile and lose their quantum properties quickly due to environmental interference.

The new approach uses a lattice of physical qubits to encode a single logical qubit, with sophisticated algorithms constantly monitoring and correcting errors in real-time.

"We've crossed the error correction threshold," explains quantum physicist Dr. Michael Zhang. "This means we can now scale quantum computers without exponentially increasing error rates."

The technology enables quantum computers to tackle problems in drug discovery, materials science, and cryptography that are impossible for classical computers.

Several pharmaceutical companies have already begun using the systems to simulate molecular interactions for drug development.`,
    date: "2025-10-22",
    author: "Quantum Research Team",
    readTime: 5,
    featured: true,
  },
  {
    id: 6,
    title: "Neural Implants Restore Speech to Paralyzed Patient",
    category: "deepseek",
    image: "/src/assets/hero-medicine.jpg",
    excerpt: "Brain-computer interface translates neural signals into words with 97% accuracy.",
    content: `A paralyzed stroke patient has regained the ability to communicate through thought alone, thanks to a revolutionary neural implant. The brain-computer interface translates neural activity into synthesized speech with 97% accuracy.

The patient, who lost the ability to speak five years ago, can now hold conversations at nearly normal speaking rates. The system uses machine learning to decode intended words from brain signals.

"This technology is giving voice to the voiceless," says neuroscientist Dr. Amy Peterson. "We're restoring one of the most fundamental human capabilities."

The implant consists of 256 electrodes placed on the brain's speech motor cortex. It wirelessly transmits signals to a decoder that converts neural patterns into text and speech.

Patients undergo a brief calibration period where the system learns their unique brain patterns. After training, they can speak simply by thinking the words.

Clinical trials are expanding to include patients with ALS, locked-in syndrome, and other conditions affecting speech.`,
    date: "2025-10-21",
    author: "Neurotechnology Lab",
    readTime: 6,
  },
  {
    id: 7,
    title: "Solar Energy Efficiency Reaches 50% in Laboratory Tests",
    category: "nvidia",
    image: "/src/assets/hero-data.jpg",
    excerpt: "Perovskite-silicon tandem cells double the efficiency of conventional solar panels.",
    content: `Researchers at MIT have achieved a record-breaking 50% efficiency in converting sunlight to electricity using innovative tandem solar cells. This doubles the performance of standard silicon panels.

The breakthrough combines layers of perovskite and silicon semiconductors, each optimized to capture different parts of the solar spectrum. Together, they convert far more sunlight into usable energy.

"This is a game-changer for renewable energy," states materials scientist Dr. Robert Chang. "We can now generate twice the power from the same surface area."

The new cells are also cheaper to manufacture than traditional silicon panels. Mass production is expected to begin within two years.

If deployed at scale, this technology could dramatically accelerate the transition away from fossil fuels. Energy analysts project solar could become the dominant global energy source by 2035.

Several governments have already committed billions to support commercialization of the technology.`,
    date: "2025-10-20",
    author: "Energy Research Group",
    readTime: 5,
  },
  {
    id: 8,
    title: "AI System Predicts Protein Structures with Perfect Accuracy",
    category: "openai",
    image: "/src/assets/hero-ai.jpg",
    excerpt: "AlphaFold 4 achieves 100% accuracy in protein structure prediction benchmark.",
    content: `DeepMind's latest AlphaFold iteration has achieved perfect accuracy on the most challenging protein structure prediction benchmark, marking a major milestone in computational biology.

AlphaFold 4 not only predicts static structures but also models protein dynamics, interactions, and how structures change in different cellular environments.

This capability is revolutionizing drug design, allowing researchers to identify drug candidates with unprecedented speed and precision. What once took years can now be accomplished in days.

"We're entering a golden age of drug discovery," explains computational biologist Dr. Lisa Kumar. "AlphaFold is democratizing access to structural biology."

The system has already contributed to the development of promising treatments for Alzheimer's, cancer, and antibiotic-resistant infections.

DeepMind has made AlphaFold 4 freely available to academic researchers, with over 100,000 scientists already using the platform.`,
    date: "2025-10-19",
    author: "Biotech Analysis",
    readTime: 5,
  },
];

export const categories = [
  { name: "NVIDIA", slug: "nvidia", icon: "Cpu" },
  { name: "OpenAI", slug: "openai", icon: "Brain" },
  { name: "Google", slug: "google", icon: "Search" },
  { name: "Anthropic", slug: "anthropic", icon: "Sparkles" },
  { name: "DeepSeek", slug: "deepseek", icon: "Target" },
  { name: "Meta", slug: "meta", icon: "Sparkles" },
];
