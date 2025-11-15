# 📋 Análise das Fontes de Notícias Novas

**Data:** 2025-11-06  
**Total de Fontes Novas:** 43  
**Site de Destino:** Todas configuradas para `scienceai`

---

## 🔍 Problemas Identificados

### 1. **Fontes que retornam 0 artigos (HTML)**
Muitas fontes HTML estão retornando 0 artigos, possíveis causas:
- CSS selectors não correspondem à estrutura atual do site
- Sites requerem JavaScript rendering (Playwright)
- Sites bloqueando scrapers (HTTP 403/429)
- URLs incorretas ou páginas não existentes (HTTP 404)
- Sites temporariamente fora do ar

### 2. **Fontes RSS com erro 404**
Algumas fontes RSS estão retornando HTTP 404, indicando que:
- Feed URL pode estar incorreta
- Feed pode ter sido removido ou movido
- Site pode não oferecer mais RSS

### 3. **Lógica de Destinos**
Todas as fontes RSS/HTML estão sendo enviadas para o site `scienceai` porque:
- A função `get_enabled_sites_for_source()` procura sites que têm collectors RSS/HTML habilitados
- Como todas as fontes novas estão no site `scienceai`, todas vão para lá
- **Isso está correto** - o problema não é os destinos, mas sim a coleta de artigos

---

## 📊 Lista Completa das 43 Fontes Novas

### 🤖 **Robótica (13 fontes)**

| ID | Nome | Tipo | URL Base | Status nos Logs |
|---|---|---|---|---|
| `html_boston_dynamics` | Boston Dynamics | HTML | `https://www.bostondynamics.com/news` | ⚠️ 0 artigos |
| `html_irobot` | iRobot | HTML | `https://www.irobot.com/about-irobot/newsroom` | ❌ HTTP 404 |
| `html_robotics_org` | Robotics Online | HTML | `https://www.robotics.org/news` | ⚠️ 0 artigos |
| `html_abb_robotics` | ABB Robotics | HTML | `https://new.abb.com/news` | ⚠️ 0 artigos |
| `html_fanuc` | FANUC Robotics | HTML | `https://www.fanuc.com/americas/en/news` | ❌ HTTP 404 |
| `html_kuka` | KUKA Robotics | HTML | `https://www.kuka.com/en-us/press/media-news` | ❌ HTTP 404 |
| `html_universal_robots` | Universal Robots | HTML | `https://www.universal-robots.com/news` | ❌ HTTP 404 |
| `html_omron` | OMRON Robotics | HTML | `https://automation.omron.com/en/us/news` | ❌ HTTP 404 |
| `html_yaskawa` | Yaskawa Robotics | HTML | `https://www.yaskawa.com/news` | ❌ HTTP 404 |
| `html_agility` | Agility Robotics | HTML | `https://www.agilityrobotics.com/news` | ⚠️ Não testado |
| `html_unitree` | Unitree Robotics | HTML | `https://www.unitree.com/news` | ⚠️ 0 artigos |
| `html_robot_report` | The Robot Report | HTML | `https://www.therobotreport.com/news` | ⚠️ 0 artigos |
| `html_robotics_business_review` | Robotics Business Review | HTML | `https://roboticsbusinessreview.com/news` | ⚠️ 0 artigos |
| `rss_ieee_robotics` | IEEE Spectrum Robotics RSS | RSS | `https://spectrum.ieee.org/topic/robotics/rss` | ❌ HTTP 404 |

### 🔬 **Computação Quântica (10 fontes)**

| ID | Nome | Tipo | URL Base | Status nos Logs |
|---|---|---|---|---|
| `html_quantum_computing_report` | Quantum Computing Report | HTML | `https://quantumcomputingreport.com/news` | ⚠️ 0 artigos |
| `html_ibm_quantum` | IBM Quantum Blog | HTML | `https://research.ibm.com/blog/` | ⚠️ 0 artigos |
| `rss_quanta_quantum` | Quanta Magazine RSS | RSS | `https://www.quantamagazine.org/feed/` | ✅ Funciona (duplicados) |
| `html_rigetti` | Rigetti Computing | HTML | `https://www.rigetti.com/news` | ⚠️ 0 artigos |
| `html_ionq` | IonQ | HTML | `https://ionq.com/news` | ⚠️ 0 artigos |
| `html_dwave` | D-Wave Systems | HTML | `https://www.dwavesys.com/news` | ❌ HTTP 404 |
| `html_quantinuum` | Quantinuum | HTML | `https://www.quantinuum.com/news` | ⚠️ Não testado |
| `html_pasqal` | Pasqal | HTML | `https://www.pasqal.com/news` | ⚠️ 0 artigos |
| `html_quantum_ml` | Quantum ML | HTML | `https://www.quantum-ml.com/blog` | ⚠️ 0 artigos |
| `html_xanadu` | Xanadu Quantum Blog | HTML | `https://www.xanadu.ai/blog` | ⚠️ Não testado |
| `html_coldquanta` | ColdQuanta | HTML | `https://www.coldquanta.com/news` | ⚠️ 0 artigos |
| `html_qci` | Quantum Computing Inc | HTML | `https://www.quantumcomputinginc.com/news` | ⚠️ Não testado |
| `html_quantum_machines` | Quantum Machines | HTML | `https://www.quantum-machines.io/news` | ❌ DNS Error |

### 🤖 **IA - Empresas e Startups (20 fontes)**

| ID | Nome | Tipo | URL Base | Status nos Logs |
|---|---|---|---|---|
| `html_anthropic_research` | Anthropic Research | HTML | `https://www.anthropic.com/research` | ✅ Funciona (4 artigos) |
| `html_adept` | Adept AI | HTML | `https://www.adept.ai/blog` | ⚠️ Não testado |
| `html_assemblyai` | AssemblyAI Blog | HTML | `https://www.assemblyai.com/blog` | ⚠️ 0 artigos |
| `html_replicate` | Replicate Blog | HTML | `https://replicate.com/blog` | ⚠️ 0 artigos |
| `html_langchain` | LangChain Blog | HTML | `https://blog.langchain.dev/` | ⚠️ 0 artigos |
| `html_pinecone` | Pinecone Blog | HTML | `https://www.pinecone.io/learn` | ⚠️ Não testado |
| `html_weaviate` | Weaviate Blog | HTML | `https://weaviate.io/blog` | ⚠️ 0 artigos |
| `html_together` | Together AI Blog | HTML | `https://www.together.ai/blog` | ⚠️ 0 artigos |
| `html_anyscale` | Anyscale Blog | HTML | `https://www.anyscale.com/blog` | ⚠️ 0 artigos |
| `html_modal` | Modal Blog | HTML | `https://www.modal.com/blog` | ⚠️ 0 artigos |
| `html_cursor` | Cursor Blog | HTML | `https://www.cursor.com/blog` | ⚠️ 0 artigos |
| `html_continual` | Continual AI | HTML | `https://www.continual.ai/blog` | ⚠️ 0 artigos |
| `html_fastai` | Fast.ai | HTML | `https://www.fast.ai/posts` | ❌ HTTP 404 |
| `html_eleuther` | EleutherAI Blog | HTML | `https://www.eleuther.ai/blog` | ❌ HTTP 404 |
| `rss_lesswrong` | LessWrong RSS | RSS | `https://www.lesswrong.com/feed.xml` | ✅ Funciona (5 artigos) |
| `rss_alignment_forum` | Alignment Forum RSS | RSS | `https://www.alignmentforum.org/feed.xml` | ⚠️ 0 artigos |
| `html_meta_ai` | Meta AI | HTML | `https://ai.meta.com/blog/` | ✅ Funciona (5 artigos) |
| `html_deepseek` | DeepSeek | HTML | `https://deepseek.ai/blog` | ⚠️ Duplicados |
| `html_menlo_ventures` | Menlo Ventures AI | HTML | `https://menlovc.com/focus-areas/ai/` | ⚠️ 0 artigos |
| `html_airesearch_news` | AIResearch News | HTML | `https://www.airesearch.news/` | ⚠️ 0 artigos |
| `html_ai_trends` | AI Trends | HTML | `https://www.aitrends.com/news` | ❌ HTTP 503 |
| `html_the_gradient` | The Gradient | HTML | `https://thegradient.pub/` | ⚠️ 0 artigos |
| `html_robohub` | Robohub | HTML | `https://robohub.org` | ⚠️ 0 artigos |

---

## 🔧 **Ações Recomendadas**

### **Prioridade Alta:**
1. **Verificar URLs HTTP 404:**
   - `html_irobot` - `https://www.irobot.com/about-irobot/newsroom`
   - `html_fanuc` - `https://www.fanuc.com/americas/en/news`
   - `html_kuka` - `https://www.kuka.com/en-us/press/media-news`
   - `html_universal_robots` - `https://www.universal-robots.com/news`
   - `html_omron` - `https://automation.omron.com/en/us/news`
   - `html_yaskawa` - `https://www.yaskawa.com/news`
   - `html_dwave` - `https://www.dwavesys.com/news`
   - `html_fastai` - `https://www.fast.ai/posts`
   - `html_eleuther` - `https://www.eleuther.ai/blog`
   - `rss_ieee_robotics` - `https://spectrum.ieee.org/topic/robotics/rss`

2. **Verificar DNS Error:**
   - `html_quantum_machines` - `https://www.quantum-machines.io/news` (DNS error)

3. **Verificar HTTP 503:**
   - `html_ai_trends` - `https://www.aitrends.com/news` (Service Unavailable)

### **Prioridade Média:**
4. **Fontes com 0 artigos (verificar se precisam de JS rendering ou novos selectors):**
   - `html_boston_dynamics`
   - `html_robotics_org`
   - `html_abb_robotics`
   - `html_unitree`
   - `html_robot_report`
   - `html_robotics_business_review`
   - `html_quantum_computing_report`
   - `html_ibm_quantum`
   - `html_rigetti`
   - `html_ionq`
   - `html_pasqal`
   - `html_quantum_ml`
   - `html_coldquanta`
   - `html_assemblyai`
   - `html_replicate`
   - `html_langchain`
   - `html_weaviate`
   - `html_together`
   - `html_anyscale`
   - `html_modal`
   - `html_cursor`
   - `html_continual`
   - `html_menlo_ventures`
   - `html_airesearch_news`
   - `html_the_gradient`
   - `html_robohub`

### **Prioridade Baixa:**
5. **Fontes que funcionam mas retornam duplicados:**
   - `html_deepseek` - Artigos já coletados anteriormente
   - `rss_quanta_quantum` - Artigos já coletados anteriormente

6. **Fontes que funcionam corretamente:**
   - `html_anthropic_research` - ✅ Funciona
   - `html_meta_ai` - ✅ Funciona
   - `rss_lesswrong` - ✅ Funciona

---

## 📝 **Notas para Verificação Manual**

### **Para cada fonte com problema:**

1. **Acesse a URL manualmente no navegador**
2. **Verifique:**
   - A URL existe? (404)
   - A página carrega? (503, DNS)
   - A estrutura HTML mudou? (0 artigos - selectors incorretos)
   - Precisa de JavaScript? (0 artigos - precisa Playwright)
   - Está bloqueando scrapers? (403, 429)

3. **Para fontes HTML com 0 artigos:**
   - Inspecione o HTML da página (F12 → Elements)
   - Identifique os seletores CSS corretos para:
     - Container de artigos (`article`)
     - Título (`title`)
     - Link (`link`)
     - Conteúdo (`content`)
   - Compare com os selectors atuais no `system_config.json`
   - Verifique se precisa adicionar o domínio à lista de JS rendering em `html_collector.rs`

4. **Para fontes RSS com 404:**
   - Verifique se o site oferece RSS
   - Procure por feed alternativo (ex: `/feed`, `/rss`, `/atom.xml`)
   - Considere converter para HTML collector se RSS não disponível

---

## 🎯 **Resumo por Status**

- ✅ **Funcionando:** 3 fontes (anthropic_research, meta_ai, lesswrong)
- ⚠️ **0 artigos:** 26 fontes (precisa verificar selectors/JS)
- ❌ **HTTP 404:** 10 fontes (URLs incorretas ou mudadas)
- ❌ **Outros erros:** 2 fontes (DNS error, HTTP 503)
- ⚠️ **Duplicados:** 2 fontes (já coletados antes)

**Total:** 43 fontes novas

---

## 🔄 **Próximos Passos**

1. Verificar manualmente todas as URLs com HTTP 404
2. Testar cada fonte HTML com 0 artigos para identificar o problema
3. Atualizar selectors CSS no `system_config.json` conforme necessário
4. Adicionar domínios que precisam de JS rendering à lista em `html_collector.rs`
5. Considerar converter fontes RSS com 404 para HTML collectors
6. Testar novamente após correções































